import { untrack } from 'svelte';
import type { App as WasmApp } from '../wasm/fidorust_wasm.js';
import { defaultStatus, type LayersData, type LibraryEntry, type Status } from './engineTypes';

export type MutateOpts = { refreshFirst?: boolean };

function extCommandDirtiesScene(name: string, _out: string): boolean {
	if (name === 'ping' || name === 'circuit.status' || name === 'circuit.sync') return false;
	return name.startsWith('circuit.');
}

export class Engine {
	app: WasmApp;
	status = $state<Status>(defaultStatus());
	layers = $state<LayersData>({ layers: [] });
	libs = $state<LibraryEntry[]>([]);
	libsRev = $state(0);
	canvas: HTMLCanvasElement | null = null;
	canvases: [HTMLCanvasElement | null, HTMLCanvasElement | null] = [null, null];
	refreshListeners = new Set<() => void>();
	onRefresh: (() => void) | null = null;

	constructor(app: WasmApp) {
		this.app = app;
		this.libs = JSON.parse(app.library_json());
		this.libsRev = JSON.parse(app.status_json()).libs_rev ?? 0;
		this.refresh();
	}

	refresh = () => {
		untrack(() => {
			const parsed = JSON.parse(this.app.status_json()) as Partial<Status>;
			const status: Status = { ...defaultStatus(), ...parsed };
			if (!Array.isArray(status.sheets) || status.sheets.length === 0) {
				status.sheets = defaultStatus().sheets;
			}
			this.status = status;
			this.layers = JSON.parse(this.app.layers_json());
			if (status.libs_rev !== this.libsRev) {
				this.libsRev = status.libs_rev;
				this.libs = JSON.parse(this.app.library_json());
			}
			this.onRefresh?.();
			for (const cb of [...this.refreshListeners]) cb();
		});
	};

	subscribeRefresh = (cb: () => void) => {
		this.refreshListeners.add(cb);
		return () => {
			this.refreshListeners.delete(cb);
		};
	};

	mutate = (fn: (app: WasmApp) => void, _opts?: MutateOpts) => {
		fn(this.app);
		this.refresh();
		this.app.render();
	};

	query = <T>(fn: (app: WasmApp) => T): T => fn(this.app);

	extCommand = (name: string, payload: string): string => {
		const app = this.app as WasmApp & { ext_command: (n: string, p: string) => string };
		const out = app.ext_command(name, payload);
		if (extCommandDirtiesScene(name, out)) this.app.render();
		return out;
	};

	capabilities = (): { pro: boolean; commands: string[] } => {
		const app = this.app as WasmApp & { capabilities_json?: () => string };
		try {
			const raw = app.capabilities_json?.() ?? '{"pro":false,"commands":[]}';
			const parsed = JSON.parse(raw) as { pro?: boolean; commands?: string[] };
			return { pro: !!parsed.pro, commands: parsed.commands ?? [] };
		} catch {
			return { pro: false, commands: [] };
		}
	};

	attachCanvas = (node: HTMLCanvasElement) => this.attachPaneCanvas(0, node);

	attachPaneCanvas = (pane: number, node: HTMLCanvasElement) => {
		const i = pane === 1 ? 1 : 0;
		this.canvases[i] = node;
		if (i === 0) this.canvas = node;
		this.app.attach_pane_canvas(i, node);
		return () => {
			this.app.detach_pane_canvas(i);
			if (this.canvases[i] === node) this.canvases[i] = null;
			if (this.canvas === node) this.canvas = this.canvases[0] ?? this.canvases[1];
		};
	};
}
