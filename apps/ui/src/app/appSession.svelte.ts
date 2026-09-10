import { SvelteMap } from 'svelte/reactivity';
import type { Locale } from '../i18n';
import type { Example } from '../lib/examples';
import { loadRecents, type RecentEntry } from '../lib/recentFiles';
import {
	loadSession,
	saveSession,
	SESSION_DEBOUNCE_MS,
	type SessionState
} from '../lib/sessionStore';
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
import * as clip from './clipboardOps';
import * as share from './shareOps';
import * as edit from './editCommands';
import { LibraryDragSession, getCursor } from './libraryDrag.svelte';
import { PreviewCache } from '../lib/previewCache';
import { APP_SHORTCUTS, runShortcuts } from './shortcuts';
import type { ExportFormat, ExportPreviewOpts } from '../lib/exportOptions';
import { componentFullName } from '../lib/libraryDrag';
import { loadUserLibraries, persistUserLibrariesBlob } from '../lib/userLibraries';
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
	#persistTimer: ReturnType<typeof setTimeout> | null = null;
	#persistEnabled = false;
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
	savedSnapshot = '';
	pendingDiscard: (() => void) | null = null;
	#libraryDrag = new LibraryDragSession(this);

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
	};

	afterChange = () => {
		this.engine?.mutate(() => {});
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
		const initWasm = (await import('../wasm/fidocad_wasm.js')).default;
		const { App } = await import('../wasm/fidocad_wasm.js');
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
		this.#persistEnabled = true;
		if (this.engine) {
			let libsRev = this.engine.libsRev;
			this.engine.onRefresh = () => {
				if (this.engine && this.engine.libsRev !== libsRev) {
					libsRev = this.engine.libsRev;
					this.cursorCache.clear();
					this.previewCache.clear();
				}
				this.schedulePersist();
			};
		}
		window.addEventListener('pagehide', this.flushPersist);
		document.addEventListener('visibilitychange', this.onVisibilityChange);
		this.schedulePersist();
	};

	restoreSession = (session: SessionState) => {
		if (!this.engine) return;
		try {
			this.engine.mutate((app) => {
				app.load_fcd(session.fcd);
				app.set_view(session.zoom, session.panX, session.panY);
				app.set_tool(
					session.tool === 'component' || session.tool === 'macro' ? 'select' : session.tool
				);
				app.set_layer(session.layer);
				if (!fcdHasProjectSettings(session.fcd)) {
					app.set_snap_enable(session.snapEnable);
					app.set_show_grid(session.showGrid);
					app.set_hide_component_origin(session.hideComponentOrigin);
				}
			});
			this.fileHandleName = session.name;
			this.savedSnapshot = session.savedFcd;
		} catch (err) {
			this.error = String(err);
			this.markClean();
		}
	};

	schedulePersist = () => {
		if (!this.#persistEnabled) return;
		if (this.#persistTimer) clearTimeout(this.#persistTimer);
		this.#persistTimer = setTimeout(() => {
			this.#persistTimer = null;
			this.persistNow();
		}, SESSION_DEBOUNCE_MS);
	};

	flushPersist = () => {
		if (this.#persistTimer) {
			clearTimeout(this.#persistTimer);
			this.#persistTimer = null;
		}
		this.persistNow();
	};

	onVisibilityChange = () => {
		if (document.visibilityState === 'hidden') this.flushPersist();
	};

	persistNow = () => {
		if (!this.#persistEnabled || !this.engine) return;
		const status = this.status;
		saveSession({
			version: 1,
			fcd: this.engine.query((app) => app.save_fcd()),
			name: this.fileHandleName,
			savedFcd: this.savedSnapshot,
			zoom: status.zoom,
			panX: status.pan_x,
			panY: status.pan_y,
			tool: status.tool,
			layer: status.layer,
			snapEnable: status.snap_enable,
			showGrid: status.show_grid,
			hideComponentOrigin: status.hide_component_origin,
			theme: this.theme,
			locale: this.locale
		});
		persistUserLibrariesBlob(this.engine.query((app) => app.user_libraries_blob()));
	};

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
	confirmExport = (opts: ExportPreviewOpts, svg: string) => files.confirmExport(this, opts, svg);
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

	pickComponent = (stem: string, key: string) => {
		const name = componentFullName(stem, key);
		this.engine?.mutate((app) => {
			app.set_pending_component(name);
		});
	};

	revealLibraryItem = (stem: string, key: string) => {
		this.rightCollapsed = false;
		this.rightTab = 'library';
		this.expandedUserLibs = { ...this.expandedUserLibs, [stem]: true };
		this.libraryFocus = { stem, key };
	};

	createComponentFromSelection = (target: string) => {
		if (!this.engine) return;
		const created = { stem: '', key: '' };
		this.engine.mutate((app) => {
			const json = app.create_component_from_selection(target, this.t.newComponent);
			if (!json) return;
			try {
				const parsed = JSON.parse(json) as { stem?: string; key?: string };
				if (parsed.stem && parsed.key) {
					created.stem = parsed.stem;
					created.key = parsed.key;
				}
			} catch {
				/* ignore */
			}
		});
		if (created.stem && created.key) {
			this.revealLibraryItem(created.stem, created.key);
			const warn = this.engine.query((app) =>
				app.local_component_uses_nonzero_layers(created.stem, created.key)
			);
			if (warn) this.dialogs.open({ kind: 'componentLayerWarning' });
		}
	};

	enterComponentEdit = (stem: string, key: string) => {
		this.ui.ctxMenu = null;
		this.rightTab = 'library';
		this.engine?.mutate((app) => {
			app.enter_component_edit(stem, key);
		});
	};

	editSelectedComponent = () => {
		this.rightTab = 'library';
		this.engine?.mutate((app) => {
			app.edit_selected_component();
		});
	};

	saveComponentEdit = () => {
		if (!this.engine) return;
		const warn = this.engine.query((app) =>
			app.editing_local_component_uses_nonzero_layers()
		);
		this.engine.mutate((app) => {
			app.save_component_edit();
		});
		if (warn) this.dialogs.open({ kind: 'componentLayerWarning' });
	};

	cancelComponentEdit = () => {
		this.engine?.mutate((app) => {
			app.cancel_component_edit();
		});
	};

	beginRenameComponent = (stem: string, key: string) => {
		this.ui.ctxMenu = null;
		this.ui.editingLibraryField = { stem, key, field: 'name' };
		this.ui.libraryFocus = { stem, key };
	};

	beginEditComponentDescription = (stem: string, key: string) => {
		this.ui.ctxMenu = null;
		this.ui.editingLibraryField = { stem, key, field: 'description' };
		this.ui.libraryFocus = { stem, key };
	};

	renameComponent = (stem: string, key: string, name: string) => {
		this.engine?.mutate((app) => {
			app.rename_component(stem, key, name);
		});
		this.editingLibraryField = null;
	};

	setComponentDescription = (stem: string, key: string, description: string) => {
		this.engine?.mutate((app) => {
			app.set_component_description(stem, key, description);
		});
		this.editingLibraryField = null;
	};

	moveComponent = (stem: string, key: string, destStem: string) => {
		let destKey = key;
		this.engine?.mutate((app) => {
			const moved = app.move_component(stem, key, destStem);
			if (moved) destKey = moved;
		});
		this.revealLibraryItem(destStem, destKey);
	};

	requestDeleteComponent = (stem: string, key: string) => {
		this.ui.ctxMenu = null;
		this.dialogs.open({ kind: 'deleteComponent', stem, key });
	};

	confirmDeleteComponent = (stem: string, key: string) => {
		this.dialogs.close();
		this.engine?.mutate((app) => {
			app.delete_component(stem, key);
		});
		if (this.libraryFocus?.stem === stem && this.libraryFocus?.key === key) {
			this.libraryFocus = null;
		}
	};

	cancelDeleteComponent = () => {
		if (this.dialogs.dialog?.kind === 'deleteComponent') this.dialogs.close();
	};

	createUserLibrary = () => {
		this.ui.ctxMenu = null;
		this.ui.closeMenu();
		let stem = '';
		this.engine?.mutate((app) => {
			stem = app.create_user_library(this.t.newLibrary);
		});
		if (!stem) return;
		this.rightTab = 'library';
		this.expandedUserLibs = { ...this.expandedUserLibs, [stem]: true };
		this.beginRenameLibrary(stem);
	};

	beginRenameLibrary = (stem: string) => {
		if (stem === 'project') return;
		this.ui.ctxMenu = null;
		this.ui.editingLibraryTitle = stem;
	};

	renameLibrary = (stem: string, title: string) => {
		let next = stem;
		this.engine?.mutate((app) => {
			const renamed = app.rename_library(stem, title);
			if (renamed) next = renamed;
		});
		this.editingLibraryTitle = null;
		if (next !== stem) {
			const expanded = { ...this.expandedUserLibs };
			expanded[next] = expanded[stem] ?? true;
			delete expanded[stem];
			this.expandedUserLibs = expanded;
		}
	};

	requestDeleteLibrary = (stem: string) => {
		this.ui.ctxMenu = null;
		this.dialogs.open({ kind: 'deleteLibrary', stem });
	};

	confirmDeleteLibrary = (stem: string) => {
		this.dialogs.close();
		this.engine?.mutate((app) => {
			if (stem === 'project') app.clear_project_library();
			else app.remove_user_library(stem);
		});
		if (stem !== 'project') {
			const expanded = { ...this.expandedUserLibs };
			delete expanded[stem];
			this.expandedUserLibs = expanded;
		}
	};

	cancelDeleteLibrary = () => {
		if (this.dialogs.dialog?.kind === 'deleteLibrary') this.dialogs.close();
	};

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

function fcdHasProjectSettings(text: string): boolean {
	return /(^|[\r\n])[ \t]*PS(?:\s|$)/i.test(text);
}
