import type { ComponentCursor } from './engineTypes';

export type Theme = 'light' | 'dark';

export type LibGhost = ComponentCursor & { x: number; y: number; scale: number; rot: number };

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
	['pan', 'pan']
] as const;
