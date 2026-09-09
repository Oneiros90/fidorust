import { SvelteMap } from 'svelte/reactivity';
import type { Engine } from '../app/engine.svelte';

type Job = { key: string; name: string };

function cacheKey(theme: string, name: string) {
	return `${theme}:${name}`;
}

function scheduleIdle(fn: (deadline?: IdleDeadline) => void) {
	const w = globalThis as typeof globalThis & {
		requestIdleCallback?: (cb: IdleRequestCallback, opts?: IdleRequestOptions) => number;
	};
	if (typeof w.requestIdleCallback === 'function') {
		w.requestIdleCallback((deadline) => fn(deadline), { timeout: 200 });
	} else {
		requestAnimationFrame(() => fn());
	}
}

/** Library thumbs: generate off the frame loop, a few tessellations per idle slice. */
export class PreviewCache {
	#svgs = new SvelteMap<string, string>();
	#queued = new Set<string>();
	#queue: Job[] = [];
	#busy = false;

	get(theme: string, name: string): string | undefined {
		return this.#svgs.get(cacheKey(theme, name));
	}

	request(engine: Engine, theme: string, name: string) {
		const key = cacheKey(theme, name);
		if (this.#svgs.has(key) || this.#queued.has(key)) return;
		this.#queued.add(key);
		this.#queue.push({ key, name });
		this.#pump(engine);
	}

	clear() {
		this.#svgs.clear();
		this.#queued.clear();
		this.#queue = [];
		this.#busy = false;
	}

	invalidateStems(stems: string[]) {
		if (stems.length === 0) return;
		const prefixes = stems.map((s) => `${s}.`);
		const hit = (name: string) => prefixes.some((p) => name.startsWith(p));
		for (const key of [...this.#svgs.keys()]) {
			const name = key.slice(key.indexOf(':') + 1);
			if (hit(name)) this.#svgs.delete(key);
		}
		this.#queue = this.#queue.filter((job) => {
			if (!hit(job.name)) return true;
			this.#queued.delete(job.key);
			return false;
		});
	}

	#pump(engine: Engine) {
		if (this.#busy) return;
		this.#busy = true;
		const step = (deadline?: IdleDeadline) => {
			const budget = () =>
				deadline && typeof deadline.timeRemaining === 'function' ? deadline.timeRemaining() : 8;
			let n = 0;
			while (this.#queue.length && n < 6 && (n === 0 || budget() > 4)) {
				const job = this.#queue.shift();
				if (!job) break;
				this.#queued.delete(job.key);
				if (!this.#svgs.has(job.key)) {
					const raw = engine.query((app) => app.component_preview_svg(job.name));
					if (raw) this.#svgs.set(job.key, raw);
				}
				n++;
			}
			if (this.#queue.length) scheduleIdle(step);
			else this.#busy = false;
		};
		scheduleIdle(step);
	}
}
