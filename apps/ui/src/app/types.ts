import type { ComponentCursor } from './engineTypes';

export type Theme = 'light' | 'dark';

export type LibGhost = ComponentCursor & { x: number; y: number; scale: number; rot: number };

export const TOOL_GROUPS = [
	[
		['select', 'select'],
		['pan', 'pan'],
		['ruler', 'ruler']
	],
	[
		['connection', 'connection'],
		['line', 'line'],
		['rect', 'rect'],
		['ellipse', 'ellipse'],
		['poly', 'poly'],
		['bezier', 'bezier'],
		['text', 'text']
	],
	[
		['pcb-track', 'pcbTrack'],
		['pcb-pad', 'pcbPad']
	]
] as const;
