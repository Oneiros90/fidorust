import { A4_PT, LETTER_PT, PDF_MAX_PT, PT_PER_LU } from './constants';
import type { PdfPage } from './exportOptions';
import { parseSvgPrims, parseSvgViewBox, type Rgba, type SvgPrim } from './svgGeom';

function concat(parts: Uint8Array[]): Uint8Array {
	let len = 0;
	for (const p of parts) len += p.length;
	const out = new Uint8Array(len);
	let o = 0;
	for (const p of parts) {
		out.set(p, o);
		o += p.length;
	}
	return out;
}

function n(v: number): string {
	return v.toFixed(3);
}

function rgb(r: number, g: number, b: number): string {
	return `${n(r / 255)} ${n(g / 255)} ${n(b / 255)}`;
}

const KAPPA = 0.5522847498307936;

export type PdfExportOpts = {
	page?: PdfPage;
	landscape?: boolean;
	scale?: number;
};

export type PdfLayout = {
	pageW: number;
	pageH: number;
	scale: number;
	ox: number;
	oy: number;
};

export function pdfLayout(svg: string, opts: PdfExportOpts = {}): PdfLayout {
	const box = parseSvgViewBox(svg);
	const userScale = opts.scale && opts.scale > 0 ? opts.scale : 1;
	const page = opts.page ?? 'drawing';
	if (page === 'drawing') {
		const s = PT_PER_LU * userScale;
		let pageW = Math.max(1, box.w * s);
		let pageH = Math.max(1, box.h * s);
		const maxPt = PDF_MAX_PT;
		const fit = Math.min(1, maxPt / Math.max(pageW, pageH, 1));
		pageW *= fit;
		pageH *= fit;
		return { pageW, pageH, scale: s * fit, ox: 0, oy: 0 };
	}
	const paper = page === 'letter' ? LETTER_PT : A4_PT;
	const pageW = opts.landscape ? paper.h : paper.w;
	const pageH = opts.landscape ? paper.w : paper.h;
	const margin = 18;
	const innerW = Math.max(1, pageW - 2 * margin);
	const innerH = Math.max(1, pageH - 2 * margin);
	const s = Math.min(innerW / box.w, innerH / box.h) * userScale;
	const ox = margin + (innerW - box.w * s) / 2;
	const oy = margin + (innerH - box.h * s) / 2;
	return { pageW, pageH, scale: s, ox, oy };
}

function ellipseOps(
	cx: number,
	cy: number,
	rx: number,
	ry: number,
	px: (x: number) => number,
	py: (y: number) => number
): string[] {
	const ox = rx * KAPPA;
	const oy = ry * KAPPA;
	const p = (x: number, y: number) => `${n(px(x))} ${n(py(y))}`;
	return [
		`${p(cx + rx, cy)} m`,
		`${p(cx + rx, cy - oy)} ${p(cx + ox, cy - ry)} ${p(cx, cy - ry)} c`,
		`${p(cx - ox, cy - ry)} ${p(cx - rx, cy - oy)} ${p(cx - rx, cy)} c`,
		`${p(cx - rx, cy + oy)} ${p(cx - ox, cy + ry)} ${p(cx, cy + ry)} c`,
		`${p(cx + ox, cy + ry)} ${p(cx + rx, cy + oy)} ${p(cx + rx, cy)} c`
	];
}

function gs(a: number): string {
	return `/A${Math.max(0, Math.min(255, Math.round(a)))} gs`;
}

function colorAlpha(c: Rgba | null | undefined): number | null {
	return c ? c[3] : null;
}

function primAlpha(prim: SvgPrim): number {
	if (prim.kind === 'hole') return 255;
	if (prim.kind === 'line' || prim.kind === 'bezier') return prim.a;
	if (prim.kind === 'text') return prim.fill[3];
	return colorAlpha(prim.fill) ?? colorAlpha(prim.stroke) ?? 255;
}

function paintPath(
	parts: string[],
	fill: Rgba | null,
	stroke: Rgba | null,
	strokeWidth: number,
	scale: number,
	close: boolean
) {
	parts.push(gs(colorAlpha(fill) ?? colorAlpha(stroke) ?? 255));
	if (fill) parts.push(`${rgb(fill[0], fill[1], fill[2])} rg`);
	if (stroke) {
		parts.push(`${rgb(stroke[0], stroke[1], stroke[2])} RG`);
		parts.push(`${n(Math.max(strokeWidth * scale, 0.2))} w`);
	}
	if (fill && stroke) parts.push(close ? 'h B' : 'B');
	else if (fill) parts.push(close ? 'h f' : 'f');
	else if (stroke) parts.push(close ? 's' : 'S');
}

