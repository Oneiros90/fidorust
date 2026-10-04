/** Mirror of fidorust-wasm json DTOs. */

export type ShapeDefaults = {
	stroke_hundredths: number;
	filled: boolean;
};

export type TextDefaults = {
	sy: number;
	sx: number;
	style: number;
	font: string;
};

export type PadDefaults = {
	dx: number;
	dy: number;
	hole: number;
	style: 'Oval' | 'Rectangular' | 'RoundedRect';
};

export type PrimitiveDefaults = {
	line_stroke_hundredths: number;
	bezier_stroke_hundredths: number;
	rect: ShapeDefaults;
	ellipse: ShapeDefaults;
	poly: ShapeDefaults;
	text: TextDefaults;
	track_width: number;
	pad: PadDefaults;
};

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
	defaults: PrimitiveDefaults;
	pending_component: string | null;
	duplicate_drag: boolean;
	hover_hit: boolean;
	hover_op: string | null;
	can_create_component: boolean;
	can_split_component: boolean;
	can_edit_component: boolean;
	can_common_node: boolean;
	common_node_on: boolean;
	common_node_names: string[];
	common_node_count: number;
	editing_component: string | null;
	editing_component_name: string | null;
	editing_component_dirty: boolean;
	libs_rev: number;
	split: boolean;
	active_pane: number;
	pane_sheets: [number, number];
	pane_zoom: [number, number];
	pane_pan_x: [number, number];
	pane_pan_y: [number, number];
	sheets: string[];
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
	defaults: {
		line_stroke_hundredths: 25,
		bezier_stroke_hundredths: 25,
		rect: { stroke_hundredths: 25, filled: false },
		ellipse: { stroke_hundredths: 25, filled: false },
		poly: { stroke_hundredths: 25, filled: false },
		text: { sy: 4, sx: 3, style: 0, font: 'Courier Prime' },
		track_width: 4,
		pad: { dx: 18, dy: 18, hole: 8, style: 'Oval' }
	},
	pending_component: null,
	duplicate_drag: false,
	hover_hit: false,
	hover_op: null,
	can_create_component: false,
	can_split_component: false,
	can_edit_component: false,
	can_common_node: false,
	common_node_on: false,
	common_node_names: [],
	common_node_count: 0,
	editing_component: null,
	editing_component_name: null,
	editing_component_dirty: false,
	libs_rev: 0,
	split: false,
	active_pane: 0,
	pane_sheets: [0, 0],
	pane_zoom: [4, 4],
	pane_pan_x: [40, 40],
	pane_pan_y: [40, 40],
	sheets: ['Foglio 1']
});
