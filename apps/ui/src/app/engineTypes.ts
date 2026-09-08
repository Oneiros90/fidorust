/** Mirror of fidocad-wasm json DTOs. */

export type Status = {
	tool: string;
	layer: number;
	x: number;
	y: number;
	xmm: number;
	ymm: number;
	zoom: number;
	pan_x: number;
	pan_y: number;
	pcb: boolean;
	n: number;
	selected: number;
	can_undo: boolean;
	can_redo: boolean;
	title: string;
	snap: number;
	snap_y: number;
	grid: number;
	grid_y: number;
	snap_enable: boolean;
	show_grid: boolean;
	hide_macro_origin: boolean;
	pending_macro: string | null;
};

export type Layer = {
	name: string;
	color: number[];
	show: boolean;
};

export type LayersData = { layers: Layer[] };

export type LibraryEntry = {
	stem: string;
	title: string;
	categories: { name: string; macros: [string, string][] }[];
};

export type TextEdit = {
	text: string;
	wx: number;
	wy: number;
	sx: number;
	sy: number;
	angle: number;
	style: number;
	screenX: number;
	screenY: number;
	zoom: number;
};

export type MacroCursor = {
	svg: string;
	ox: number;
	oy: number;
	w: number;
	h: number;
};

export const defaultStatus = (): Status => ({
	tool: 'select',
	layer: 0,
	x: 0,
	y: 0,
	xmm: 0,
	ymm: 0,
	zoom: 4,
	pan_x: 40,
	pan_y: 40,
	pcb: false,
	n: 0,
	selected: 0,
	can_undo: false,
	can_redo: false,
	title: '',
	snap: 5,
	snap_y: 5,
	grid: 5,
	grid_y: 5,
	snap_enable: true,
	show_grid: true,
	hide_macro_origin: true,
	pending_macro: null
});
