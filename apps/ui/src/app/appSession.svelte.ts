import { SvelteMap } from 'svelte/reactivity';
import { dict, type Locale } from '../i18n';
import { svgToPdfBlob } from '../lib/svgPdf';
import { canvasLocal, cssPerLu, parseMacroCursor, type MacroCursor } from '../lib/libraryDrag';
import { loadRecents, pushRecent, type RecentEntry } from '../lib/recentFiles';
import { decodeProject, encodeProject, looksLikeFcd, shareUrl } from '../lib/shareCodec';
import type { App as WasmApp } from '../wasm/fidocad_wasm.js';
import { parsePropForm, type PropFormField, type PropPatch } from '../lib/propForm';
import {
	defaultStatus,
	type LayersData,
	type LibraryEntry,
	type Status,
	type Theme
} from './types';

export type { RecentEntry };

export type LibGhost = MacroCursor & { x: number; y: number; scale: number; rot: number };

function download(name: string, content: string | Blob, mime: string) {
	const blob = content instanceof Blob ? content : new Blob([content], { type: mime });
	const a = document.createElement('a');
	a.href = URL.createObjectURL(blob);
	a.download = name;
	a.click();
	URL.revokeObjectURL(a.href);
}

export class AppSession {
	engine = $state<WasmApp | null>(null);
	locale = $state<Locale>((navigator.language.startsWith('en') ? 'en' : 'it') as Locale);
	theme = $state<Theme>('light');
	t = $derived.by(() => dict(this.locale));
	status = $state<Status>(defaultStatus());
	libs = $state<LibraryEntry[]>([]);
	layers = $state<LayersData>({ layers: [] });
	showLayers = $state(false);
	showAbout = $state(false);
	showGridDlg = $state(false);
	showPropsDlg = $state(false);
	propsFormFields = $state<PropFormField[]>([]);
	splitMacros = $state(true);
	menu = $state<string | null>(null);
	error = $state('');
	fileHandleName = $state('untitled.fcd');
	filePicker: HTMLInputElement | undefined;
	ctxMenu = $state<{ x: number; y: number } | null>(null);
	libGhost = $state<LibGhost | null>(null);
	cursorCache = new SvelteMap<string, MacroCursor>();
	recents = $state<RecentEntry[]>(loadRecents());
	savedSnapshot = '';
	pendingDiscard: (() => void) | null = null;
	showDiscardConfirm = $state(false);
	shareLinkUrl = $state<string | null>(null);
	showShareLink = $state(false);
	shareFcdText = $state<string | null>(null);

	assetUrl = (path: string) => `${import.meta.env.BASE_URL}${path.replace(/^\//, '')}`;

	refresh = () => {
		if (!this.engine) return;
		this.status = JSON.parse(this.engine.status_json());
		this.layers = JSON.parse(this.engine.layers_json());
	};

	afterChange = () => {
		this.engine?.render();
		this.refresh();
	};

	toggleMenu = (id: string) => {
		this.menu = this.menu === id ? null : id;
	};

	closeMenu = () => {
		this.menu = null;
	};

	openContextMenu = (x: number, y: number) => {
		this.menu = null;
		this.ctxMenu = { x, y };
	};

	applyTheme = () => {
		document.documentElement.dataset.theme = this.theme;
		this.cursorCache.clear();
		this.engine?.set_theme(this.theme);
		this.engine?.render();
	};

	syncLocale = () => {
		document.documentElement.lang = this.locale;
		this.engine?.set_locale(this.locale);
	};

	init = async () => {
		this.applyTheme();
		const initWasm = (await import('../wasm/fidocad_wasm.js')).default;
		const { App } = await import('../wasm/fidocad_wasm.js');
		await initWasm();
		this.engine = new App();
		this.libs = JSON.parse(this.engine.library_json());
		this.engine.set_locale(this.locale);
		this.engine.set_theme(this.theme);
		this.refresh();
		const project = new URLSearchParams(window.location.search).get('project');
		if (project) {
			const url = new URL(window.location.href);
			url.searchParams.delete('project');
			const search = url.searchParams.toString();
			history.replaceState(null, '', `${url.pathname}${search ? `?${search}` : ''}${url.hash}`);
			try {
				this.loadText(await decodeProject(project), 'shared.fcd');
			} catch (err) {
				this.error = String(err);
				this.markClean();
			}
		} else {
			this.markClean();
		}
	};

