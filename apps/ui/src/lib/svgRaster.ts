import { LU_PER_INCH, PNG_MAX_EDGE } from './constants';
import { parseSvgViewBox, withSvgPixelSize } from './svgGeom';

export type RasterOpts = {
	ppi: number;
	whiteBg: boolean;
	antiAlias: boolean;
	maxEdge?: number;
};

export function pngPixelSize(
	svg: string,
	ppi: number,
	maxEdge = PNG_MAX_EDGE
): { w: number; h: number; clipped: boolean } {
	const box = parseSvgViewBox(svg);
	const scale = ppi / LU_PER_INCH;
	let w = Math.max(1, Math.round(box.w * scale));
	let h = Math.max(1, Math.round(box.h * scale));
	const clipped = Math.max(w, h) > maxEdge;
	if (clipped) {
		const s = maxEdge / Math.max(w, h);
		w = Math.max(1, Math.round(w * s));
		h = Math.max(1, Math.round(h * s));
	}
	return { w, h, clipped };
}

export function rasterizeSvg(svg: string, opts: RasterOpts): Promise<HTMLCanvasElement> {
	const { w, h } = pngPixelSize(svg, opts.ppi, opts.maxEdge ?? PNG_MAX_EDGE);
	const sized = withSvgPixelSize(svg, w, h);
	const blob = new Blob([sized], { type: 'image/svg+xml;charset=utf-8' });
	const url = URL.createObjectURL(blob);
	return new Promise((resolve, reject) => {
		const img = new Image();
		img.onload = () => {
			URL.revokeObjectURL(url);
			const canvas = document.createElement('canvas');
			canvas.width = w;
			canvas.height = h;
			const ctx = canvas.getContext('2d');
			if (!ctx) {
				reject(new Error('No 2D context'));
				return;
			}
			ctx.imageSmoothingEnabled = opts.antiAlias;
			if (opts.antiAlias) ctx.imageSmoothingQuality = 'high';
			if (opts.whiteBg) {
				ctx.fillStyle = '#ffffff';
				ctx.fillRect(0, 0, w, h);
			}
			ctx.drawImage(img, 0, 0, w, h);
			resolve(canvas);
		};
		img.onerror = () => {
			URL.revokeObjectURL(url);
			reject(new Error('Failed to rasterize SVG'));
		};
		img.src = url;
	});
}

export function canvasToPngBlob(canvas: HTMLCanvasElement): Promise<Blob> {
	return new Promise((resolve, reject) => {
		canvas.toBlob((b) => {
			if (!b) reject(new Error('PNG encode failed'));
			else resolve(b);
		}, 'image/png');
	});
}
