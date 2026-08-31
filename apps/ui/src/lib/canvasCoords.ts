export function dpr(): number {
	return Math.min(window.devicePixelRatio || 1, 2);
}

export function canvasLocal(
	canvas: HTMLCanvasElement,
	clientX: number,
	clientY: number
): { x: number; y: number; inside: boolean } {
	const r = canvas.getBoundingClientRect();
	const scale = dpr();
	return {
		x: (clientX - r.left) * scale,
		y: (clientY - r.top) * scale,
		inside: clientX >= r.left && clientX <= r.right && clientY >= r.top && clientY <= r.bottom
	};
}

export function cssPerLu(zoom: number): number {
	return zoom / dpr();
}
