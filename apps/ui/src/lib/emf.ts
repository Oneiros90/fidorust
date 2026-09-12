import { MM_PER_LU } from './constants';
import { parseSvgPrims, parseSvgViewBox, type Rgba, type SvgPrim } from './svgGeom';

const EMR_HEADER = 1;
const EMR_POLYBEZIER = 5;
const EMR_POLYGON = 3;
const EMR_POLYLINE = 4;
const EMR_EOF = 14;
const EMR_SETBKMODE = 18;
const EMR_SETPOLYFILLMODE = 19;
const EMR_SETTEXTALIGN = 22;
const EMR_SELECTOBJECT = 37;
const EMR_CREATEPEN = 38;
const EMR_CREATEBRUSHINDIRECT = 39;
const EMR_DELETEOBJECT = 40;
const EMR_ELLIPSE = 42;
const EMR_RECTANGLE = 43;
const EMR_ROUNDRECT = 44;
const EMR_EXTCREATEFONTINDIRECTW = 82;
const EMR_EXTTEXTOUTW = 84;
const ENHMETA_SIGNATURE = 0x464d4520;
const TRANSPARENT = 1;
const WINDING = 2;
const PS_SOLID = 0;
const BS_SOLID = 0;
const WHITE_BRUSH = 0;
const NULL_BRUSH = 5;
const NULL_PEN = 8;
const STOCK = 0x80000000;

class Buf {
	bytes: number[] = [];
	u16(v: number) {
		this.bytes.push(v & 255, (v >>> 8) & 255);
	}
	i32(v: number) {
		const x = v | 0;
		this.bytes.push(x & 255, (x >>> 8) & 255, (x >>> 16) & 255, (x >>> 24) & 255);
	}
	u32(v: number) {
		this.i32(v);
	}
	color(r: number, g: number, b: number) {
		this.u32((r & 255) | ((g & 255) << 8) | ((b & 255) << 16));
	}
	patchU32(at: number, v: number) {
		this.bytes[at] = v & 255;
		this.bytes[at + 1] = (v >>> 8) & 255;
		this.bytes[at + 2] = (v >>> 16) & 255;
		this.bytes[at + 3] = (v >>> 24) & 255;
	}
}

function unit(v: number, scale: number): number {
	return Math.round(v * scale);
}

function boundsOf(pts: [number, number][]): { l: number; t: number; r: number; b: number } {
	let l = pts[0][0],
		t = pts[0][1],
		r = pts[0][0],
		b = pts[0][1];
	for (const [x, y] of pts) {
		l = Math.min(l, x);
		t = Math.min(t, y);
		r = Math.max(r, x);
		b = Math.max(b, y);
	}
	return { l, t, r, b };
}

