import { LU_PER_INCH, clampPngPpi } from './constants';
import { parseSvgViewBox, withSvgPixelSize } from './svgGeom';

export type RasterOpts = {
	ppi: number;
	antiAlias: boolean;
};

export function pngPixelSize(svg: string, ppi: number): { w: number; h: number } {
	const box = parseSvgViewBox(svg);
	const scale = clampPngPpi(ppi) / LU_PER_INCH;
	return {
		w: Math.max(1, Math.round(box.w * scale)),
		h: Math.max(1, Math.round(box.h * scale))
	};
}

/** SVG-as-image is 1:1, so canvas imageSmoothing never runs; disable AA in the SVG itself. */
function withSvgCrispEdges(svg: string): string {
	return svg.replace(/<svg\b([^>]*)>/, (_m, attrs: string) => {
		const cleaned = String(attrs)
			.replace(/\s*\bshape-rendering="[^"]*"/g, '')
			.replace(/\s*\btext-rendering="[^"]*"/g, '');
		return `<svg${cleaned} shape-rendering="crispEdges" text-rendering="optimizeSpeed">`;
	});
}

export function rasterizeSvg(svg: string, opts: RasterOpts): Promise<HTMLCanvasElement> {
	const { w, h } = pngPixelSize(svg, opts.ppi);
	const sized = withSvgPixelSize(svg, w, h);
	const raster = opts.antiAlias ? sized : withSvgCrispEdges(sized);
	// Data URIs load `@font-face` inside SVG-as-image; blob: URLs often do not.
	const url = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(raster)}`;
	return new Promise((resolve, reject) => {
		const img = new Image();
		img.onload = () => {
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
			ctx.drawImage(img, 0, 0, w, h);
			resolve(canvas);
		};
		img.onerror = () => {
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
