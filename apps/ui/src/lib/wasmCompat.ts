import type { App as WasmApp } from '../wasm/fidorust_wasm.js';

/** Packed Pro WASM can lag the UI. Map dual-pane calls onto the last single-canvas API. */
export function ensureLegacyPaneApi(app: WasmApp): WasmApp {
	const a = app as WasmApp & Record<string, unknown>;
	const fill = <K extends keyof WasmApp>(name: K, fn: WasmApp[K]) => {
		if (typeof a[name] !== 'function') a[name] = fn as never;
	};

	fill('pointer_down_on', (_pane, x, y, shift, pan) => a.pointer_down(x, y, shift, pan));
	fill('pointer_move_on', (_pane, x, y) => a.pointer_move(x, y));
	fill('pointer_up_on', (_pane, x, y) => a.pointer_up(x, y));
	fill('pointer_right_on', (_pane, x, y) => a.pointer_right(x, y));
	fill('resize_pane', (_pane, w, h) => a.resize(w, h));
	fill('attach_pane_canvas', (_pane, canvas) => a.attach_canvas(canvas));
	fill('detach_pane_canvas', () => {});
	fill('hover_pane', () => {});
	fill('fit_pane', () => a.fit());
	fill('wheel_on', (_pane, x, y, delta) => a.wheel(x, y, delta));
	fill('dblclick_on', (_pane, x, y) => a.dblclick(x, y));
	fill('world_to_screen_json_on', (_pane, wx, wy) => a.world_to_screen_json(wx, wy));
	fill('begin_marquee_on', (_pane, x, y, shift) => a.begin_marquee(x, y, shift));
	fill('prepare_context_menu_on', (_pane, x, y) => a.prepare_context_menu(x, y));
	fill('set_pane_view', (_pane, zoom, panX, panY) => a.set_view(zoom, panX, panY));
	fill('set_pane_sheet', () => {});
	fill('set_split', () => {});
	fill('add_sheet_on', () => (typeof a.add_sheet === 'function' ? a.add_sheet() : 0));
	return app;
}
