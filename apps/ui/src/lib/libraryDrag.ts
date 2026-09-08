import type { ComponentCursor } from '../app/engineTypes';

export { canvasLocal, cssPerLu } from './canvasCoords';
export type { ComponentCursor } from '../app/engineTypes';

export function componentFullName(stem: string, key: string): string {
	return stem === 'stdlib' ? key : `${stem}.${key}`;
}

export function parseComponentCursor(json: string): ComponentCursor | null {
	try {
		const c = JSON.parse(json) as ComponentCursor;
		if (!c?.svg || !(c.w > 0) || !(c.h > 0)) return null;
		return c;
	} catch {
		return null;
	}
}