function emitPrim(
	prim: SvgPrim,
	parts: string[],
	px: (x: number) => number,
	py: (y: number) => number,
	scale: number
) {
	if (prim.kind === 'polygon') {
		if (prim.pts.length < 2) return;
		const pts = prim.pts.map(([x, y]) => [px(x), py(y)] as const);
		parts.push(`${n(pts[0][0])} ${n(pts[0][1])} m`);
		for (let i = 1; i < pts.length; i++) parts.push(`${n(pts[i][0])} ${n(pts[i][1])} l`);
		paintPath(parts, prim.fill, prim.stroke, prim.strokeWidth, scale, true);
		return;
	}
	if (prim.kind === 'line') {
		const w = Math.max(prim.width * scale, 0.2);
		parts.push(gs(prim.a));
		parts.push(`${rgb(prim.r, prim.g, prim.b)} RG`);
		parts.push(`${n(w)} w`);
		parts.push(`${n(px(prim.x1))} ${n(py(prim.y1))} m ${n(px(prim.x2))} ${n(py(prim.y2))} l S`);
		return;
	}
	if (prim.kind === 'ellipse') {
		const ops = ellipseOps(prim.cx, prim.cy, prim.rx, prim.ry, px, py);
		parts.push(...ops);
		paintPath(parts, prim.fill, prim.stroke, prim.strokeWidth, scale, true);
		return;
	}
	if (prim.kind === 'rect') {
		parts.push(...roundedRectOps(prim.x, prim.y, prim.w, prim.h, prim.rx, prim.ry, px, py));
		paintPath(parts, prim.fill, prim.stroke, prim.strokeWidth, scale, true);
		return;
	}
	if (prim.kind === 'bezier') {
		parts.push(gs(prim.a));
		parts.push(`${rgb(prim.r, prim.g, prim.b)} RG`);
		parts.push(`${n(Math.max(prim.width * scale, 0.2))} w`);
		parts.push(
			`${n(px(prim.x0))} ${n(py(prim.y0))} m`,
			`${n(px(prim.x1))} ${n(py(prim.y1))} ${n(px(prim.x2))} ${n(py(prim.y2))} ${n(px(prim.x3))} ${n(py(prim.y3))} c`,
			'S'
		);
		return;
	}
	if (prim.kind === 'text') {
		emitPdfText(parts, prim, px, py, scale);
		return;
	}
	parts.push(gs(255));
	parts.push('1 1 1 rg');
	parts.push(...ellipseOps(prim.cx, prim.cy, prim.r, prim.r, px, py), 'h f');
}

function roundedRectOps(
	x: number,
	y: number,
	w: number,
	h: number,
	rx: number,
	ry: number,
	px: (x: number) => number,
	py: (y: number) => number
): string[] {
	const rxi = Math.min(Math.max(rx, 0), w / 2);
	const ryi = Math.min(Math.max(ry, 0), h / 2);
	const sx = px(x + 1) - px(x);
	const sy = py(y) - py(y + 1);
	if (rxi < 0.001 && ryi < 0.001) {
		return [`${n(px(x))} ${n(py(y + h))} ${n(w * sx)} ${n(h * sy)} re`];
	}
	const k = KAPPA;
	const x0 = x;
	const y0 = y;
	const x1 = x + w;
	const y1 = y + h;
	const p = (xx: number, yy: number) => `${n(px(xx))} ${n(py(yy))}`;
	return [
		`${p(x0 + rxi, y0)} m`,
		`${p(x1 - rxi, y0)} l`,
		`${p(x1 - rxi + rxi * k, y0)} ${p(x1, y0 + ryi - ryi * k)} ${p(x1, y0 + ryi)} c`,
		`${p(x1, y1 - ryi)} l`,
		`${p(x1, y1 - ryi + ryi * k)} ${p(x1 - rxi + rxi * k, y1)} ${p(x1 - rxi, y1)} c`,
		`${p(x0 + rxi, y1)} l`,
		`${p(x0 + rxi - rxi * k, y1)} ${p(x0, y1 - ryi + ryi * k)} ${p(x0, y1 - ryi)} c`,
		`${p(x0, y0 + ryi)} l`,
		`${p(x0, y0 + ryi - ryi * k)} ${p(x0 + rxi - rxi * k, y0)} ${p(x0 + rxi, y0)} c`
	];
}

function pdfStr(s: string): string {
	let o = '(';
	for (const ch of s) {
		const c = ch.codePointAt(0) ?? 32;
		if (ch === '\\' || ch === '(' || ch === ')') o += '\\' + ch;
		else if (c >= 32 && c <= 126) o += ch;
		else if (c < 256) o += '\\' + c.toString(8).padStart(3, '0');
		else o += '?';
	}
	return o + ')';
}

