export function clampInt(n: number, min: number, max: number): number {
	const v = Math.round(Number(n));
	if (!Number.isFinite(v)) return min;
	return Math.min(max, Math.max(min, v));
}
