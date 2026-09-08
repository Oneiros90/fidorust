import type { ComponentCursor } from './engineTypes';

export type Theme = 'light' | 'dark';

export type LibGhost = ComponentCursor & { x: number; y: number; scale: number; rot: number };

export const TOOLS = [
	['select', 'select'],
	['pan', 'pan'],
	['line', 'line'],
	['rect', 'rect'],
	['ellipse', 'ellipse'],
	['poly', 'poly'],
	['bezier', 'bezier'],
	['text', 'text'],
	['connection', 'connection'],
	['pcb-track', 'pcbTrack'],
	['pcb-pad', 'pcbPad']
] as const;
