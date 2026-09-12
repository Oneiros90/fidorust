import { componentFullName } from '../lib/libraryDrag';
import type { AppSession } from './appSession.svelte';

export function pickComponent(s: AppSession, stem: string, key: string) {
	const name = componentFullName(stem, key);
	s.engine?.mutate((app) => {
		app.set_pending_component(name);
	});
}

export function revealLibraryItem(s: AppSession, stem: string, key: string) {
	s.rightCollapsed = false;
	s.rightTab = 'library';
	s.expandedUserLibs = { ...s.expandedUserLibs, [stem]: true };
	s.libraryFocus = { stem, key };
}

export function createComponentFromSelection(s: AppSession, target: string) {
	if (!s.engine) return;
	const created = { stem: '', key: '' };
	s.engine.mutate((app) => {
		const json = app.create_component_from_selection(target, s.t.newComponent);
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
		revealLibraryItem(s, created.stem, created.key);
		const warn = s.engine.query((app) =>
			app.local_component_uses_nonzero_layers(created.stem, created.key)
		);
		if (warn) s.dialogs.open({ kind: 'componentLayerWarning' });
	}
}

export function enterComponentEdit(s: AppSession, stem: string, key: string) {
	s.ui.ctxMenu = null;
	s.rightTab = 'library';
	s.engine?.mutate((app) => {
		app.enter_component_edit(stem, key);
	});
}

export function editSelectedComponent(s: AppSession) {
	s.rightTab = 'library';
	s.engine?.mutate((app) => {
		app.edit_selected_component();
	});
}

export function saveComponentEdit(s: AppSession) {
	if (!s.engine) return;
	const warn = s.engine.query((app) => app.editing_local_component_uses_nonzero_layers());
	s.engine.mutate((app) => {
		app.save_component_edit();
	});
	if (warn) s.dialogs.open({ kind: 'componentLayerWarning' });
}

export function cancelComponentEdit(s: AppSession) {
	s.engine?.mutate((app) => {
		app.cancel_component_edit();
	});
}

export function beginRenameComponent(s: AppSession, stem: string, key: string) {
	s.ui.ctxMenu = null;
	s.ui.editingLibraryField = { stem, key, field: 'name' };
	s.ui.libraryFocus = { stem, key };
}

export function beginEditComponentKey(s: AppSession, stem: string, key: string) {
	s.ui.ctxMenu = null;
	s.ui.editingLibraryField = { stem, key, field: 'key' };
	s.ui.libraryFocus = { stem, key };
}

export function renameComponent(s: AppSession, stem: string, key: string, name: string) {
	s.engine?.mutate((app) => {
		app.rename_component(stem, key, name);
	});
	s.editingLibraryField = null;
}

export function renameComponentKey(s: AppSession, stem: string, key: string, newKey: string) {
	let ok = false;
	s.engine?.mutate((app) => {
		ok = app.rename_component_key(stem, key, newKey);
	});
	s.editingLibraryField = null;
	if (ok) s.libraryFocus = { stem, key: newKey.trim() };
}

export function moveComponent(s: AppSession, stem: string, key: string, destStem: string) {
	let destKey = key;
	s.engine?.mutate((app) => {
		const moved = app.move_component(stem, key, destStem);
		if (moved) destKey = moved;
	});
	revealLibraryItem(s, destStem, destKey);
}

export function requestDeleteComponent(s: AppSession, stem: string, key: string) {
	s.ui.ctxMenu = null;
	s.dialogs.open({ kind: 'deleteComponent', stem, key });
}

export function confirmDeleteComponent(s: AppSession, stem: string, key: string) {
	s.dialogs.close();
	s.engine?.mutate((app) => {
		app.delete_component(stem, key);
	});
	if (s.libraryFocus?.stem === stem && s.libraryFocus?.key === key) {
		s.libraryFocus = null;
	}
}

export function cancelDeleteComponent(s: AppSession) {
	if (s.dialogs.dialog?.kind === 'deleteComponent') s.dialogs.close();
}

export function createUserLibrary(s: AppSession) {
	s.ui.ctxMenu = null;
	s.ui.closeMenu();
	let stem = '';
	s.engine?.mutate((app) => {
		stem = app.create_user_library(s.t.newLibrary);
	});
	if (!stem) return;
	s.rightTab = 'library';
	s.expandedUserLibs = { ...s.expandedUserLibs, [stem]: true };
	beginRenameLibrary(s, stem);
}

export function beginRenameLibrary(s: AppSession, stem: string) {
	if (stem === 'project') return;
	s.ui.ctxMenu = null;
	s.ui.editingLibraryTitle = stem;
}

export function renameLibrary(s: AppSession, stem: string, title: string) {
	let next = stem;
	s.engine?.mutate((app) => {
		const renamed = app.rename_library(stem, title);
		if (renamed) next = renamed;
	});
	s.editingLibraryTitle = null;
	if (next !== stem) {
		const expanded = { ...s.expandedUserLibs };
		expanded[next] = expanded[stem] ?? true;
		delete expanded[stem];
		s.expandedUserLibs = expanded;
	}
}

export function requestDeleteLibrary(s: AppSession, stem: string) {
	s.ui.ctxMenu = null;
	s.dialogs.open({ kind: 'deleteLibrary', stem });
}

export function confirmDeleteLibrary(s: AppSession, stem: string) {
	s.dialogs.close();
	s.engine?.mutate((app) => {
		if (stem === 'project') app.clear_project_library();
		else app.remove_user_library(stem);
	});
	if (stem !== 'project') {
		const expanded = { ...s.expandedUserLibs };
		delete expanded[stem];
		s.expandedUserLibs = expanded;
	}
}

export function cancelDeleteLibrary(s: AppSession) {
	if (s.dialogs.dialog?.kind === 'deleteLibrary') s.dialogs.close();
}
