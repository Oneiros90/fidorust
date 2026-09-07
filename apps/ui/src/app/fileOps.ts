import { svgToPdfBlob } from '../lib/svgPdf';
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

export function exportSvg(s: AppSession) {
	if (!s.engine) return;
	download(
		s.fileHandleName.replace(/\.fcd$/i, '') + '.svg',
		s.engine.query((app) => app.export_svg()),
		'image/svg+xml'
	);
}

export function exportPng(s: AppSession) {
	const canvas = s.engine?.canvas;
	if (!canvas) return;
	canvas.toBlob((b) => {
		if (!b) return;
		download(s.fileHandleName.replace(/\.fcd$/i, '') + '.png', b, b.type || 'image/png');
	});
}

export function exportPdf(s: AppSession) {
	if (!s.engine) return;
	try {
		download(
			s.fileHandleName.replace(/\.fcd$/i, '') + '.pdf',
			svgToPdfBlob(s.engine.query((app) => app.export_svg())),
			'application/pdf'
		);
	} catch (err) {
		s.error = String(err);
	}
}

export function printDoc(s: AppSession) {
	const canvas = s.engine?.canvas;
	if (!canvas) return;
	const w = window.open('');
	if (!w) return;
	w.document.write(`<img src="${canvas.toDataURL('image/png')}" style="max-width:100%">`);
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
