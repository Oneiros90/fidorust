export function parseSvgElement(raw: string): SVGElement | null {
	const parsed = new DOMParser().parseFromString(raw, 'image/svg+xml');
	const el = parsed.documentElement;
	if (el.tagName.toLowerCase() !== 'svg') return null;
	return document.importNode(el, true) as SVGElement;
}

export function attachSvgString(raw: string) {
	return (node: HTMLElement) => {
		if (!raw) return;
		const el = parseSvgElement(raw);
		if (el) node.replaceChildren(el);
	};
}
