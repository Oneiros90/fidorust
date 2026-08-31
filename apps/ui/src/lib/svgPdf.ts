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

function parseSvgSize(svg: string): { w: number; h: number } {
	const m = svg.match(/<svg[^>]*\bwidth="([\d.]+)"[^>]*\bheight="([\d.]+)"/);
	if (!m) throw new Error('Invalid SVG');
	const w = Number(m[1]);
	const h = Number(m[2]);
	if (!Number.isFinite(w) || !Number.isFinite(h) || w <= 0 || h <= 0) throw new Error('Invalid SVG size');
	return { w, h };
}

function svgToContentStream(svg: string, pageW: number, pageH: number, scale: number): string {
	const py = (y: number) => pageH - y * scale;
	const px = (x: number) => x * scale;
	const parts: string[] = ['1 1 1 rg', `0 0 ${n(pageW)} ${n(pageH)} re`, 'f'];

	const polyRe = /<polygon points="([^"]+)" fill="rgb\((\d+),(\d+),(\d+)\)"\/>/g;
	for (const m of svg.matchAll(polyRe)) {
		const pts = m[1].trim().split(/\s+/).map((p) => {
			const [x, y] = p.split(',').map(Number);
			return [px(x), py(y)] as const;
		});
		if (pts.length < 3) continue;
		parts.push(`${rgb(+m[2], +m[3], +m[4])} rg`);
		parts.push(`${n(pts[0][0])} ${n(pts[0][1])} m`);
		for (let i = 1; i < pts.length; i++) parts.push(`${n(pts[i][0])} ${n(pts[i][1])} l`);
		parts.push('h f');
	}

	const lineRe =
		/<line x1="([^"]+)" y1="([^"]+)" x2="([^"]+)" y2="([^"]+)" stroke="rgb\((\d+),(\d+),(\d+)\)" stroke-width="([^"]+)"[^/]*\/>/g;
	parts.push('1 J');
	for (const m of svg.matchAll(lineRe)) {
		const w = Number(m[8]) * scale;
		parts.push(`${rgb(+m[5], +m[6], +m[7])} RG`);
		parts.push(`${n(Math.max(w, 0.2))} w`);
		parts.push(`${n(px(+m[1]))} ${n(py(+m[2]))} m ${n(px(+m[3]))} ${n(py(+m[4]))} l S`);
	}

	return parts.join('\n') + '\n';
}

function wrapPdf(pageW: number, pageH: number, content: Uint8Array): Uint8Array {
	const enc = new TextEncoder();
	const header = enc.encode('%PDF-1.4\n%\x80\x80\x80\x80\n');
	const obj1 = enc.encode('1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n');
	const obj2 = enc.encode('2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n');
	const obj3 = enc.encode(
		`3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${n(pageW)} ${n(pageH)}] /Contents 4 0 R /Resources << >> >>\nendobj\n`
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

export function svgToPdfBlob(svg: string): Blob {
	const { w, h } = parseSvgSize(svg);
	const maxPt = 720;
	const scale = Math.min(1, maxPt / Math.max(w, h, 1));
	const pageW = Math.max(1, w * scale);
	const pageH = Math.max(1, h * scale);
	const content = new TextEncoder().encode(svgToContentStream(svg, pageW, pageH, scale));
	const pdf = wrapPdf(pageW, pageH, content);
	const copy = new Uint8Array(pdf.byteLength);
	copy.set(pdf);
	return new Blob([copy.buffer], { type: 'application/pdf' });
}
