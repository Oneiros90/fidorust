export type SvgBox = { x: number; y: number; w: number; h: number };

export type Rgba = [number, number, number, number];

export type SvgMatrix = [number, number, number, number, number, number];

export function identitySvgMatrix(): SvgMatrix {
	return [1, 0, 0, 1, 0, 0];
}

export function mulSvgMatrix(a: SvgMatrix, b: SvgMatrix): SvgMatrix {
	return [
		a[0] * b[0] + a[2] * b[1],
		a[1] * b[0] + a[3] * b[1],
		a[0] * b[2] + a[2] * b[3],
		a[1] * b[2] + a[3] * b[3],
		a[0] * b[4] + a[2] * b[5] + a[4],
		a[1] * b[4] + a[3] * b[5] + a[5]
	];
}

export function applySvgMatrix(m: SvgMatrix, x: number, y: number): [number, number] {
	return [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];
}

export function parseExportMatrix(svg: string): SvgMatrix | null {
	const found = svg.match(/<g data-export-xform="1" transform="matrix\(([^)]+)\)">/);
	if (!found) return null;
	const n = found[1].split(/[\s,]+/).map(Number);
	if (n.length !== 6 || n.some((v) => !Number.isFinite(v))) return null;
	return n as SvgMatrix;
}

export type SvgPrim =
	| {
			kind: 'polygon';
			pts: [number, number][];
			fill: Rgba | null;
			stroke: Rgba | null;
			strokeWidth: number;
	  }
	| {
			kind: 'line';
			x1: number;
			y1: number;
			x2: number;
			y2: number;
			width: number;
			r: number;
			g: number;
			b: number;
			a: number;
	  }
	| {
			kind: 'ellipse';
			cx: number;
			cy: number;
			rx: number;
			ry: number;
			fill: Rgba | null;
			stroke: Rgba | null;
			strokeWidth: number;
	  }
	| {
			kind: 'rect';
			x: number;
			y: number;
			w: number;
			h: number;
			rx: number;
			ry: number;
			fill: Rgba | null;
			stroke: Rgba | null;
			strokeWidth: number;
	  }
	| {
			kind: 'bezier';
			x0: number;
			y0: number;
			x1: number;
			y1: number;
			x2: number;
			y2: number;
			x3: number;
			y3: number;
			width: number;
			r: number;
			g: number;
			b: number;
			a: number;
	  }
	| {
			kind: 'text';
			x: number;
			y: number;
			content: string;
			fontSize: number;
			fontFamily: string;
			fill: Rgba;
			italic: boolean;
			bold: boolean;
			angle: number;
			mirrored: boolean;
			textLength: number;
	  }
	| { kind: 'hole'; cx: number; cy: number; r: number };

export function parseSvgSize(svg: string): { w: number; h: number } {
	const box = parseSvgViewBox(svg);
	return { w: box.w, h: box.h };
}

export function parseSvgViewBox(svg: string): SvgBox {
	const vb = svg.match(/viewBox="([-\d.]+) ([-\d.]+) ([\d.]+) ([\d.]+)"/);
	if (vb) {
		const x = Number(vb[1]);
		const y = Number(vb[2]);
		const w = Number(vb[3]);
		const h = Number(vb[4]);
		if ([x, y, w, h].every(Number.isFinite) && w > 0 && h > 0) return { x, y, w, h };
	}
	const m = svg.match(/<svg[^>]*\bwidth="([\d.]+)"[^>]*\bheight="([\d.]+)"/);
	if (!m) throw new Error('Invalid SVG');
	const w = Number(m[1]);
	const h = Number(m[2]);
	if (!Number.isFinite(w) || !Number.isFinite(h) || w <= 0 || h <= 0)
		throw new Error('Invalid SVG size');
	return { x: 0, y: 0, w, h };
}

export function withSvgPixelSize(svg: string, w: number, h: number): string {
	return withSvgSizeAttrs(svg, String(w), String(h));
}