	onKey = (e: KeyboardEvent) => {
		if (e.key === 'Escape' && this.showLayers) {
			this.showLayers = false;
			e.preventDefault();
			return;
		}
		if (e.key === 'Escape' && this.showPropsDlg) {
			this.showPropsDlg = false;
			e.preventDefault();
			return;
		}
		if (
			this.showGridDlg ||
			this.showPropsDlg ||
			this.showAbout ||
			this.error ||
			this.showDiscardConfirm ||
			this.showShareLink ||
			this.shareFcdText
		)
			return;
		if (e.defaultPrevented) return;
		const target = e.target as HTMLElement | null;
		if (target?.closest('input, textarea, select, [contenteditable]')) return;
		const meta = e.metaKey || e.ctrlKey;
		if (this.engine?.key(e.key, meta)) {
			e.preventDefault();
			this.engine.render();
			this.refresh();
		}
		if (meta && e.key.toLowerCase() === 'o') {
			e.preventDefault();
			this.openFile();
		}
		if (meta && e.key.toLowerCase() === 's') {
			e.preventDefault();
			this.saveFile();
		}
		if (meta && e.key.toLowerCase() === 'x') {
			e.preventDefault();
			void this.cutFcd();
		}
		if (meta && e.key.toLowerCase() === 'c') {
			e.preventDefault();
			void this.copyFcd();
		}
		if (meta && e.key.toLowerCase() === 'v') {
			e.preventDefault();
			void this.pasteFcd();
		}
		if (e.altKey && e.key === 'Enter' && this.status.selected > 0) {
			e.preventDefault();
			this.openProperties();
		}
	};

	onDragOver = (e: DragEvent) => {
		const types = [...(e.dataTransfer?.types ?? [])];
		if (types.includes('Files')) e.preventDefault();
	};

	onDropFile = async (e: DragEvent) => {
		e.preventDefault();
		const file = e.dataTransfer?.files[0];
		if (!file) return;
		const bytes = new Uint8Array(await file.arrayBuffer());
		this.confirmDiscard(() => this.loadBytes(bytes, file.name));
	};

	tool = (id: string) => {
		this.engine?.set_tool(id);
		if (id === 'text') {
			const txt = prompt(this.t.textPrompt, 'TEXT');
			if (txt) this.engine?.set_pending_text(txt);
		}
		this.afterChange();
	};

	openProperties = () => {
		if (!this.engine || this.status.selected === 0) return;
		this.propsFormFields = parsePropForm(this.engine.selection_props_form_json());
		this.showPropsDlg = true;
	};

