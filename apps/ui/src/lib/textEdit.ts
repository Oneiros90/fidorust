export type TextEdit = {
	text: string;
	wx: number;
	wy: number;
	sx: number;
	sy: number;
	angle: number;
	style: number;
	screenX: number;
	screenY: number;
	zoom: number;
};

export function parseEdit(raw: string): TextEdit | null {
	if (!raw || raw === 'null') return null;
	try {
		const o = JSON.parse(raw) as {
			text?: string;
			wx: number;
			wy: number;
			sx: number;
			sy: number;
			angle: number;
			style: number;
			screen_x: number;
			screen_y: number;
			zoom: number;
		};
		if (typeof o?.text !== 'string') return null;
		return {
			text: o.text,
			wx: o.wx,
			wy: o.wy,
			sx: o.sx,
			sy: o.sy,
			angle: o.angle,
			style: o.style,
			screenX: o.screen_x,
			screenY: o.screen_y,
			zoom: o.zoom
		};
	} catch {
		return null;
	}
}

export function textOverlayLayout(edit: TextEdit, scale: number) {
	return {
		x: edit.screenX / scale,
		y: edit.screenY / scale,
		fontSize: Math.max(8, (edit.sy * edit.zoom) / scale),
		charWidth: Math.max(4, (edit.sx * edit.zoom) / scale),
		angle: edit.angle,
		italic: (edit.style & 2) !== 0,
		mirrored: (edit.style & 4) !== 0,
		text: edit.text
	};
}
