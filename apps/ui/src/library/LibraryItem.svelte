<script lang="ts">
	import type { Engine } from '../app/engine.svelte';
	import { parseSvgElement } from '../lib/attachSvg';
	import { PREVIEW_ROOT_MARGIN, THUMB_SIZE } from '../lib/constants';
	import { macroFullName } from '../lib/libraryDrag';

	let {
		engine,
		stem,
		macroKey,
		label,
		selected,
		theme,
		onPick,
		onArmDrag
	}: {
		engine: Engine | null;
		stem: string;
		macroKey: string;
		label: string;
		selected: boolean;
		theme: 'light' | 'dark';
		onPick: (stem: string, key: string) => void;
		onArmDrag: (name: string, e: PointerEvent) => void;
	} = $props();

	const name = $derived(macroFullName(stem, macroKey));

	const attachPreview = $derived.by(() => {
		const eng = engine;
		const n = name;
		const themeKey = theme;
		return (node: HTMLElement) => {
			if (!eng) return;
			node.dataset.theme = themeKey;
			const io = new IntersectionObserver(
				(entries) => {
					if (!entries.some((e) => e.isIntersecting)) return;
					const raw = eng.query((app) => app.macro_preview_svg(n));
					const el = raw ? parseSvgElement(raw) : null;
					if (el) node.replaceChildren(el);
					io.disconnect();
				},
				{ rootMargin: PREVIEW_ROOT_MARGIN }
			);
			io.observe(node);
			return () => io.disconnect();
		};
	});

	function pick() {
		onPick(stem, macroKey);
	}

	function onPointerDown(e: PointerEvent) {
		if (e.button !== 0) return;
		pick();
		onArmDrag(name, e);
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			pick();
		}
	}
</script>

<div
	class={['mac', { selected }]}
	role="button"
	tabindex="0"
	aria-pressed={selected}
	title={label}
	onclick={pick}
	onkeydown={onKey}
	onpointerdown={onPointerDown}
	style:--thumb-size="{THUMB_SIZE}px"
>
	<div class="thumb" {@attach attachPreview}></div>
	<span class="meta">
		<span class="key">{macroKey}</span>
		<span class="label">{label}</span>
	</span>
</div>

<style>
	.mac {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		text-align: left;
		border: 1px solid transparent;
		background: transparent;
		padding: 3px 6px;
		border-radius: 6px;
		font: inherit;
		font-size: 12px;
		line-height: 1.3;
		color: inherit;
		cursor: grab;
		user-select: none;
	}
	.mac:hover {
		background: var(--bg-menu);
	}
	.mac.selected {
		background: color-mix(in srgb, var(--accent) 22%, var(--bg-panel));
		border-color: var(--accent);
		box-shadow: inset 3px 0 0 var(--accent);
	}
	.mac:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: -2px;
	}
	.mac:active {
		cursor: grabbing;
	}
	.thumb {
		width: var(--thumb-size);
		height: var(--thumb-size);
		flex-shrink: 0;
		border-radius: 4px;
		background: var(--canvas-bg);
		border: 1px solid var(--border);
		overflow: hidden;
		display: grid;
		place-items: center;
		pointer-events: none;
	}
	.thumb :global(svg) {
		display: block;
		width: var(--thumb-size);
		height: var(--thumb-size);
	}
	.meta {
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.key {
		font-family: var(--mono);
		font-size: 11px;
		color: var(--fg-muted);
	}
	.label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
