import { SvelteMap } from 'svelte/reactivity';
import type { Locale } from '../i18n';
import type { Example } from '../lib/examples';
import { loadRecents, type RecentEntry } from '../lib/recentFiles';
import { loadSession, type SessionState } from '../lib/sessionStore';
import { decodeProject } from '../lib/shareCodec';
import { parsePropForm, type PropPatch } from '../lib/propForm';
import { registerSystemMonospace } from '../lib/systemFonts';
import { defaultStatus } from './engineTypes';
import type { ComponentCursor } from './engineTypes';
import type { LibGhost, Theme } from './types';
import { Engine } from './engine.svelte';
import { Dialogs } from './dialogs.svelte';
import { Settings } from './settings.svelte';
import { UiState } from './uiState.svelte';
import * as files from './fileOps';
import * as exportOps from './exportOps';
import * as comps from './componentOps';
import { restoreSession as restoreSessionState, SessionPersist } from './persistSession';
import * as clip from './clipboardOps';
import * as share from './shareOps';
import * as edit from './editCommands';
import { LibraryDragSession, getCursor } from './libraryDrag.svelte';
import { PreviewCache } from '../lib/previewCache';
import { APP_SHORTCUTS, runShortcuts } from './shortcuts';
import type { ExportFormat, ExportPreviewOpts } from '../lib/exportOptions';
import { startDesktopFileBridge } from '../lib/desktopFiles';
import { loadUserLibraries } from '../lib/userLibraries';
import type { SaveLibraryPolicy } from './fileOps';

export type { RecentEntry };
export type { LibGhost };

export type ProjectSettingsValues = {
	gridX: number;
	gridY: number;
	snapX: number;
	snapY: number;
	showGrid: boolean;
	snapEnable: boolean;
	hideComponentOrigin: boolean;
	strokeHundredths: number;
	defaultFilled: boolean;
};

