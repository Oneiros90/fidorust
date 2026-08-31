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

export function canvasLocal(
	canvas: HTMLCanvasElement,
	clientX: number,
	clientY: number
): { x: number; y: number; inside: boolean } {
	const r = canvas.getBoundingClientRect();
	const dpr = Math.min(window.devicePixelRatio || 1, 2);
	return {
		x: (clientX - r.left) * dpr,
		y: (clientY - r.top) * dpr,
		inside: clientX >= r.left && clientX <= r.right && clientY >= r.top && clientY <= r.bottom
	};
}

export function cssPerLu(zoom: number): number {
	const dpr = Math.min(window.devicePixelRatio || 1, 2);
	return zoom / dpr;
}
