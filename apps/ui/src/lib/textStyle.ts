/** Bit flags matching Rust `STYLE_BOLD=1, ITALIC=2, MIRRORED=4, UNDERLINE=8`. */
export const STYLE_BOLD = 1;
export const STYLE_ITALIC = 2;
export const STYLE_MIRRORED = 4;
export const STYLE_UNDERLINE = 8;

export function styleHasItalic(style: number): boolean {
	return (style & STYLE_ITALIC) !== 0;
}

export function styleHasMirrored(style: number): boolean {
	return (style & STYLE_MIRRORED) !== 0;
}
