/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module './wasm/fidocad_wasm.js' {
	export default function init(module?: unknown): Promise<unknown>;
	export class App {
		constructor();
		attach_canvas(canvas: HTMLCanvasElement): void;
		resize(w: number, h: number): void;
		render(): void;
		load_fcd_bytes(bytes: Uint8Array): void;
		save_fcd(): string;
		clipboard_fcd(): string;
		export_svg(): string;
		pointer_down(sx: number, sy: number, shift: boolean, pan: boolean): void;
		pointer_move(sx: number, sy: number): void;
		pointer_up(sx: number, sy: number): void;
		pointer_right(sx: number, sy: number): boolean;
		prepare_context_menu(sx: number, sy: number): void;
		invert_selection(): void;
		split_selected_macros(): void;
		paste_selection(text: string): void;
		load_fcd(text: string): void;
		dblclick(sx: number, sy: number): string;
		begin_selected_text_edit(): string;
		commit_text_edit(text: string): void;
		cancel_text_edit(): void;
		world_to_screen_json(wx: number, wy: number): string;
		wheel(sx: number, sy: number, delta: number): void;
		key(key: string, meta: boolean): boolean;
		set_tool(id: string): void;
		set_layer(n: number): void;
		set_pcb_mode(on: boolean): void;
		set_grid(x: number, y: number): void;
		set_snap(x: number, y: number): void;
		set_snap_enable(on: boolean): void;
		set_hide_macro_origin(on: boolean): void;
		set_show_grid(on: boolean): void;
		set_filled(on: boolean): void;
		set_track_width(w: number): void;
		set_split_macros(on: boolean): void;
		set_pending_macro(name: string): void;
		place_macro_at(name: string, sx: number, sy: number): void;
		clear_hover(): void;
		macro_preview_svg(name: string): string;
		macro_cursor_json(name: string): string;
		set_pending_text(text: string): void;
		undo(): void;
		redo(): void;
		rotate(): void;
		mirror(): void;
		fit(): void;
		new_doc(): void;
		set_locale(loc: string): void;
		set_theme(theme: string): void;
		set_layer_show(n: number, show: boolean): void;
		set_layer_print(n: number, print: boolean): void;
		set_layer_name(n: number, name: string): void;
		set_layer_color(n: number, r: number, g: number, b: number): void;
		status_json(): string;
		library_json(): string;
		layers_json(): string;
		selection_props_json(): string;
	}
}

declare module '../wasm/fidocad_wasm.js' {
  export default function init(module?: unknown): Promise<unknown>;
  export class App {
    constructor();
    attach_canvas(canvas: HTMLCanvasElement): void;
    resize(w: number, h: number): void;
    render(): void;
    load_fcd_bytes(bytes: Uint8Array): void;
    save_fcd(): string;
    clipboard_fcd(): string;
    export_svg(): string;
    pointer_down(sx: number, sy: number, shift: boolean, pan: boolean): void;
    pointer_move(sx: number, sy: number): void;
    pointer_up(sx: number, sy: number): void;
    pointer_right(sx: number, sy: number): boolean;
    prepare_context_menu(sx: number, sy: number): void;
    invert_selection(): void;
    split_selected_macros(): void;
    paste_selection(text: string): void;
    load_fcd(text: string): void;
    dblclick(sx: number, sy: number): string;
    begin_selected_text_edit(): string;
    commit_text_edit(text: string): void;
    cancel_text_edit(): void;
    world_to_screen_json(wx: number, wy: number): string;
    wheel(sx: number, sy: number, delta: number): void;
    key(key: string, meta: boolean): boolean;
    set_tool(id: string): void;
    set_layer(n: number): void;
    set_pcb_mode(on: boolean): void;
    set_grid(x: number, y: number): void;
    set_snap(x: number, y: number): void;
    set_snap_enable(on: boolean): void;
    set_hide_macro_origin(on: boolean): void;
    set_show_grid(on: boolean): void;
    set_filled(on: boolean): void;
    set_track_width(w: number): void;
    set_split_macros(on: boolean): void;
    set_pending_macro(name: string): void;
    place_macro_at(name: string, sx: number, sy: number): void;
    clear_hover(): void;
    macro_preview_svg(name: string): string;
    macro_cursor_json(name: string): string;
    set_pending_text(text: string): void;
    undo(): void;
    redo(): void;
    rotate(): void;
    mirror(): void;
    fit(): void;
    new_doc(): void;
    set_locale(loc: string): void;
    set_theme(theme: string): void;
    set_layer_show(n: number, show: boolean): void;
    set_layer_print(n: number, print: boolean): void;
    set_layer_name(n: number, name: string): void;
    set_layer_color(n: number, r: number, g: number, b: number): void;
    status_json(): string;
    library_json(): string;
    layers_json(): string;
    selection_props_json(): string;
  }
}