	applyProperties = (patch: PropPatch) => {
		if (!this.engine) return;
		try {
			this.engine.apply_selection_props(JSON.stringify(patch));
			this.afterChange();
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
		this.showPropsDlg = false;
	};

	setLayer = (n: number) => {
		this.engine?.set_layer(n);
		this.afterChange();
	};

	isDirty = () => {
		if (!this.engine) return false;
		return this.engine.save_fcd() !== this.savedSnapshot;
	};

	markClean = () => {
		this.savedSnapshot = this.engine?.save_fcd() ?? '';
	};

	rememberCurrent = (name: string) => {
		if (!this.engine) return;
		this.recents = pushRecent(this.recents, name, this.engine.save_fcd());
	};

	confirmDiscard = (action: () => void) => {
		if (!this.isDirty()) {
			action();
			return;
		}
		this.pendingDiscard = action;
		this.showDiscardConfirm = true;
	};

	acceptDiscard = () => {
		const action = this.pendingDiscard;
		this.pendingDiscard = null;
		this.showDiscardConfirm = false;
		action?.();
	};

	cancelDiscard = () => {
		this.pendingDiscard = null;
		this.showDiscardConfirm = false;
	};

	applyLoaded = (load: () => void, name: string) => {
		if (!this.engine) return;
		try {
			load();
			this.fileHandleName = name;
			this.afterChange();
			this.error = '';
			this.markClean();
			this.rememberCurrent(name);
		} catch (err) {
			this.error = String(err);
		}
	};

	loadBytes = (bytes: Uint8Array, name: string) => {
		this.applyLoaded(() => this.engine!.load_fcd_bytes(bytes), name);
	};

	loadText = (text: string, name: string) => {
		this.applyLoaded(() => this.engine!.load_fcd(text), name);
	};

	loadFromFile = async (file: File) => {
		this.loadBytes(new Uint8Array(await file.arrayBuffer()), file.name);
	};

	openExample = async (file: string) => {
		this.confirmDiscard(() => void this.loadExampleFile(file));
	};

	loadExampleFile = async (file: string) => {
		const r = await fetch(this.assetUrl(file));
		if (!r.ok) {
			this.error = `${r.status} ${r.url}`;
			return;
		}
		this.loadBytes(new Uint8Array(await r.arrayBuffer()), file);
	};

	openSample = () => this.openExample('sample.fcd');

	openFile = () => {
		this.filePicker?.click();
	};

	onPickedFile = async (e: Event) => {
		const input = e.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;
		const bytes = new Uint8Array(await file.arrayBuffer());
		this.confirmDiscard(() => this.loadBytes(bytes, file.name));
	};

	openRecent = (entry: RecentEntry) => {
		this.confirmDiscard(() => this.loadText(entry.fcd, entry.name));
	};

	requestNewDoc = () => {
		this.confirmDiscard(() => this.newDoc());
	};

	newDoc = () => {
		this.engine?.new_doc();
		this.fileHandleName = 'untitled.fcd';
		this.afterChange();
		this.markClean();
	};

	saveFile = () => {
		if (!this.engine) return;
		const name = this.fileHandleName.endsWith('.fcd') ? this.fileHandleName : 'drawing.fcd';
		download(name, this.engine.save_fcd(), 'text/plain');
		this.fileHandleName = name;
		this.markClean();
		this.rememberCurrent(name);
	};

	exportSvg = () => {
		if (!this.engine) return;
		download(this.fileHandleName.replace(/\.fcd$/i, '') + '.svg', this.engine.export_svg(), 'image/svg+xml');
	};

	exportPng = () => {
		const canvas = document.getElementById('draw-canvas') as HTMLCanvasElement | null;
		if (!canvas) return;
		canvas.toBlob((b) => {
			if (!b) return;
			const a = document.createElement('a');
			a.href = URL.createObjectURL(b);
			a.download = this.fileHandleName.replace(/\.fcd$/i, '') + '.png';
			a.click();
		});
	};

	exportPdf = () => {
		if (!this.engine) return;
		try {
			download(
				this.fileHandleName.replace(/\.fcd$/i, '') + '.pdf',
				svgToPdfBlob(this.engine.export_svg()),
				'application/pdf'
			);
		} catch (err) {
			this.error = String(err);
		}
	};

	printDoc = () => {
		const canvas = document.getElementById('draw-canvas') as HTMLCanvasElement | null;
		if (!canvas) return;
		const w = window.open('');
		if (!w) return;
		w.document.write(`<img src="${canvas.toDataURL('image/png')}" style="max-width:100%">`);
		w.document.close();
		w.focus();
		w.print();
	};

	copyFcd = async () => {
		if (!this.engine) return;
		await navigator.clipboard.writeText(this.engine.clipboard_fcd());
	};

	cutFcd = async () => {
		if (!this.engine) return;
		await this.copyFcd();
		this.engine.key('Delete', false);
		this.afterChange();
	};

	pasteFcd = async () => {
		if (!this.engine) return;
		const text = await navigator.clipboard.readText();
		if (looksLikeFcd(text)) {
			this.engine.paste_selection(text);
			this.afterChange();
		}
	};

	pasteNewDoc = async () => {
		const text = await navigator.clipboard.readText();
		if (!looksLikeFcd(text)) {
			this.error = this.t.clipboardNotFcd;
			return;
		}
		this.confirmDiscard(() => this.loadText(text, 'clipboard.fcd'));
	};

	openShareLink = async () => {
		if (!this.engine) return;
		this.shareFcdText = null;
		this.showShareLink = true;
		this.shareLinkUrl = '';
		try {
			this.shareLinkUrl = shareUrl(await encodeProject(this.engine.save_fcd()));
		} catch (err) {
			this.showShareLink = false;
			this.shareLinkUrl = null;
			this.error = String(err);
		}
	};

	openShareFcd = () => {
		if (!this.engine) return;
		this.showShareLink = false;
		this.shareLinkUrl = null;
		this.shareFcdText = this.engine.save_fcd();
	};

	closeShare = () => {
		this.showShareLink = false;
		this.shareLinkUrl = null;
		this.shareFcdText = null;
	};

	doDelete = () => {
		this.engine?.key('Delete', false);
		this.afterChange();
	};

	doUndo = () => {
		this.engine?.undo();
		this.afterChange();
	};

	doRedo = () => {
		this.engine?.redo();
		this.afterChange();
	};

	doRotate = () => {
		this.engine?.rotate();
		this.afterChange();
	};

	doMirror = () => {
		this.engine?.mirror();
		this.afterChange();
	};

	doSplit = () => {
		this.engine?.split_selected_macros();
		this.afterChange();
	};

	doSelectAll = () => {
		this.engine?.key('a', true);
		this.afterChange();
	};

	doInvert = () => {
		this.engine?.invert_selection();
		this.afterChange();
	};

	fit = () => {
		this.engine?.fit();
		this.afterChange();
	};

	togglePcb = () => {
		this.engine?.set_pcb_mode(!this.status.pcb);
		this.refresh();
	};

	toggleSplitMacros = () => {
		this.splitMacros = !this.splitMacros;
		this.engine?.set_split_macros(this.splitMacros);
	};

	setLocale = (loc: Locale) => {
		this.locale = loc;
	};

	setTheme = (theme: Theme) => {
		this.theme = theme;
		this.applyTheme();
	};

	pickMacro = (stem: string, key: string) => {
		const name = stem === 'stdlib' ? key : `${stem}.${key}`;
		this.engine?.set_pending_macro(name);
		this.afterChange();
	};

	getCursor = (name: string): MacroCursor | null => {
		if (!this.engine) return null;
		const key = `${this.theme}:${name}`;
		let c = this.cursorCache.get(key);
		if (!c) {
			const parsed = parseMacroCursor(this.engine.macro_cursor_json(name));
			if (!parsed) return null;
			this.cursorCache.set(key, parsed);
			c = parsed;
		}
		return c;
	};

	armLibraryDrag = (name: string, e: PointerEvent) => {
		const pointerId = e.pointerId;
		const x0 = e.clientX;
		const y0 = e.clientY;
		let active = false;
		let rot = 0;

		const finish = () => {
			window.removeEventListener('pointermove', move);
			window.removeEventListener('pointerup', stop);
			window.removeEventListener('pointercancel', stop);
			window.removeEventListener('keydown', onEsc);
			window.removeEventListener('contextmenu', onCtx, true);
			document.body.classList.remove('lib-dragging');
			this.libGhost = null;
		};

		const move = (ev: PointerEvent) => {
			if (ev.pointerId !== pointerId || !this.engine) return;
			if (!active) {
				if (Math.hypot(ev.clientX - x0, ev.clientY - y0) < 5) return;
				active = true;
				document.body.classList.add('lib-dragging');
			}
			const canvas = document.getElementById('draw-canvas') as HTMLCanvasElement | null;
			if (canvas) {
				const loc = canvasLocal(canvas, ev.clientX, ev.clientY);
				if (loc.inside) {
					this.engine.pointer_move(loc.x, loc.y);
					this.engine.render();
					this.libGhost = null;
					return;
				}
				this.engine.clear_hover();
				this.engine.render();
			}
			const cur = this.getCursor(name);
			if (!cur) {
				this.libGhost = null;
				return;
			}
			this.libGhost = {
				...cur,
				x: ev.clientX,
				y: ev.clientY,
				scale: cssPerLu(this.status.zoom),
				rot
			};
		};

		const stop = (ev: PointerEvent) => {
			if (ev.pointerId !== pointerId || ev.button !== 0) return;
			const wasActive = active;
			finish();
			if (!wasActive || !this.engine) return;
			const canvas = document.getElementById('draw-canvas') as HTMLCanvasElement | null;
			if (!canvas) return;
			const loc = canvasLocal(canvas, ev.clientX, ev.clientY);
			if (loc.inside) {
				this.engine.place_macro_at(name, loc.x, loc.y);
				this.afterChange();
			} else {
				this.engine.clear_hover();
				this.engine.render();
			}
		};

		const onCtx = (ev: MouseEvent) => {
			ev.preventDefault();
			ev.stopPropagation();
			if (!this.engine) return;
			this.engine.pointer_right(0, 0);
			rot = (rot + 1) % 4;
			this.engine.render();
			if (this.libGhost) this.libGhost = { ...this.libGhost, rot };
		};

		const onEsc = (ke: KeyboardEvent) => {
			if (ke.key !== 'Escape') return;
			ke.preventDefault();
			finish();
			this.engine?.clear_hover();
			this.engine?.render();
		};

		window.addEventListener('pointermove', move);
		window.addEventListener('pointerup', stop);
		window.addEventListener('pointercancel', stop);
		window.addEventListener('keydown', onEsc);
		window.addEventListener('contextmenu', onCtx, true);
	};

	applyGrid = (v: {
		gridX: number;
		gridY: number;
		snapX: number;
		snapY: number;
		showGrid: boolean;
		snapEnable: boolean;
		hideMacroOrigin: boolean;
	}) => {
		this.engine?.set_grid(v.gridX, v.gridY);
		this.engine?.set_snap(v.snapX, v.snapY);
		this.engine?.set_show_grid(v.showGrid);
		this.engine?.set_snap_enable(v.snapEnable);
		this.engine?.set_hide_macro_origin(v.hideMacroOrigin);
		this.afterChange();
		this.showGridDlg = false;
	};

	setLayerColor = (i: number, r: number, g: number, b: number) => {
		this.engine?.set_layer_color(i, r, g, b);
		this.engine?.render();
	};

	setLayerName = (i: number, name: string) => {
		this.engine?.set_layer_name(i, name);
	};

	setLayerShow = (i: number, show: boolean) => {
		this.engine?.set_layer_show(i, show);
		this.afterChange();
	};

	setLayerPrint = (i: number, print: boolean) => {
		this.engine?.set_layer_print(i, print);
	};
}
