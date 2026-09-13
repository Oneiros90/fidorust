import { untrack } from 'svelte';
import type { App as WasmApp } from '../wasm/fidorust_wasm.js';
import { defaultStatus, type LayersData, type LibraryEntry, type Status } from './engineTypes';

export type MutateOpts = { refreshFirst?: boolean };

export class Engine {
	app: WasmApp;
	status = $state<Status>(defaultStatus());
	layers = $state<LayersData>({ layers: [] });
	libs = $state<LibraryEntry[]>([]);
	libsRev = $state(0);
	canvas: HTMLCanvasElement | null = null;
	onRefresh: (() => void) | null = null;

	constructor(app: WasmApp) {
		this.app = app;
		this.libs = JSON.parse(app.library_json());
		this.libsRev = JSON.parse(app.status_json()).libs_rev ?? 0;
		this.refresh();
	}

	refresh = () => {
		untrack(() => {
			const status: Status = JSON.parse(this.app.status_json());
			this.status = status;
			this.layers = JSON.parse(this.app.layers_json());
			if (status.libs_rev !== this.libsRev) {
				this.libsRev = status.libs_rev;
				this.libs = JSON.parse(this.app.library_json());
			}
			this.onRefresh?.();
		});
	};

	mutate = (fn: (app: WasmApp) => void, opts?: MutateOpts) => {
		if (opts?.refreshFirst) {
			fn(this.app);
			this.refresh();
			this.app.render();
			return;
		}
		fn(this.app);
		this.app.render();
		this.refresh();
	};

	query = <T>(fn: (app: WasmApp) => T): T => fn(this.app);

	extCommand = (name: string, payload: string): string => {
		const app = this.app as WasmApp & { ext_command: (n: string, p: string) => string };
		return app.ext_command(name, payload);
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

	attachCanvas = (node: HTMLCanvasElement) => {
		this.canvas = node;
		this.app.attach_canvas(node);
		return () => {
			if (this.canvas === node) this.canvas = null;
		};
	};
}