export function withSvgSizeAttrs(svg: string, w: string, h: string): string {
	return svg.replace(/<svg([^>]*)>/, (_m, attrs: string) => {
		const cleaned = attrs.replace(/\s*\bwidth="[^"]*"/g, '').replace(/\s*\bheight="[^"]*"/g, '');
		return `<svg${cleaned} width="${w}" height="${h}">`;
	});
}

function attr(tag: string, name: string): string | undefined {
	return tag.match(new RegExp(`\\b${name}="([^"]*)"`))?.[1];
}

function decodeXml(s: string): string {
	return s
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/&apos;/g, "'")
		.replace(/&amp;/g, '&');
}

/** First family from a CSS `font-family` list (`"Courier Prime",monospace`). */
export function parseCssFontFamily(raw: string | undefined): string {
	if (!raw) return 'Courier Prime';
	const decoded = decodeXml(raw).trim();
	const first = decoded.split(',')[0]?.trim() ?? '';
	const unquoted = first.replace(/^["']|["']$/g, '').trim();
	return unquoted || 'Courier Prime';
}

function rgbAttr(tag: string, name: string): Rgba | null {
	const v = attr(tag, name);
	if (!v || v === 'none') return null;
	const rgba = v.match(/rgba\((\d+),(\d+),(\d+),([0-9.]+)\)/);
	if (rgba) {
		const a = Math.round(Number(rgba[4]) * 255);
		return [+rgba[1], +rgba[2], +rgba[3], Number.isFinite(a) ? Math.max(0, Math.min(255, a)) : 255];
	}
	const m = v.match(/rgb\((\d+),(\d+),(\d+)\)/);
	if (!m) return null;
	return [+m[1], +m[2], +m[3], 255];
}

export function parseSvgPrims(svg: string): SvgPrim[] {
	const body = svg.replace(/<defs>[\s\S]*?<\/defs>/g, '');
	const out: SvgPrim[] = [];
	const re = /<(polygon|line|ellipse|circle|rect|path)\s([^>]*?)\/>/g;
	for (const m of body.matchAll(re)) {
		const name = m[1];
		const tag = m[2];
		if (name === 'polygon') {
			const fill = rgbAttr(tag, 'fill');
			const stroke = rgbAttr(tag, 'stroke');
			const ptsRaw = attr(tag, 'points');
			if (!ptsRaw) continue;
			const pts = ptsRaw
				.trim()
				.split(/\s+/)
				.map((p) => {
					const [x, y] = p.split(',').map(Number);
					return [x, y] as [number, number];
				})
				.filter((p) => p.every(Number.isFinite));
			const strokeWidth = Number(attr(tag, 'stroke-width') ?? '0');
			if (pts.length >= 2 && (fill || stroke)) {
				out.push({
					kind: 'polygon',
					pts,
					fill,
					stroke,
					strokeWidth: Number.isFinite(strokeWidth) ? strokeWidth : 0
				});
			}
			continue;
		}
		if (name === 'line') {
			const stroke = rgbAttr(tag, 'stroke');
			const x1 = Number(attr(tag, 'x1'));
			const y1 = Number(attr(tag, 'y1'));
			const x2 = Number(attr(tag, 'x2'));
			const y2 = Number(attr(tag, 'y2'));
			const width = Number(attr(tag, 'stroke-width'));
			if (!stroke || ![x1, y1, x2, y2, width].every(Number.isFinite)) continue;
			out.push({
				kind: 'line',
				x1,
				y1,
				x2,
				y2,
				width,
				r: stroke[0],
				g: stroke[1],
				b: stroke[2],
				a: stroke[3]
			});
			continue;
		}
		if (name === 'ellipse') {
			pushEllipse(out, tag);
			continue;
		}
		if (name === 'rect') {
			if (/\bclass="export-bg"/.test(tag)) continue;
			const x = Number(attr(tag, 'x'));
			const y = Number(attr(tag, 'y'));
			const w = Number(attr(tag, 'width'));
			const h = Number(attr(tag, 'height'));
			if (![x, y, w, h].every(Number.isFinite)) continue;
			const rx = Number(attr(tag, 'rx') ?? '0');
			const ry = Number(attr(tag, 'ry') ?? '0');
			const strokeWidth = Number(attr(tag, 'stroke-width') ?? '0');
			out.push({
				kind: 'rect',
				x,
				y,
				w,
				h,
				rx: Number.isFinite(rx) ? rx : 0,
				ry: Number.isFinite(ry) ? ry : 0,
				fill: rgbAttr(tag, 'fill'),
				stroke: rgbAttr(tag, 'stroke'),
				strokeWidth: Number.isFinite(strokeWidth) ? strokeWidth : 0
			});
			continue;
		}
		if (name === 'path') {
			const d = attr(tag, 'd') ?? '';
			const cubic = d.match(
				/M\s*([-\d.]+)[,\s]+([-\d.]+)\s*C\s*([-\d.]+)[,\s]+([-\d.]+)[,\s]+([-\d.]+)[,\s]+([-\d.]+)[,\s]+([-\d.]+)[,\s]+([-\d.]+)/i
			);
			const stroke = rgbAttr(tag, 'stroke');
			const width = Number(attr(tag, 'stroke-width'));
			if (!cubic || !stroke || !Number.isFinite(width)) continue;
			const nums = cubic.slice(1).map(Number);
			if (!nums.every(Number.isFinite)) continue;
			out.push({
				kind: 'bezier',
				x0: nums[0],
				y0: nums[1],
				x1: nums[2],
				y1: nums[3],
				x2: nums[4],
				y2: nums[5],
				x3: nums[6],
				y3: nums[7],
				width,
				r: stroke[0],
				g: stroke[1],
				b: stroke[2],
				a: stroke[3]
			});
			continue;
		}
		if (name === 'circle' && /\bclass="pad-hole"/.test(tag)) {
			const cx = Number(attr(tag, 'cx'));
			const cy = Number(attr(tag, 'cy'));
			const r = Number(attr(tag, 'r'));
			if ([cx, cy, r].every(Number.isFinite)) out.push({ kind: 'hole', cx, cy, r });
			continue;
		}
		if (name === 'circle') {
			const r = Number(attr(tag, 'r'));
			const fill = rgbAttr(tag, 'fill');
			const cx = Number(attr(tag, 'cx'));
			const cy = Number(attr(tag, 'cy'));
			if (!fill || ![cx, cy, r].every(Number.isFinite)) continue;
			out.push({
				kind: 'ellipse',
				cx,
				cy,
				rx: r,
				ry: r,
				fill,
				stroke: rgbAttr(tag, 'stroke'),
				strokeWidth: Number(attr(tag, 'stroke-width') ?? '0') || 0
			});
		}
	}
	const textRe = /<text\s([^>]*?)>([^<]*)<\/text>/g;
	for (const m of body.matchAll(textRe)) {
		const tag = m[1];
		const fill = rgbAttr(tag, 'fill');
		if (!fill) continue;
		const tr = attr(tag, 'transform') ?? '';
		const trn = tr.match(/translate\(([-\d.]+),([-\d.]+)\)/);
		const rot = tr.match(/rotate\(([-\d.]+)/);
		const scl = tr.match(/scale\(([-\d.]+)/);
		const ax = Number(attr(tag, 'x') ?? '0');
		const ay = Number(attr(tag, 'y') ?? '0');
		const x = trn ? Number(trn[1]) : ax;
		const y = trn ? Number(trn[2]) : ay;
		const fontSize = Number(attr(tag, 'font-size') ?? '10');
		const textLength = Number(attr(tag, 'textLength') ?? '0');
		if (![x, y, fontSize].every(Number.isFinite)) continue;
		out.push({
			kind: 'text',
			x,
			y,
			content: decodeXml(m[2]),
			fontSize,
			fontFamily: parseCssFontFamily(attr(tag, 'font-family')),
			fill,
			italic: attr(tag, 'font-style') === 'italic',
			bold: attr(tag, 'font-weight') === 'bold',
			angle: rot ? Number(rot[1]) : 0,
			mirrored: scl ? Number(scl[1]) < 0 : false,
			textLength: Number.isFinite(textLength) ? textLength : 0
		});
	}
	const xform = parseExportMatrix(svg);
	return xform ? out.map((prim) => transformPrim(prim, xform)) : out;
}

function applyLin(m: SvgMatrix, x: number, y: number): [number, number] {
	return [m[0] * x + m[2] * y, m[1] * x + m[3] * y];
}

function transformPrim(prim: SvgPrim, m: SvgMatrix): SvgPrim {
	const pt = (x: number, y: number) => applySvgMatrix(m, x, y);
	if (prim.kind === 'polygon') {
		return { ...prim, pts: prim.pts.map(([x, y]) => pt(x, y)) };
	}
	if (prim.kind === 'line') {
		const [x1, y1] = pt(prim.x1, prim.y1);
		const [x2, y2] = pt(prim.x2, prim.y2);
		return { ...prim, x1, y1, x2, y2 };
	}
	if (prim.kind === 'bezier') {
		const [x0, y0] = pt(prim.x0, prim.y0);
		const [x1, y1] = pt(prim.x1, prim.y1);
		const [x2, y2] = pt(prim.x2, prim.y2);
		const [x3, y3] = pt(prim.x3, prim.y3);
		return { ...prim, x0, y0, x1, y1, x2, y2, x3, y3 };
	}
	if (prim.kind === 'ellipse') {
		const [cx, cy] = pt(prim.cx, prim.cy);
		const [hx0, hy0] = applyLin(m, prim.rx, 0);
		const [hx1, hy1] = applyLin(m, 0, prim.ry);
		return {
			...prim,
			cx,
			cy,
			rx: Math.hypot(hx0, hx1),
			ry: Math.hypot(hy0, hy1)
		};
	}
	if (prim.kind === 'rect') {
		const corners = [
			pt(prim.x, prim.y),
			pt(prim.x + prim.w, prim.y),
			pt(prim.x + prim.w, prim.y + prim.h),
			pt(prim.x, prim.y + prim.h)
		];
		let minX = Infinity;
		let minY = Infinity;
		let maxX = -Infinity;
		let maxY = -Infinity;
		for (const [x, y] of corners) {
			minX = Math.min(minX, x);
			minY = Math.min(minY, y);
			maxX = Math.max(maxX, x);
			maxY = Math.max(maxY, y);
		}
		const [hx0, hy0] = applyLin(m, prim.rx, 0);
		const [hx1, hy1] = applyLin(m, 0, prim.ry);
		return {
			...prim,
			x: minX,
			y: minY,
			w: maxX - minX,
			h: maxY - minY,
			rx: Math.hypot(hx0, hx1),
			ry: Math.hypot(hy0, hy1)
		};
	}
	if (prim.kind === 'hole') {
		const [cx, cy] = pt(prim.cx, prim.cy);
		const [hx0, hy0] = applyLin(m, prim.r, 0);
		const [hx1, hy1] = applyLin(m, 0, prim.r);
		return { ...prim, cx, cy, r: Math.max(Math.hypot(hx0, hx1), Math.hypot(hy0, hy1)) };
	}
	const [x, y] = pt(prim.x, prim.y);
	const rad = (prim.angle * Math.PI) / 180;
	const [dx, dy] = applyLin(m, Math.cos(rad), Math.sin(rad));
	const det = m[0] * m[3] - m[1] * m[2];
	return {
		...prim,
		x,
		y,
		angle: (Math.atan2(dy, dx) * 180) / Math.PI,
		mirrored: det < 0 ? !prim.mirrored : prim.mirrored
	};
}

function pushEllipse(out: SvgPrim[], tag: string) {
	const cx = Number(attr(tag, 'cx'));
	const cy = Number(attr(tag, 'cy'));
	const rx = Number(attr(tag, 'rx'));
	const ry = Number(attr(tag, 'ry'));
	if (![cx, cy, rx, ry].every(Number.isFinite)) return;
	const strokeWidth = Number(attr(tag, 'stroke-width') ?? '0');
	out.push({
		kind: 'ellipse',
		cx,
		cy,
		rx,
		ry,
		fill: rgbAttr(tag, 'fill'),
		stroke: rgbAttr(tag, 'stroke'),
		strokeWidth: Number.isFinite(strokeWidth) ? strokeWidth : 0
	});
}
