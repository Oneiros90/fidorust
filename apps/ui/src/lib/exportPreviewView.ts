import type { ExportFormat } from './exportOptions';

export const PREVIEW_ZOOM_MIN = 0.05;
export const PREVIEW_ZOOM_MAX = 32;
export const PREVIEW_FIT_PAD = 24;
export const PREVIEW_FIT_ZOOM_CAP = 8;
export const PREVIEW_WHEEL_FACTOR = 1.12;

export type PreviewCamera = {
	zoom: number;
	panX: number;
	panY: number;
	fittedFormat: ExportFormat | null;
	artW: number;
	artH: number;
};

export function measureArt(viewerEl: HTMLDivElement | undefined) {
	const view = viewerEl;
	if (!view) return null;
	const art = view.querySelector('.art') as HTMLElement | null;
	if (!art) return null;
	const w = art.offsetWidth || art.scrollWidth;
	const h = art.offsetHeight || art.scrollHeight;
	if (!(w > 0 && h > 0)) return null;
	return { view, w, h };
}

export function clampPreviewZoom(z: number) {
	return Math.min(PREVIEW_ZOOM_MAX, Math.max(PREVIEW_ZOOM_MIN, z));
}

export function fitToView(
	cam: PreviewCamera,
	viewerEl: HTMLDivElement | undefined,
	format: ExportFormat
) {
	const s = measureArt(viewerEl);
	if (!s) return;
	const pad = PREVIEW_FIT_PAD;
	const vw = Math.max(1, s.view.clientWidth - pad);
	const vh = Math.max(1, s.view.clientHeight - pad);
	const z = Math.min(vw / s.w, vh / s.h, PREVIEW_FIT_ZOOM_CAP);
	cam.zoom = Number.isFinite(z) && z > 0 ? z : 1;
	cam.panX = (s.view.clientWidth - s.w * cam.zoom) / 2;
	cam.panY = (s.view.clientHeight - s.h * cam.zoom) / 2;
	cam.artW = s.w;
	cam.artH = s.h;
	cam.fittedFormat = format;
}

/** Keep pan/zoom across option changes; refit on first show, format switch, or aspect change. */
export function syncPreviewView(
	cam: PreviewCamera,
	viewerEl: HTMLDivElement | undefined,
	format: ExportFormat
) {
	const s = measureArt(viewerEl);
	if (!s) return;
	if (cam.fittedFormat !== format || cam.artW === 0) {
		fitToView(cam, viewerEl, format);
		return;
	}
	if (s.w === cam.artW && s.h === cam.artH) return;
	const oldAspect = cam.artW / Math.max(cam.artH, 1);
	const newAspect = s.w / Math.max(s.h, 1);
	if (oldAspect > 1 !== newAspect > 1) {
		fitToView(cam, viewerEl, format);
		return;
	}
	cam.zoom = clampPreviewZoom(cam.zoom * (cam.artW / s.w));
	cam.artW = s.w;
	cam.artH = s.h;
}

export function applyPreviewWheel(
	cam: PreviewCamera,
	viewerEl: HTMLDivElement | undefined,
	e: WheelEvent
) {
	const view = viewerEl;
	if (!view) return;
	const factor = e.deltaY < 0 ? PREVIEW_WHEEL_FACTOR : 1 / PREVIEW_WHEEL_FACTOR;
	const rect = view.getBoundingClientRect();
	const cx = e.clientX - rect.left;
	const cy = e.clientY - rect.top;
	const next = clampPreviewZoom(cam.zoom * factor);
	const k = next / cam.zoom;
	cam.panX = cx - (cx - cam.panX) * k;
	cam.panY = cy - (cy - cam.panY) * k;
	cam.zoom = next;
}