export function svgToEmf(svg: string, scale = 1, background: Rgba | null = null): Uint8Array {
	const box = parseSvgViewBox(svg);
	const s = Math.max(0.01, scale);
	const x0 = box.x;
	const y0 = box.y;
	const mapX = (x: number) => unit(x - x0, s);
	const mapY = (y: number) => unit(y - y0, s);
	const prims = parseSvgPrims(svg);
	const w = Math.max(1, unit(box.w, s));
	const h = Math.max(1, unit(box.h, s));
	const mmW = Math.max(1, Math.round(box.w * MM_PER_LU * s));
	const mmH = Math.max(1, Math.round(box.h * MM_PER_LU * s));
	const frameR = Math.round(box.w * 12.7 * s);
	const frameB = Math.round(box.h * 12.7 * s);

	const buf = new Buf();
	buf.u32(EMR_HEADER);
	buf.u32(88);
	buf.i32(0);
	buf.i32(0);
	buf.i32(w - 1);
	buf.i32(h - 1);
	buf.i32(0);
	buf.i32(0);
	buf.i32(frameR);
	buf.i32(frameB);
	buf.u32(ENHMETA_SIGNATURE);
	buf.u32(0x10000);
	const nBytesAt = buf.bytes.length;
	buf.u32(0);
	const nRecordsAt = buf.bytes.length;
	buf.u32(0);
	const nHandlesAt = buf.bytes.length;
	buf.u16(0);
	buf.u16(0);
	buf.u32(0);
	buf.u32(0);
	buf.u32(0);
	buf.i32(w);
	buf.i32(h);
	buf.i32(mmW);
	buf.i32(mmH);

	let records = 1;
	let nextHandle = 1;
	let maxHandle = 1;

	const rec = () => {
		records += 1;
	};

	const setMode = (type: number, mode: number) => {
		buf.u32(type);
		buf.u32(12);
		buf.u32(mode);
		rec();
	};
	setMode(EMR_SETBKMODE, TRANSPARENT);
	setMode(EMR_SETPOLYFILLMODE, WINDING);

	const select = (ih: number) => {
		buf.u32(EMR_SELECTOBJECT);
		buf.u32(12);
		buf.u32(ih);
		rec();
	};
	const del = (ih: number) => {
		buf.u32(EMR_DELETEOBJECT);
		buf.u32(12);
		buf.u32(ih);
		rec();
	};
	const createPen = (width: number, r: number, g: number, b: number) => {
		const ih = nextHandle++;
		maxHandle = Math.max(maxHandle, ih);
		buf.u32(EMR_CREATEPEN);
		buf.u32(28);
		buf.u32(ih);
		buf.u32(PS_SOLID);
		buf.i32(Math.max(1, unit(width, s)));
		buf.i32(0);
		buf.color(r, g, b);
		rec();
		return ih;
	};
	const createBrush = (r: number, g: number, b: number) => {
		const ih = nextHandle++;
		maxHandle = Math.max(maxHandle, ih);
		buf.u32(EMR_CREATEBRUSHINDIRECT);
		buf.u32(24);
		buf.u32(ih);
		buf.u32(BS_SOLID);
		buf.color(r, g, b);
		buf.u32(0);
		rec();
		return ih;
	};

	const emitPoly = (
		pts: [number, number][],
		fill: Rgba | null,
		stroke: Rgba | null,
		strokeWidth: number
	) => {
		if (pts.length < 2) return;
		const bb = boundsOf(pts);
		let ownBrush = 0;
		let ownPen = 0;
		if (fill) {
			ownBrush = createBrush(fill[0], fill[1], fill[2]);
			select(ownBrush);
		} else {
			select(STOCK | NULL_BRUSH);
		}
		if (stroke) {
			ownPen = createPen(strokeWidth, stroke[0], stroke[1], stroke[2]);
			select(ownPen);
		} else {
			select(STOCK | NULL_PEN);
		}
		const type = fill ? EMR_POLYGON : EMR_POLYLINE;
		buf.u32(type);
		buf.u32(28 + 8 * pts.length);
		buf.i32(bb.l);
		buf.i32(bb.t);
		buf.i32(bb.r);
		buf.i32(bb.b);
		buf.u32(pts.length);
		for (const [x, y] of pts) {
			buf.i32(x);
			buf.i32(y);
		}
		rec();
		select(STOCK | WHITE_BRUSH);
		select(STOCK | NULL_PEN);
		if (ownBrush) del(ownBrush);
		if (ownPen) del(ownPen);
	};

	const emit = (prim: SvgPrim) => {
		if (prim.kind === 'polygon') {
			const pts = prim.pts.map(([x, y]) => [mapX(x), mapY(y)] as [number, number]);
			emitPoly(pts, prim.fill, prim.stroke, prim.strokeWidth);
			return;
		}
		if (prim.kind === 'line') {
			const pen = createPen(prim.width, prim.r, prim.g, prim.b);
			select(pen);
			const x1 = mapX(prim.x1);
			const y1 = mapY(prim.y1);
			const x2 = mapX(prim.x2);
			const y2 = mapY(prim.y2);
			buf.u32(EMR_POLYLINE);
			buf.u32(44);
			buf.i32(Math.min(x1, x2));
			buf.i32(Math.min(y1, y2));
			buf.i32(Math.max(x1, x2));
			buf.i32(Math.max(y1, y2));
			buf.u32(2);
			buf.i32(x1);
			buf.i32(y1);
			buf.i32(x2);
			buf.i32(y2);
			rec();
			select(STOCK | NULL_PEN);
			del(pen);
			if (prim.width >= 1.5) {
				const hr = prim.width / 2;
				const brush = createBrush(prim.r, prim.g, prim.b);
				select(brush);
				select(STOCK | NULL_PEN);
				for (const [cx, cy] of [
					[prim.x1, prim.y1],
					[prim.x2, prim.y2]
				] as const) {
					buf.u32(EMR_ELLIPSE);
					buf.u32(24);
					buf.i32(mapX(cx - hr));
					buf.i32(mapY(cy - hr));
					buf.i32(mapX(cx + hr));
					buf.i32(mapY(cy + hr));
					rec();
				}
				select(STOCK | WHITE_BRUSH);
				del(brush);
			}
			return;
		}
		if (prim.kind === 'ellipse') {
			const l = mapX(prim.cx - prim.rx);
			const t = mapY(prim.cy - prim.ry);
			const r = mapX(prim.cx + prim.rx);
			const b = mapY(prim.cy + prim.ry);
			let ownBrush = 0;
			let ownPen = 0;
			if (prim.fill) {
				ownBrush = createBrush(prim.fill[0], prim.fill[1], prim.fill[2]);
				select(ownBrush);
			} else {
				select(STOCK | NULL_BRUSH);
			}
			if (prim.stroke) {
				ownPen = createPen(prim.strokeWidth, prim.stroke[0], prim.stroke[1], prim.stroke[2]);
				select(ownPen);
			} else {
				select(STOCK | NULL_PEN);
			}
			buf.u32(EMR_ELLIPSE);
			buf.u32(24);
			buf.i32(l);
			buf.i32(t);
			buf.i32(r);
			buf.i32(b);
			rec();
			select(STOCK | WHITE_BRUSH);
			select(STOCK | NULL_PEN);
			if (ownBrush) del(ownBrush);
			if (ownPen) del(ownPen);
			return;
		}
		if (prim.kind === 'rect') {
			const l = mapX(prim.x);
			const t = mapY(prim.y);
			const r = mapX(prim.x + prim.w);
			const b = mapY(prim.y + prim.h);
			let ownBrush = 0;
			let ownPen = 0;
			if (prim.fill) {
				ownBrush = createBrush(prim.fill[0], prim.fill[1], prim.fill[2]);
				select(ownBrush);
			} else {
				select(STOCK | NULL_BRUSH);
			}
			if (prim.stroke) {
				ownPen = createPen(prim.strokeWidth, prim.stroke[0], prim.stroke[1], prim.stroke[2]);
				select(ownPen);
			} else {
				select(STOCK | NULL_PEN);
			}
			if (prim.rx > 0.01 || prim.ry > 0.01) {
				buf.u32(EMR_ROUNDRECT);
				buf.u32(32);
				buf.i32(l);
				buf.i32(t);
				buf.i32(r);
				buf.i32(b);
				buf.i32(unit(prim.rx * 2, s));
				buf.i32(unit(prim.ry * 2, s));
			} else {
				buf.u32(EMR_RECTANGLE);
				buf.u32(24);
				buf.i32(l);
				buf.i32(t);
				buf.i32(r);
				buf.i32(b);
			}
			rec();
			select(STOCK | WHITE_BRUSH);
			select(STOCK | NULL_PEN);
			if (ownBrush) del(ownBrush);
			if (ownPen) del(ownPen);
			return;
		}
		if (prim.kind === 'bezier') {
			const pts: [number, number][] = [
				[mapX(prim.x0), mapY(prim.y0)],
				[mapX(prim.x1), mapY(prim.y1)],
				[mapX(prim.x2), mapY(prim.y2)],
				[mapX(prim.x3), mapY(prim.y3)]
			];
			const bb = boundsOf(pts);
			const pen = createPen(prim.width, prim.r, prim.g, prim.b);
			select(pen);
			select(STOCK | NULL_BRUSH);
			buf.u32(EMR_POLYBEZIER);
			buf.u32(28 + 8 * pts.length);
			buf.i32(bb.l);
			buf.i32(bb.t);
			buf.i32(bb.r);
			buf.i32(bb.b);
			buf.u32(pts.length);
			for (const [x, y] of pts) {
				buf.i32(x);
				buf.i32(y);
			}
			rec();
			select(STOCK | NULL_PEN);
			del(pen);
			return;
		}
		if (prim.kind === 'text') {
			emitText(prim);
			return;
		}
		const l = mapX(prim.cx - prim.r);
		const t = mapY(prim.cy - prim.r);
		const r = mapX(prim.cx + prim.r);
		const b = mapY(prim.cy + prim.r);
		const brush = createBrush(255, 255, 255);
		select(brush);
		select(STOCK | NULL_PEN);
		buf.u32(EMR_ELLIPSE);
		buf.u32(24);
		buf.i32(l);
		buf.i32(t);
		buf.i32(r);
		buf.i32(b);
		rec();
		select(STOCK | WHITE_BRUSH);
		del(brush);
	};

	const emitText = (prim: Extract<SvgPrim, { kind: 'text' }>) => {
		const x = mapX(prim.x);
		const y = mapY(prim.y);
		const h = Math.max(1, unit(prim.fontSize, s));
		const face = prim.fontFamily.slice(0, 31);
		const ih = nextHandle++;
		maxHandle = Math.max(maxHandle, ih);
		buf.u32(EMR_EXTCREATEFONTINDIRECTW);
		buf.u32(104);
		buf.u32(ih);
		buf.i32(-h);
		buf.i32(0);
		buf.i32(Math.round(-prim.angle * 10));
		buf.i32(Math.round(-prim.angle * 10));
		buf.i32(prim.bold ? 700 : 400);
		buf.bytes.push(prim.italic ? 1 : 0, 0, 0, 1);
		buf.bytes.push(0, 0, 0, 0);
		for (let i = 0; i < 32; i++) {
			const code = face.charCodeAt(i) || 0;
			buf.u16(code);
		}
		rec();
		select(ih);
		buf.u32(EMR_SETTEXTALIGN);
		buf.u32(12);
		buf.u32(0);
		rec();
		buf.u32(24);
		buf.u32(12);
		buf.color(prim.fill[0], prim.fill[1], prim.fill[2]);
		rec();
		const chars = Array.from(prim.content);
		const utf16: number[] = chars.map((ch) => ch.codePointAt(0) ?? 32);
		const strBytes = utf16.length * 2;
		const strPad = (4 - (strBytes % 4)) % 4;
		const offString = 76;
		const size = 76 + strBytes + strPad;
		buf.u32(EMR_EXTTEXTOUTW);
		buf.u32(size);
		buf.i32(x);
		buf.i32(y);
		buf.i32(x + Math.max(1, unit(prim.textLength || prim.content.length * prim.fontSize * 0.6, s)));
		buf.i32(y + h);
		buf.u32(1);
		buf.bytes.push(0, 0, 0, 0, 0, 0, 0, 0);
		buf.i32(x);
		buf.i32(y);
		buf.u32(utf16.length);
		buf.u32(offString);
		buf.u32(0);
		buf.i32(0);
		buf.i32(0);
		buf.i32(0);
		buf.i32(0);
		buf.u32(0);
		for (const c of utf16) buf.u16(c);
		for (let i = 0; i < strPad; i++) buf.bytes.push(0);
		rec();
		select(STOCK | 13);
		del(ih);
	};

	if (background && background[3] > 0) {
		emit({
			kind: 'rect',
			x: box.x,
			y: box.y,
			w: box.w,
			h: box.h,
			rx: 0,
			ry: 0,
			fill: background,
			stroke: null,
			strokeWidth: 0
		});
	}
	for (const prim of prims) emit(prim);

	buf.u32(EMR_EOF);
	buf.u32(20);
	buf.u32(0);
	buf.u32(0);
	buf.u32(20);
	rec();

	buf.patchU32(nBytesAt, buf.bytes.length);
	buf.patchU32(nRecordsAt, records);
	buf.bytes[nHandlesAt] = (maxHandle + 1) & 255;
	buf.bytes[nHandlesAt + 1] = ((maxHandle + 1) >>> 8) & 255;
	return new Uint8Array(buf.bytes);
}
