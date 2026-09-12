import { rgbaCss } from './color';
import {
	applySvgMatrix,
	identitySvgMatrix,
	mulSvgMatrix,
	parseSvgViewBox,
	type SvgBox,
	type SvgMatrix
} from './svgGeom';
import type { ExportPreviewOpts, ExportRotate } from './exportOptions';

export function flipHMatrix(cx: number): SvgMatrix {
	return [-1, 0, 0, 1, 2 * cx, 0];
}

export function flipVMatrix(cy: number): SvgMatrix {
	return [1, 0, 0, -1, 0, 2 * cy];
}

/** 90° clockwise in SVG coordinates (y down), about (cx, cy). */
export function rot90cwMatrix(cx: number, cy: number): SvgMatrix {
	return [0, 1, -1, 0, cx + cy, cy - cx];
}

export function exportMatrix(
	box: SvgBox,
	opts: { flipH: boolean; flipV: boolean; rotate: ExportRotate }
): { matrix: SvgMatrix; box: SvgBox } {
	const cx = box.x + box.w / 2;
	const cy = box.y + box.h / 2;
	let m = identitySvgMatrix();
	if (opts.flipH) m = mulSvgMatrix(flipHMatrix(cx), m);
	if (opts.flipV) m = mulSvgMatrix(flipVMatrix(cy), m);
	const turns = ((opts.rotate / 90) | 0) % 4;
	for (let i = 0; i < turns; i++) m = mulSvgMatrix(rot90cwMatrix(cx, cy), m);

	const corners: [number, number][] = [
		[box.x, box.y],
		[box.x + box.w, box.y],
		[box.x + box.w, box.y + box.h],
		[box.x, box.y + box.h]
	];
	let minX = Infinity;
	let minY = Infinity;
	let maxX = -Infinity;
	let maxY = -Infinity;
	for (const [x, y] of corners) {
		const [nx, ny] = applySvgMatrix(m, x, y);
		minX = Math.min(minX, nx);
		minY = Math.min(minY, ny);
		maxX = Math.max(maxX, nx);
		maxY = Math.max(maxY, ny);
	}
	return { matrix: m, box: { x: minX, y: minY, w: maxX - minX, h: maxY - minY } };
}

export function isIdentityExport(opts: {
	flipH: boolean;
	flipV: boolean;
	rotate: ExportRotate;
}): boolean {
	return !opts.flipH && !opts.flipV && opts.rotate === 0;
}

/** Flip the current preview left-right (after existing rotate/flips). */
export function toggleExportFlipH(opts: ExportPreviewOpts) {
	if (opts.rotate === 90 || opts.rotate === 270) opts.flipV = !opts.flipV;
	else opts.flipH = !opts.flipH;
}

/** Flip the current preview top-bottom (after existing rotate/flips). */
export function toggleExportFlipV(opts: ExportPreviewOpts) {
	if (opts.rotate === 90 || opts.rotate === 270) opts.flipH = !opts.flipH;
	else opts.flipV = !opts.flipV;
}

export function rotateExport90(opts: ExportPreviewOpts) {
	opts.rotate = ((opts.rotate + 90) % 360) as ExportRotate;
}

export function currentFlipH(opts: ExportPreviewOpts): boolean {
	return opts.rotate === 90 || opts.rotate === 270 ? opts.flipV : opts.flipH;
}

export function currentFlipV(opts: ExportPreviewOpts): boolean {
	return opts.rotate === 90 || opts.rotate === 270 ? opts.flipH : opts.flipV;
}

function fmt(n: number): string {
	return n.toFixed(2);
}

export function applyExportLayout(svg: string, opts: ExportPreviewOpts): string {
	if (!svg.includes('<svg')) return svg;
	const src = parseSvgViewBox(svg);
	const { matrix, box } = exportMatrix(src, opts);
	const identity = isIdentityExport(opts);
	if (identity && !opts.bgEnabled) return svg;
	const bg = opts.bgEnabled
		? `<rect class="export-bg" x="${fmt(box.x)}" y="${fmt(box.y)}" width="${fmt(box.w)}" height="${fmt(box.h)}" fill="${rgbaCss(opts.bgColor)}"/>`
		: '';

	return svg.replace(/<svg\b([^>]*)>([\s\S]*)<\/svg>\s*$/, (_m, attrs: string, inner: string) => {
		const cleaned = String(attrs)
			.replace(/\s*\bwidth="[^"]*"/g, '')
			.replace(/\s*\bheight="[^"]*"/g, '')
			.replace(/\s*\bviewBox="[^"]*"/g, '');
		const body = identity
			? inner
			: `<g data-export-xform="1" transform="matrix(${matrix.map((v) => Number(v.toFixed(6))).join(' ')})">${inner}</g>`;
		return `<svg${cleaned} width="${fmt(box.w)}" height="${fmt(box.h)}" viewBox="${fmt(box.x)} ${fmt(box.y)} ${fmt(box.w)} ${fmt(box.h)}">${bg}${body}</svg>`;
	});
}