export class AppSession {
	#session = loadSession();
	#persist = new SessionPersist(this);
	engine = $state<Engine | null>(null);
	settings = new Settings(
		() => this.engine,
		(theme) => {
			this.cursorCache.clear();
			this.previewCache.clear();
			this.engine?.query((app) => {
				app.set_theme(theme);
				app.render();
			});
			this.schedulePersist();
		},
		this.#session ? { locale: this.#session.locale, theme: this.#session.theme } : undefined
	);
	ui = new UiState();
	dialogs = new Dialogs();
	fileHandleName = $state(this.#session?.name ?? 'untitled.fcd');
	filePicker: HTMLInputElement | undefined;
	libraryPicker: HTMLInputElement | undefined;
	libGhost = $state<LibGhost | null>(null);
	cursorCache = new SvelteMap<string, ComponentCursor>();
	previewCache = new PreviewCache();
	recents = $state<RecentEntry[]>(loadRecents());
	savedSnapshot = $state('');
	pendingDiscard: (() => void) | null = null;
	#libraryDrag = new LibraryDragSession(this);
	#titleSig = '';
	#titleEpoch = $state(0);

	windowTitle = $derived.by(() => {
		void this.#titleEpoch;
		void this.savedSnapshot;
		const name = this.fileHandleName || 'untitled.fcd';
		return files.isDirty(this) ? `*${name}` : name;
	});

	get locale() {
		return this.settings.locale;
	}
	get theme() {
		return this.settings.theme;
	}
	get t() {
		return this.settings.t;
	}
	get status() {
		return this.engine?.status ?? defaultStatus();
	}
	get layers() {
		return this.engine?.layers ?? { layers: [] };
	}
	get libs() {
		return this.engine?.libs ?? [];
	}
	get menu() {
		return this.ui.menu;
	}
	set menu(v) {
		this.ui.menu = v;
	}
	get ctxMenu() {
		return this.ui.ctxMenu;
	}
	set ctxMenu(v) {
		this.ui.ctxMenu = v;
	}
	get rightTab() {
		return this.ui.rightTab;
	}
	set rightTab(v) {
		this.ui.rightTab = v;
	}
	get rightCollapsed() {
		return this.ui.rightCollapsed;
	}
	set rightCollapsed(v) {
		this.ui.rightCollapsed = v;
	}
	get editingLayerName() {
		return this.ui.editingLayerName;
	}
	set editingLayerName(v) {
		this.ui.editingLayerName = v;
	}
	get editingLibraryField() {
		return this.ui.editingLibraryField;
	}
	set editingLibraryField(v) {
		this.ui.editingLibraryField = v;
	}
	get editingLibraryTitle() {
		return this.ui.editingLibraryTitle;
	}
	set editingLibraryTitle(v) {
		this.ui.editingLibraryTitle = v;
	}
	get expandedUserLibs() {
		return this.ui.expandedUserLibs;
	}
	set expandedUserLibs(v) {
		this.ui.expandedUserLibs = v;
	}
	get libraryFocus() {
		return this.ui.libraryFocus;
	}
	set libraryFocus(v) {
		this.ui.libraryFocus = v;
	}
	get error() {
		const d = this.dialogs.dialog;
		return d?.kind === 'error' ? d.message : '';
	}
	set error(msg: string) {
		if (msg) this.dialogs.open({ kind: 'error', message: msg });
		else if (this.dialogs.dialog?.kind === 'error') this.dialogs.close();
	}

	assetUrl = (path: string) => `${import.meta.env.BASE_URL}${path.replace(/^\//, '')}`;

	refresh = () => {
		this.engine?.refresh();
		this.syncTitleEpoch();
	};

	afterChange = () => {
		this.engine?.mutate(() => {});
	};

	syncTitleEpoch = () => {
		const s = this.engine?.status;
		const sig = s
			? `${s.can_undo}|${s.can_redo}|${s.n}|${Number(s.editing_component_dirty)}|${s.editing_component ?? ''}|${s.libs_rev}`
			: '';
		if (sig === this.#titleSig) return;
		this.#titleSig = sig;
		this.#titleEpoch += 1;
	};

	toggleMenu = (id: string) => this.ui.toggleMenu(id);
	closeMenu = () => this.ui.closeMenu();

	openContextMenu = (x: number, y: number) => {
		this.ui.menu = null;
		this.ui.editingLayerName = null;
		this.ui.editingLibraryField = null;
		this.ui.editingLibraryTitle = null;
		this.ui.ctxMenu = { kind: 'edit', x, y };
	};

	openLayerContextMenu = (x: number, y: number, index: number) => {
		this.ui.menu = null;
		this.ui.editingLayerName = null;
		this.ui.editingLibraryField = null;
		this.ui.editingLibraryTitle = null;
		this.ui.ctxMenu = { kind: 'layer', x, y, index };
		this.setLayer(index);
	};

	openLibraryItemContextMenu = (x: number, y: number, stem: string, key: string) => {
		this.ui.menu = null;
		this.ui.editingLayerName = null;
		this.ui.editingLibraryField = null;
		this.ui.editingLibraryTitle = null;
		this.ui.ctxMenu = { kind: 'libraryItem', x, y, stem, key };
		this.ui.libraryFocus = { stem, key };
	};

	openLibraryContextMenu = (x: number, y: number, stem: string) => {
		this.ui.menu = null;
		this.ui.editingLayerName = null;
		this.ui.editingLibraryField = null;
		this.ui.editingLibraryTitle = null;
		this.ui.ctxMenu = { kind: 'library', x, y, stem };
	};

	libraryTitle = (stem: string) => {
		if (stem === 'project') return this.t.projectLibrary;
		return this.libs.find((l) => l.stem === stem)?.title ?? stem;
	};

	beginRenameLayer = (index: number) => {
		this.ui.ctxMenu = null;
		this.ui.editingLayerName = index;
	};

	applyTheme = () => this.settings.applyTheme();
	setLocale = (loc: Locale) => {
		this.settings.setLocale(loc);
		this.schedulePersist();
	};
	setTheme = (theme: Theme) => this.settings.setTheme(theme);

	init = async () => {
		this.applyTheme();
		const initWasm = (await import('../wasm/fidorust_wasm.js')).default;
		const { App } = await import('../wasm/fidorust_wasm.js');
		await initWasm();
		this.engine = new Engine(new App());
		await registerSystemMonospace(this.engine.app);
		this.engine.app.render();
		const userLibs = loadUserLibraries();
		this.engine.query((app) => {
			app.set_locale(this.locale);
			app.set_theme(this.theme);
			if (userLibs.length) app.load_user_libraries(JSON.stringify(userLibs));
			if (this.#session) {
				app.set_hide_component_origin(this.#session.hideComponentOrigin);
			}
		});
		this.refresh();
		const fromLaunch = await startDesktopFileBridge(this);
		if (!fromLaunch) {
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
			} else if (this.#session) {
				this.restoreSession(this.#session);
			} else {
				this.markClean();
			}
		}
		this.syncTitleEpoch();
		this.#persist.enabled = true;
		if (this.engine) {
			let libsRev = this.engine.libsRev;
			this.engine.onRefresh = () => {
				if (this.engine && this.engine.libsRev !== libsRev) {
					libsRev = this.engine.libsRev;
					this.cursorCache.clear();
					this.previewCache.clear();
				}
				this.syncTitleEpoch();
				this.schedulePersist();
			};
		}
		window.addEventListener('pagehide', this.flushPersist);
		document.addEventListener('visibilitychange', this.onVisibilityChange);
		this.schedulePersist();
	};

	restoreSession = (session: SessionState) => restoreSessionState(this, session);

	schedulePersist = () => this.#persist.schedule();

	flushPersist = () => this.#persist.flush();

	onVisibilityChange = () => this.#persist.onVisibilityChange();

	persistNow = () => this.#persist.persistNow();

	onKey = (e: KeyboardEvent) => {
		if (
			e.key === 'Escape' &&
			(this.dialogs.dialog?.kind === 'deleteLayer' ||
				this.dialogs.dialog?.kind === 'deleteComponent' ||
				this.dialogs.dialog?.kind === 'deleteLibrary')
		) {
			this.dialogs.close();
			e.preventDefault();
			return;
		}
		if (this.dialogs.isOpen) return;
		if (e.defaultPrevented) return;
		const target = e.target as HTMLElement | null;
		if (target?.closest('input, textarea, select, [contenteditable]')) return;
		const meta = e.metaKey || e.ctrlKey;
		if (this.engine?.query((app) => app.key(e.key, meta))) {
			e.preventDefault();
			this.engine.query((app) => {
				app.render();
			});
			this.engine.refresh();
		}
		runShortcuts(this, e, APP_SHORTCUTS);
	};

	onDragOver = files.onDragOver;
	onDropFile = (e: DragEvent) => files.onDropFile(this, e);

	tool = (id: string) => {
		this.engine?.query((app) => {
			app.set_tool(id);
		});
		if (id !== 'component') this.libraryFocus = null;
		if (id === 'text') {
			const txt = prompt(this.t.textPrompt, 'TEXT');
			if (txt)
				this.engine?.query((app) => {
					app.set_pending_text(txt);
				});
		}
		this.afterChange();
	};

	openAbout = () => this.dialogs.open({ kind: 'about' });
	openTechnologies = () => this.dialogs.open({ kind: 'technologies' });
	openProjectSettings = () => this.dialogs.open({ kind: 'projectSettings' });

	openProperties = () => {
		if (!this.engine || this.status.selected === 0) return;
		const fields = parsePropForm(this.engine.query((app) => app.selection_props_form_json()));
		this.dialogs.open({ kind: 'properties', fields });
	};

	applyProperties = (patch: PropPatch) => {
		if (!this.engine) return;
		try {
			this.engine.mutate((app) => {
				app.apply_selection_props(JSON.stringify(patch));
			});
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
		if (this.dialogs.dialog?.kind === 'properties') this.dialogs.close();
	};

	setLayer = (n: number) => {
		this.engine?.mutate((app) => {
			app.set_layer(n);
		});
	};

	isDirty = () => files.isDirty(this);
	markClean = () => {
		files.markClean(this);
		this.schedulePersist();
	};
	rememberCurrent = (name: string) => files.rememberCurrent(this, name);
	confirmDiscard = (action: () => void) => files.confirmDiscard(this, action);
	acceptDiscard = () => files.acceptDiscard(this);
	cancelDiscard = () => files.cancelDiscard(this);
	loadBytes = (bytes: Uint8Array, name: string) => files.loadBytes(this, bytes, name);
	loadText = (text: string, name: string) => files.loadText(this, text, name);
	openExample = (ex: Example) => files.openExample(this, ex);
	openFile = () => files.openFile(this);
	onPickedFile = (e: Event) => files.onPickedFile(this, e);
	onPickedLibraries = (e: Event) => files.onPickedLibraries(this, e);
	openRecent = (entry: RecentEntry) => files.openRecent(this, entry);
	requestNewDoc = () => files.requestNewDoc(this);
	newDoc = () => files.newDoc(this);
	saveFile = () => files.saveFile(this);
	importLibrary = () => files.importLibrary(this);
	exportLibrary = (stem: string) => files.exportLibrary(this, stem);
	openExport = (format: ExportFormat) => files.openExport(this, format);
	confirmExport = (opts: ExportPreviewOpts, svg: string) =>
		exportOps.confirmExport(this, opts, svg);
	copyFcd = () => clip.copyFcd(this);
	cutFcd = () => clip.cutFcd(this);
	pasteFcd = () => clip.pasteFcd(this);
	pasteNewDoc = () => clip.pasteNewDoc(this);
	openShareLink = () => share.openShareLink(this);
	openShareFcd = () => share.openShareFcd(this);
	closeShare = () => share.closeShare(this);
	doDelete = () => edit.doDelete(this);
	doUndo = () => edit.doUndo(this);
	doRedo = () => edit.doRedo(this);
	doRotate = () => edit.doRotate(this);
	doMirror = () => edit.doMirror(this);
	doSplit = () => edit.doSplit(this);
	doSelectAll = () => edit.doSelectAll(this);
	doInvert = () => edit.doInvert(this);
	fit = () => edit.fit(this);
	duplicateSelection = () => edit.duplicateSelection(this);

	togglePcb = () => {
		this.engine?.mutate((app) => {
			app.set_pcb_mode(!this.status.pcb);
		});
	};

	pickComponent = (stem: string, key: string) => comps.pickComponent(this, stem, key);

	revealLibraryItem = (stem: string, key: string) => comps.revealLibraryItem(this, stem, key);

	createComponentFromSelection = (target: string) =>
		comps.createComponentFromSelection(this, target);

	enterComponentEdit = (stem: string, key: string) => comps.enterComponentEdit(this, stem, key);

	editSelectedComponent = () => comps.editSelectedComponent(this);

	saveComponentEdit = () => comps.saveComponentEdit(this);

	cancelComponentEdit = () => comps.cancelComponentEdit(this);

	beginRenameComponent = (stem: string, key: string) => comps.beginRenameComponent(this, stem, key);

	beginEditComponentKey = (stem: string, key: string) =>
		comps.beginEditComponentKey(this, stem, key);

	renameComponent = (stem: string, key: string, name: string) =>
		comps.renameComponent(this, stem, key, name);

	renameComponentKey = (stem: string, key: string, newKey: string) =>
		comps.renameComponentKey(this, stem, key, newKey);

	moveComponent = (stem: string, key: string, destStem: string) =>
		comps.moveComponent(this, stem, key, destStem);

	requestDeleteComponent = (stem: string, key: string) =>
		comps.requestDeleteComponent(this, stem, key);

	confirmDeleteComponent = (stem: string, key: string) =>
		comps.confirmDeleteComponent(this, stem, key);

	cancelDeleteComponent = () => comps.cancelDeleteComponent(this);

	createUserLibrary = () => comps.createUserLibrary(this);

	beginRenameLibrary = (stem: string) => comps.beginRenameLibrary(this, stem);

	renameLibrary = (stem: string, title: string) => comps.renameLibrary(this, stem, title);

	requestDeleteLibrary = (stem: string) => comps.requestDeleteLibrary(this, stem);

	confirmDeleteLibrary = (stem: string) => comps.confirmDeleteLibrary(this, stem);

	cancelDeleteLibrary = () => comps.cancelDeleteLibrary(this);

	confirmSaveLocalComponents = (policy: SaveLibraryPolicy) => {
		const d = this.dialogs.dialog;
		if (d?.kind !== 'saveLocalComponents') return;
		const purpose = d.purpose;
		this.dialogs.close();
		if (purpose === 'save') files.finishSaveFcd(this, policy);
		else if (purpose === 'shareFcd') share.openShareFcdWithPolicy(this, policy);
		else void share.openShareLinkWithPolicy(this, policy);
	};

	cancelSaveLocalComponents = () => {
		if (this.dialogs.dialog?.kind === 'saveLocalComponents') this.dialogs.close();
	};

	getCursor = (name: string) => getCursor(this, name);
	armLibraryDrag = (name: string, e: PointerEvent) => this.#libraryDrag.arm(name, e);

	applyProjectSettings = (v: ProjectSettingsValues) => {
		this.engine?.mutate((app) => {
			app.apply_project_settings(
				JSON.stringify({
					grid: v.gridX,
					grid_y: v.gridY,
					snap: v.snapX,
					snap_y: v.snapY,
					show_grid: v.showGrid,
					snap_enable: v.snapEnable,
					hide_component_origin: v.hideComponentOrigin,
					stroke_hundredths: v.strokeHundredths,
					default_filled: v.defaultFilled
				})
			);
		});
		this.dialogs.close();
	};

	setGrid = (x: number, y: number) => {
		this.engine?.mutate((app) => {
			app.set_grid(x, y);
		});
	};

	setSnap = (x: number, y: number) => {
		this.engine?.mutate((app) => {
			app.set_snap(x, y);
		});
	};

	setShowGrid = (on: boolean) => {
		this.engine?.mutate((app) => {
			app.set_show_grid(on);
		});
	};

	setSnapEnable = (on: boolean) => {
		this.engine?.mutate((app) => {
			app.set_snap_enable(on);
		});
	};

	addLayer = () => {
		this.engine?.mutate((app) => {
			app.add_layer();
		});
	};

	reorderLayer = (from: number, to: number) => {
		if (from === to) return;
		this.engine?.mutate((app) => {
			app.reorder_layer(from, to);
		});
	};

	requestDeleteLayer = (i: number) => {
		if (this.layers.layers.length <= 1) return;
		const n = this.engine?.query((app) => app.layer_object_count(i)) ?? 0;
		if (n === 0) {
			this.engine?.mutate((app) => {
				app.delete_layer(i, 'objects', 0);
			});
			return;
		}
		this.dialogs.open({ kind: 'deleteLayer', index: i });
	};

	confirmDeleteLayer = (mode: 'objects' | 'move', moveTo: number) => {
		const d = this.dialogs.dialog;
		const i = d?.kind === 'deleteLayer' ? d.index : null;
		this.dialogs.close();
		if (i === null) return;
		this.applyDeleteLayer(i, mode, moveTo);
	};

	applyDeleteLayer = (i: number, mode: 'objects' | 'move', moveTo: number) => {
		this.editingLayerName = null;
		this.engine?.mutate((app) => {
			app.delete_layer(i, mode, moveTo);
		});
	};

	cancelDeleteLayer = () => {
		if (this.dialogs.dialog?.kind === 'deleteLayer') this.dialogs.close();
	};

	setLayerColor = (i: number, r: number, g: number, b: number, a = 255) => {
		this.engine?.mutate((app) => {
			app.set_layer_color(i, r, g, b, a);
		});
	};

	setLayerName = (i: number, name: string) => {
		this.engine?.query((app) => {
			app.set_layer_name(i, name);
		});
		this.editingLayerName = null;
		this.refresh();
	};

	setLayerShow = (i: number, show: boolean) => {
		this.engine?.mutate((app) => {
			app.set_layer_show(i, show);
		});
	};
}