function emitPdfText(
	parts: string[],
	prim: Extract<SvgPrim, { kind: 'text' }>,
	px: (x: number) => number,
	py: (y: number) => number,
	scale: number
) {
	const size = Math.max(prim.fontSize * scale, 0.5);
	const ascent = prim.fontSize * 0.8;
	const rad = (prim.angle * Math.PI) / 180;
	const cos = Math.cos(rad);
	const sin = Math.sin(rad);
	const e = px(prim.x - ascent * sin);
	const f = py(prim.y + ascent * cos);
	const sx = prim.mirrored ? -size : size;
	const a = sx * cos;
	const b = -sx * sin;
	const c = size * sin;
	const d = size * cos;
	const tz =
		prim.textLength > 0 && prim.content.length > 0
			? ((prim.textLength * scale) / (prim.content.length * size * 0.6)) * 100
			: 100;
	parts.push(gs(prim.fill[3]));
	parts.push(`${rgb(prim.fill[0], prim.fill[1], prim.fill[2])} rg`);
	parts.push('BT');
	parts.push('/F1 1 Tf');
	parts.push(`${n(Math.max(tz, 1))} Tz`);
	parts.push(`${n(a)} ${n(b)} ${n(c)} ${n(d)} ${n(e)} ${n(f)} Tm`);
	parts.push(`${pdfStr(prim.content)} Tj`);
	parts.push('ET');
}

function svgToContentStream(svg: string, layout: PdfLayout): { content: string; alphas: number[] } {
	const box = parseSvgViewBox(svg);
	const { pageW, pageH, scale, ox, oy } = layout;
	const px = (x: number) => ox + (x - box.x) * scale;
	const py = (y: number) => pageH - (oy + (y - box.y) * scale);
	const prims = parseSvgPrims(svg);
	const parts: string[] = [
		'1 1 1 rg',
		`0 0 ${n(pageW)} ${n(pageH)} re`,
		'f',
		'1 J',
		'1 j',
		gs(255)
	];
	for (const prim of prims) emitPrim(prim, parts, px, py, scale);
	const alphas = new Set<number>([255]);
	for (const prim of prims) alphas.add(primAlpha(prim));
	return { content: parts.join('\n') + '\n', alphas: [...alphas] };
}

function wrapPdf(pageW: number, pageH: number, content: Uint8Array, alphas: number[]): Uint8Array {
	const enc = new TextEncoder();
	const header = enc.encode('%PDF-1.4\n%\x80\x80\x80\x80\n');
	const obj1 = enc.encode('1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n');
	const obj2 = enc.encode('2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n');
	const gsDict = [...new Set(alphas)]
		.sort((a, b) => a - b)
		.map((a) => `/A${a} << /Type /ExtGState /ca ${n(a / 255)} /CA ${n(a / 255)} >>`)
		.join(' ');
	const obj3 = enc.encode(
		`3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${n(pageW)} ${n(pageH)}] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Courier >> >> /ExtGState << ${gsDict} >> >> >>\nendobj\n`
	);
	const obj4 = concat([
		enc.encode(`4 0 obj\n<< /Length ${content.length} >>\nstream\n`),
		content,
		enc.encode('\nendstream\nendobj\n')
	]);

	const chunks: Uint8Array[] = [header];
	const offsets = [0];
	let pos = header.length;
	const addObj = (data: Uint8Array) => {
		offsets.push(pos);
		chunks.push(data);
		pos += data.length;
	};
	addObj(obj1);
	addObj(obj2);
	addObj(obj3);
	addObj(obj4);
	const xrefPos = pos;
	const xrefLines = ['xref', `0 ${offsets.length}`, '0000000000 65535 f '];
	for (let i = 1; i < offsets.length; i++) {
		xrefLines.push(`${String(offsets[i]).padStart(10, '0')} 00000 n `);
	}
	const tail = enc.encode(
		`${xrefLines.join('\n')}\ntrailer\n<< /Size ${offsets.length} /Root 1 0 R >>\nstartxref\n${xrefPos}\n%%EOF\n`
	);
	return concat([...chunks, tail]);
}

export function svgToPdfBlob(svg: string, opts: PdfExportOpts = {}): Blob {
	const layout = pdfLayout(svg, opts);
	const { content, alphas } = svgToContentStream(svg, layout);
	const bytes = new TextEncoder().encode(content);
	const pdf = wrapPdf(layout.pageW, layout.pageH, bytes, alphas);
	const copy = new Uint8Array(pdf.byteLength);
	copy.set(pdf);
	return new Blob([copy.buffer], { type: 'application/pdf' });
}
