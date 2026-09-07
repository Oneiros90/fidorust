import { svgToPdfBlob } from '../lib/svgPdf';
import { svgToEmf } from '../lib/emf';
import { canvasToPngBlob, rasterizeSvg } from '../lib/svgRaster';
import { MM_PER_LU } from '../lib/constants';
import { parseSvgViewBox, withSvgPixelSize, withSvgSizeAttrs } from '../lib/svgGeom';
import { wasmExportJson, type ExportFormat, type ExportPreviewOpts } from '../lib/exportOptions';
import type { RecentEntry } from '../lib/recentFiles';
import { pushRecent } from '../lib/recentFiles';
import type { Example } from '../lib/examples';
import type { App as WasmApp } from '../wasm/fidocad_wasm.js';
import type { AppSession } from './appSession.svelte';

export function download(name: string, content: string | Blob, mime: string) {
	const blob = content instanceof Blob ? content : new Blob([content], { type: mime });
	const a = document.createElement('a');
	a.href = URL.createObjectURL(blob);
	a.download = name;
	a.click();
	URL.revokeObjectURL(a.href);
}

export function isDirty(s: AppSession): boolean {
	if (!s.engine) return false;
	return s.engine.query((app) => app.save_fcd()) !== s.savedSnapshot;
}

export function markClean(s: AppSession) {
	s.savedSnapshot = s.engine?.query((app) => app.save_fcd()) ?? '';
}

export function rememberCurrent(s: AppSession, name: string) {
	if (!s.engine) return;
	const fcd = s.engine.query((app) => app.save_fcd());
	s.recents = pushRecent(s.recents, name, fcd);
}

export function confirmDiscard(s: AppSession, action: () => void) {
	if (!isDirty(s)) {
		action();
		return;
	}
	s.pendingDiscard = action;
	s.dialogs.open({ kind: 'discard' });
}

export function acceptDiscard(s: AppSession) {
	const action = s.pendingDiscard;
	s.pendingDiscard = null;
	s.dialogs.close();
	action?.();
}

export function cancelDiscard(s: AppSession) {
	s.pendingDiscard = null;
	s.dialogs.close();
}

export function applyLoaded(s: AppSession, load: (app: WasmApp) => void, name: string) {
	if (!s.engine) return;
	try {
		s.engine.mutate(load);
		s.fileHandleName = name;
		s.error = '';
		markClean(s);
		rememberCurrent(s, name);
	} catch (err) {
		s.error = String(err);
	}
}

export function loadBytes(s: AppSession, bytes: Uint8Array, name: string) {
	applyLoaded(s, (app) => app.load_fcd_bytes(bytes), name);
}

export function loadText(s: AppSession, text: string, name: string) {
	applyLoaded(s, (app) => app.load_fcd(text), name);
}

export function openExample(s: AppSession, ex: Example) {
	confirmDiscard(s, () => loadText(s, ex.fcd, ex.file));
}

export function openFile(s: AppSession) {
	s.filePicker?.click();
}

export async function onPickedFile(s: AppSession, e: Event) {
	const input = e.currentTarget as HTMLInputElement;
	const file = input.files?.[0];
	input.value = '';
	if (!file) return;
	const bytes = new Uint8Array(await file.arrayBuffer());
	confirmDiscard(s, () => loadBytes(s, bytes, file.name));
}

export function openRecent(s: AppSession, entry: RecentEntry) {
	confirmDiscard(s, () => loadText(s, entry.fcd, entry.name));
}

export function requestNewDoc(s: AppSession) {
	confirmDiscard(s, () => newDoc(s));
}

export function newDoc(s: AppSession) {
	s.engine?.mutate((app) => app.new_doc());
	s.fileHandleName = 'untitled.fcd';
	markClean(s);
}

export function saveFile(s: AppSession) {
	if (!s.engine) return;
	const name = s.fileHandleName.endsWith('.fcd') ? s.fileHandleName : 'drawing.fcd';
	download(
		name,
		s.engine.query((app) => app.save_fcd()),
		'text/plain'
	);
	s.fileHandleName = name;
	markClean(s);
	rememberCurrent(s, name);
}

export function openExport(s: AppSession, format: ExportFormat) {
	s.dialogs.open({ kind: 'export', format });
}

export function queryExportSvg(s: AppSession, opts: ExportPreviewOpts): string {
	if (!s.engine) return '';
	return s.engine.query((app) => app.export_svg(wasmExportJson(opts)));
}

function baseName(s: AppSession): string {
	return s.fileHandleName.replace(/\.fcd$/i, '') || 'drawing';
}

export async function confirmExport(s: AppSession, opts: ExportPreviewOpts, svg: string) {
	const name = baseName(s);
	try {
		if (opts.format === 'svg') {
			const box = parseSvgViewBox(svg);
			const scaled = withSvgPixelSize(svg, box.w * opts.scale, box.h * opts.scale);
			download(name + '.svg', scaled, 'image/svg+xml');
		} else if (opts.format === 'png') {
			const canvas = await rasterizeSvg(svg, {
				ppi: opts.ppi,
				whiteBg: opts.whiteBg,
				antiAlias: opts.antiAlias
			});
			const blob = await canvasToPngBlob(canvas);
			download(name + '.png', blob, blob.type || 'image/png');
		} else if (opts.format === 'pdf') {
			download(
				name + '.pdf',
				svgToPdfBlob(svg, {
					page: opts.pdfPage,
					landscape: opts.pdfLandscape,
					scale: opts.pdfScale
				}),
				'application/pdf'
			);
		} else if (opts.format === 'emf') {
			const bytes = svgToEmf(svg, opts.scale);
			const copy = new Uint8Array(bytes.byteLength);
			copy.set(bytes);
			download(name + '.emf', new Blob([copy.buffer], { type: 'image/x-emf' }), 'image/x-emf');
		} else {
			printSvg(svg, opts);
		}
		s.dialogs.close();
	} catch (err) {
		s.error = String(err);
	}
}

export function printSvg(svg: string, opts: ExportPreviewOpts) {
	const box = parseSvgViewBox(svg);
	const wMm = box.w * MM_PER_LU;
	const hMm = box.h * MM_PER_LU;
	const page =
		opts.printPage === 'letter'
			? `letter ${opts.printLandscape ? 'landscape' : 'portrait'}`
			: opts.printPage === 'a4'
				? `A4 ${opts.printLandscape ? 'landscape' : 'portrait'}`
				: `${wMm}mm ${hMm}mm`;
	const sized = opts.printScale === '1:1' ? withSvgSizeAttrs(svg, `${wMm}mm`, `${hMm}mm`) : svg;
	const svgCss =
		opts.printScale === 'fit'
			? 'svg{width:100%;height:auto;max-height:100%;display:block;}'
			: 'svg{display:block;}';
	const w = window.open('');
	if (!w) return;
	w.document.open();
	w.document.write(`<!DOCTYPE html><html><head><title>Print</title><style>
@page{size:${page};margin:${opts.printMarginMm}mm;}
html,body{margin:0;background:#fff;}
${svgCss}
</style></head><body>${sized}</body></html>`);
	w.document.close();
	w.focus();
	w.print();
}

export async function onDropFile(s: AppSession, e: DragEvent) {
	e.preventDefault();
	const file = e.dataTransfer?.files[0];
	if (!file) return;
	const bytes = new Uint8Array(await file.arrayBuffer());
	confirmDiscard(s, () => loadBytes(s, bytes, file.name));
}

export function onDragOver(e: DragEvent) {
	const types = [...(e.dataTransfer?.types ?? [])];
	if (types.includes('Files')) e.preventDefault();
}
