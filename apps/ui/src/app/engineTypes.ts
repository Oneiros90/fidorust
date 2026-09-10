/** Mirror of fidorust-wasm json DTOs. */

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
	hide_component_origin: boolean;
	stroke_hundredths: number;
	default_filled: boolean;
	pending_component: string | null;
	duplicate_drag: boolean;
	can_create_component: boolean;
	can_split_component: boolean;
	can_edit_component: boolean;
	editing_component: string | null;
	editing_component_name: string | null;
	editing_component_dirty: boolean;
	libs_rev: number;
};

export type Layer = {
	name: string;
	color: number[];
	show: boolean;
};

export type LayersData = { layers: Layer[] };

export type LibraryKind = 'builtin' | 'project' | 'local';

export type LibraryComponent = {
	key: string;
	name: string;
};

export type LibraryEntry = {
	stem: string;
	title: string;
	standard: boolean;
	kind: LibraryKind;
	writable: boolean;
	categories: { name: string; components: LibraryComponent[] }[];
};

export type UserLibraryInfo = {
	stem: string;
	title: string;
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

export type ComponentCursor = {
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
	hide_component_origin: true,
	stroke_hundredths: 25,
	default_filled: false,
	pending_component: null,
	duplicate_drag: false,
	can_create_component: false,
	can_split_component: false,
	can_edit_component: false,
	editing_component: null,
	editing_component_name: null,
	editing_component_dirty: false,
	libs_rev: 0
});
