export function rgbToHex(color: number[]): string {
	return '#' + color.map((c) => c.toString(16).padStart(2, '0')).join('');
}

export function hexToRgb(value: string): [number, number, number] {
	return [
		parseInt(value.slice(1, 3), 16),
		parseInt(value.slice(3, 5), 16),
		parseInt(value.slice(5, 7), 16)
	];
}
