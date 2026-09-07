import type { MacroCursor } from '../app/engineTypes';

export { canvasLocal, cssPerLu } from './canvasCoords';
export type { MacroCursor } from '../app/engineTypes';

export function macroFullName(stem: string, key: string): string {
	return stem === 'stdlib' ? key : `${stem}.${key}`;
}

export function parseMacroCursor(json: string): MacroCursor | null {
	try {
		const c = JSON.parse(json) as MacroCursor;
		if (!c?.svg || !(c.w > 0) || !(c.h > 0)) return null;
		return c;
	} catch {
		return null;
	}
}
