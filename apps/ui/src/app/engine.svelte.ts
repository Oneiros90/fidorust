import type { App as WasmApp } from '../wasm/fidocad_wasm.js';
import { defaultStatus, type LayersData, type LibraryEntry, type Status } from './engineTypes';

export type MutateOpts = { refreshFirst?: boolean };

export class Engine {
	app: WasmApp;
	status = $state<Status>(defaultStatus());
	layers = $state<LayersData>({ layers: [] });
	libs = $state<LibraryEntry[]>([]);
	canvas: HTMLCanvasElement | null = null;

	constructor(app: WasmApp) {
		this.app = app;
		this.libs = JSON.parse(app.library_json());
		this.refresh();
	}

	refresh = () => {
		this.status = JSON.parse(this.app.status_json());
		this.layers = JSON.parse(this.app.layers_json());
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

	attachCanvas = (node: HTMLCanvasElement) => {
		this.canvas = node;
		this.app.attach_canvas(node);
		return () => {
			if (this.canvas === node) this.canvas = null;
		};
	};
}
