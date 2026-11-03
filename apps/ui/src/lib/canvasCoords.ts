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

/** Canvas under the pointer, or the nearest one when the pointer is on the splitter. */
export function paneFromClient(
	clientX: number,
	clientY: number
): { pane: number; sx: number; sy: number } | null {
	const nodes = document.querySelectorAll<HTMLCanvasElement>('canvas[data-pane]');
	if (nodes.length === 0) return null;
	let inside: HTMLCanvasElement | null = null;
	let nearest: HTMLCanvasElement | null = null;
	let nearestDist = Infinity;
	for (const node of Array.from(nodes)) {
		const r = node.getBoundingClientRect();
		if (clientX >= r.left && clientX <= r.right && clientY >= r.top && clientY <= r.bottom) {
			inside = node;
			break;
		}
		const dx = clientX < r.left ? r.left - clientX : clientX > r.right ? clientX - r.right : 0;
		const dy = clientY < r.top ? r.top - clientY : clientY > r.bottom ? clientY - r.bottom : 0;
		const dist = dx + dy;
		if (dist < nearestDist) {
			nearestDist = dist;
			nearest = node;
		}
	}
	const node = inside ?? nearest;
	if (!node) return null;
	const local = canvasLocal(node, clientX, clientY);
	return { pane: Number(node.dataset.pane ?? 0), sx: local.x, sy: local.y };
}

export function cssPerLu(zoom: number): number {
	return zoom / dpr();
}
