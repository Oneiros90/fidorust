export { canvasLocal, cssPerLu } from './canvasCoords';

export function macroFullName(stem: string, key: string): string {
	return stem === 'stdlib' ? key : `${stem}.${key}`;
}

export type MacroCursor = {
	svg: string;
	ox: number;
	oy: number;
	w: number;
	h: number;
};

export function parseMacroCursor(json: string): MacroCursor | null {
	try {
		const c = JSON.parse(json) as MacroCursor;
		if (!c?.svg || !(c.w > 0) || !(c.h > 0)) return null;
		return c;
	} catch {
		return null;
	}
}
