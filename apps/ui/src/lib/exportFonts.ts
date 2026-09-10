import { parseCssFontFamily } from './svgGeom';

const DEFAULT_FACE = 'Courier Prime';

function toBase64(bytes: Uint8Array): string {
	let bin = '';
	const chunk = 0x8000;
	for (let i = 0; i < bytes.length; i += chunk) {
		bin += String.fromCharCode(...bytes.subarray(i, i + chunk));
	}
	return btoa(bin);
}

function cssFamilyName(name: string): string {
	return name.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

function familiesInSvg(svg: string): string[] {
	const names = new Set<string>();
	for (const m of svg.matchAll(/font-family="([^"]*)"/g)) {
		names.add(parseCssFontFamily(m[1]));
	}
	if (names.size === 0 && svg.includes('<text')) names.add(DEFAULT_FACE);
	return [...names];
}

/**
 * Embed used TTF faces as `@font-face` so SVG/PNG viewers use the same
 * glyphs as the canvas instead of a system fallback.
 */
export function embedExportFonts(
	svg: string,
	fontBytes: (name: string) => Uint8Array | null | undefined
): string {
	if (!svg.includes('<svg') || svg.includes('@font-face')) return svg;
	const faces: string[] = [];
	for (const name of familiesInSvg(svg)) {
		const raw = fontBytes(name);
		if (!raw || raw.length === 0) continue;
		const copy = new Uint8Array(raw);
		faces.push(
			`@font-face{font-family:"${cssFamilyName(name)}";src:url(data:font/ttf;base64,${toBase64(copy)}) format("truetype");font-weight:400;font-style:normal}`
		);
	}
	if (!faces.length) return svg;
	const open = svg.match(/^<svg\b[^>]*>/);
	if (!open) return svg;
	return (
		svg.slice(0, open[0].length) +
		`<defs><style type="text/css"><![CDATA[${faces.join('')}]]></style></defs>` +
		svg.slice(open[0].length)
	);
}
