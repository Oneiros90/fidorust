export type Theme = 'light' | 'dark';

export type Status = {
	tool: string;
	layer: number;
	x: number;
	y: number;
	xmm: number;
	ymm: number;
	zoom: number;
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
	print: boolean;
};

export type LayersData = { layers: Layer[] };

export type LibraryEntry = {
	stem: string;
	title: string;
	categories: { name: string; macros: [string, string][] }[];
};

export const defaultStatus = (): Status => ({
	tool: 'select',
	layer: 0,
	x: 0,
	y: 0,
	xmm: 0,
	ymm: 0,
	zoom: 4,
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

export const TOOLS = [
	['select', 'select'],
	['line', 'line'],
	['rect', 'rect'],
	['ellipse', 'ellipse'],
	['poly', 'poly'],
	['bezier', 'bezier'],
	['text', 'text'],
	['connection', 'connection'],
	['pcb-track', 'pcbTrack'],
	['pcb-pad', 'pcbPad'],
	['macro', 'macro'],
	['pan', 'pan']
] as const;
