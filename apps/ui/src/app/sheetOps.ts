import type { AppSession } from './appSession.svelte';
import type { DrawingDefaultsValues, SheetSettingsValues } from './appSession.svelte';

export function hoverPane(s: AppSession, pane: number) {
	if ((s.status.active_pane ?? 0) === pane && (s.status.split || pane === 0)) return;
	s.engine?.mutate((app) => {
		app.hover_pane(pane);
	});
}

export function setSplit(s: AppSession, on: boolean) {
	if (s.status.editing_component) return;
	s.engine?.mutate((app) => {
		app.set_split(on);
	});
}

export function toggleSplit(s: AppSession) {
	setSplit(s, !s.status.split);
}

export function selectSheet(s: AppSession, pane: number, index: number) {
	s.engine?.mutate((app) => {
		app.hover_pane(pane);
		app.set_pane_sheet(pane, index);
	});
}

export function addSheet(s: AppSession, pane: number) {
	s.engine?.mutate((app) => {
		app.add_sheet_on(pane);
	});
}

export function duplicateSheet(s: AppSession, pane: number, index: number) {
	s.ui.ctxMenu = null;
	s.engine?.mutate((app) => {
		app.duplicate_sheet_on(pane, index);
	});
}

export function renameSheet(s: AppSession, index: number, name: string) {
	s.engine?.mutate((app) => {
		app.rename_sheet(index, name);
	});
	s.ui.editingSheetName = null;
}

export function beginRenameSheet(s: AppSession, pane: number, index: number) {
	s.ui.ctxMenu = null;
	s.ui.editingSheetName = { pane, index };
}

export function reorderSheets(s: AppSession, from: number, to: number) {
	if (from === to) return;
	s.engine?.mutate((app) => {
		app.reorder_sheets(from, to);
	});
}

export function requestDeleteSheet(s: AppSession, index: number) {
	s.ui.ctxMenu = null;
	if (!s.engine) return;
	if ((s.status.sheets?.length ?? 1) <= 1) return;
	const empty = s.engine.query((app) => app.sheet_is_empty(index));
	if (empty) {
		s.engine.mutate((app) => {
			app.delete_sheet(index);
		});
		return;
	}
	s.dialogs.open({ kind: 'deleteSheet', index });
}

export function confirmDeleteSheet(s: AppSession) {
	const d = s.dialogs.dialog;
	s.dialogs.close();
	if (d?.kind !== 'deleteSheet') return;
	s.engine?.mutate((app) => {
		app.delete_sheet(d.index);
	});
}

export function cancelDeleteSheet(s: AppSession) {
	if (s.dialogs.dialog?.kind === 'deleteSheet') s.dialogs.close();
}

export function openSheetSettings(s: AppSession, pane: number, index: number) {
	s.ui.ctxMenu = null;
	s.engine?.mutate((app) => {
		app.hover_pane(pane);
		app.set_pane_sheet(pane, index);
	});
	s.dialogs.open({ kind: 'sheetSettings' });
}

export function openSheetContextMenu(
	s: AppSession,
	x: number,
	y: number,
	pane: number,
	index: number
) {
	s.ui.menu = null;
	s.ui.editingSheetName = null;
	s.ui.ctxMenu = { kind: 'sheet', x, y, pane, index };
	hoverPane(s, pane);
}

export function applySheetSettings(s: AppSession, v: SheetSettingsValues) {
	s.engine?.mutate((app) => {
		app.apply_project_settings(
			JSON.stringify({
				grid: v.gridX,
				grid_y: v.gridY,
				snap: v.snapX,
				snap_y: v.snapY,
				show_grid: v.showGrid,
				snap_enable: v.snapEnable,
				hide_component_origin: s.status.hide_component_origin,
				stroke_hundredths: s.status.stroke_hundredths,
				default_filled: s.status.default_filled
			})
		);
	});
	s.dialogs.close();
}

export function applyDrawingDefaults(s: AppSession, v: DrawingDefaultsValues) {
	s.engine?.mutate((app) => {
		app.apply_drawing_defaults(v.hideComponentOrigin, v.strokeHundredths, v.defaultFilled);
	});
	s.dialogs.close();
}
