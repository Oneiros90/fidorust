export type SvgBox = { x: number; y: number; w: number; h: number };

export type SvgPrim =
	| { kind: 'polygon'; pts: [number, number][]; r: number; g: number; b: number }
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
	  }
	| {
			kind: 'ellipse';
			cx: number;
			cy: number;
			rx: number;
			ry: number;
			fill: [number, number, number] | null;
			stroke: [number, number, number] | null;
			strokeWidth: number;
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

function rgbAttr(tag: string, name: string): [number, number, number] | null {
	const v = attr(tag, name);
	if (!v || v === 'none') return null;
	const m = v.match(/rgb\((\d+),(\d+),(\d+)\)/);
	if (!m) return null;
	return [+m[1], +m[2], +m[3]];
}

export function parseSvgPrims(svg: string): SvgPrim[] {
	const body = svg.replace(/<defs>[\s\S]*?<\/defs>/g, '');
	const out: SvgPrim[] = [];
	const re = /<(polygon|line|ellipse|circle)\s([^>]*?)\/>/g;
	for (const m of body.matchAll(re)) {
		const name = m[1];
		const tag = m[2];
		if (name === 'polygon') {
			const fill = rgbAttr(tag, 'fill');
			const ptsRaw = attr(tag, 'points');
			if (!fill || !ptsRaw) continue;
			const pts = ptsRaw
				.trim()
				.split(/\s+/)
				.map((p) => {
					const [x, y] = p.split(',').map(Number);
					return [x, y] as [number, number];
				})
				.filter((p) => p.every(Number.isFinite));
			if (pts.length >= 3) out.push({ kind: 'polygon', pts, r: fill[0], g: fill[1], b: fill[2] });
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
				b: stroke[2]
			});
			continue;
		}
		if (name === 'ellipse') {
			const cx = Number(attr(tag, 'cx'));
			const cy = Number(attr(tag, 'cy'));
			const rx = Number(attr(tag, 'rx'));
			const ry = Number(attr(tag, 'ry'));
			if (![cx, cy, rx, ry].every(Number.isFinite)) continue;
			const fill = rgbAttr(tag, 'fill');
			const stroke = rgbAttr(tag, 'stroke');
			const strokeWidth = Number(attr(tag, 'stroke-width') ?? '0');
			out.push({
				kind: 'ellipse',
				cx,
				cy,
				rx,
				ry,
				fill,
				stroke,
				strokeWidth: Number.isFinite(strokeWidth) ? strokeWidth : 0
			});
			continue;
		}
		if (name === 'circle' && /\bclass="pad-hole"/.test(tag)) {
			const cx = Number(attr(tag, 'cx'));
			const cy = Number(attr(tag, 'cy'));
			const r = Number(attr(tag, 'r'));
			if ([cx, cy, r].every(Number.isFinite)) out.push({ kind: 'hole', cx, cy, r });
		}
	}
	return out;
}
