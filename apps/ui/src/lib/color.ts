export function rgbToHex(color: number[]): string {
	const [r = 0, g = 0, b = 0] = color;
	return (
		'#' +
		[r, g, b]
			.map((c) =>
				Math.max(0, Math.min(255, Math.round(c)))
					.toString(16)
					.padStart(2, '0')
			)
			.join('')
	);
}

export function hexToRgb(value: string): [number, number, number] {
	return [
		parseInt(value.slice(1, 3), 16),
		parseInt(value.slice(3, 5), 16),
		parseInt(value.slice(5, 7), 16)
	];
}

export function layerAlpha(color: number[]): number {
	return color[3] ?? 255;
}

export function rgbaCss(color: number[]): string {
	const [r = 0, g = 0, b = 0] = color;
	const a = layerAlpha(color) / 255;
	if (a >= 0.999) return `rgb(${r},${g},${b})`;
	return `rgba(${r},${g},${b},${Math.round(a * 1000) / 1000})`;
}
