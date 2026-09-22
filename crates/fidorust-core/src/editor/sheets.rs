//! Multi-sheet and dual-pane view state.

use super::{Editor, EditorError, PaneState};
use crate::consts::FIT_MARGIN;
use crate::document::{default_sheet_name, Document};
use crate::library::{
    component_full_name, max_used_layer_index, rewrite_component_names_map, ComponentDef, Library,
    PROJECT_STEM,
};
use std::collections::{HashMap, HashSet};

impl Editor {
    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = if locale.eq_ignore_ascii_case("en") {
            "en".into()
        } else {
            "it".into()
        };
        if self.doc.sheets.len() == 1
            && (self.doc.sheets[0].name == crate::document::DEFAULT_SHEET_NAME
                || self.doc.sheets[0].name == "Sheet 1")
            && self.doc.sheets[0].primitives.is_empty()
        {
            self.doc.sheets[0].name = default_sheet_name(&self.locale, 0);
        }
    }

    pub fn split(&self) -> bool {
        self.split && self.component_edit.is_none()
    }

    pub fn set_split(&mut self, on: bool) {
        if self.component_edit.is_some() {
            self.split = false;
            self.set_active_pane(0);
            return;
        }
        if on == self.split {
            return;
        }
        self.store_active_pane();
        self.split = on;
        if on {
            self.panes[1].sheet_index = self.panes[0].sheet_index;
            self.panes[1].zoom = self.panes[0].zoom;
            self.panes[1].pan = self.panes[0].pan;
            self.panes[1].selected.clear();
            self.panes[1].hover = None;
            self.panes[1].hover_hit = false;
            self.panes[1].hover_index = None;
        } else {
            self.set_active_pane(0);
        }
    }

    pub fn active_pane(&self) -> usize {
        if self.split {
            self.active_pane.min(1)
        } else {
            0
        }
    }

    pub fn pane(&self, i: usize) -> &PaneState {
        &self.panes[i.min(1)]
    }

    pub fn panes(&self) -> &[PaneState; 2] {
        &self.panes
    }

    pub fn store_active_pane(&mut self) {
        let i = self.active_pane();
        self.panes[i].zoom = self.zoom;
        self.panes[i].pan = self.pan;
        self.panes[i].selected = self.selected.clone();
        self.panes[i].hover = self.hover;
        self.panes[i].hover_hit = self.hover_hit;
        self.panes[i].hover_index = self.hover_index;
        self.panes[i].sheet_index = self.doc.view_index();
    }

    pub(super) fn restore_active_pane(&mut self) {
        let i = self.active_pane();
        let pane = self.panes[i].clone();
        self.zoom = pane.zoom;
        self.pan = pane.pan;
        self.selected = pane.selected;
        self.hover = pane.hover;
        self.hover_hit = pane.hover_hit;
        self.hover_index = pane.hover_index;
        self.sync_view_sheet();
    }

    pub fn set_active_pane(&mut self, pane: usize) {
        let pane = if self.split { pane.min(1) } else { 0 };
        if pane == self.active_pane && self.doc.view_index() == self.panes[pane].sheet_index {
            return;
        }
        self.store_active_pane();
        self.active_pane = pane;
        self.restore_active_pane();
    }

    pub fn sync_view_sheet(&mut self) {
        let i = self.panes[self.active_pane()].sheet_index;
        self.doc.set_view_index(i);
    }

    pub fn set_pane_sheet(&mut self, pane: usize, sheet: usize) {
        if sheet >= self.doc.sheets.len() {
            return;
        }
        let pane = if self.split { pane.min(1) } else { 0 };
        self.store_active_pane();
        self.panes[pane].sheet_index = sheet;
        self.panes[pane].selected.clear();
        self.panes[pane].hover = None;
        self.panes[pane].hover_hit = false;
        self.panes[pane].hover_index = None;
        if pane == self.active_pane() {
            self.restore_active_pane();
        }
    }

    pub fn sheet_names(&self) -> Vec<String> {
        self.doc.sheets.iter().map(|s| s.name.clone()).collect()
    }

    pub fn sheet_count(&self) -> usize {
        self.doc.sheets.len()
    }

    pub fn set_pane_view(&mut self, pane: usize, zoom: f32, pan: (f32, f32)) {
        let pane = if self.split { pane.min(1) } else { 0 };
        self.store_active_pane();
        self.panes[pane].zoom = zoom;
        self.panes[pane].pan = pan;
        if pane == self.active_pane() {
            self.zoom = zoom;
            self.pan = pan;
        }
    }

    pub fn add_sheet(&mut self) -> usize {
        self.add_sheet_on(self.active_pane())
    }

    pub fn add_sheet_on(&mut self, pane: usize) -> usize {
        if self.component_edit.is_some() {
            return self.doc.view_index();
        }
        self.store_active_pane();
        self.push_project_undo();
        let name = default_sheet_name(&self.locale, self.doc.sheets.len());
        let idx = self.doc.add_sheet(&name);
        if let Some(src) = self.doc.sheet(self.doc.view_index()) {
            let grid = (
                src.grid,
                src.grid_y,
                src.snap,
                src.snap_y,
                src.show_grid,
                src.snap_enable,
            );
            if let Some(sheet) = self.doc.sheet_mut(idx) {
                sheet.grid = grid.0;
                sheet.grid_y = grid.1;
                sheet.snap = grid.2;
                sheet.snap_y = grid.3;
                sheet.show_grid = grid.4;
                sheet.snap_enable = grid.5;
            }
        }
        let pane = if self.split { pane.min(1) } else { 0 };
        self.active_pane = pane;
        self.panes[pane].sheet_index = idx;
        self.panes[pane].selected.clear();
        self.panes[pane].zoom = 4.0;
        self.panes[pane].pan = (FIT_MARGIN, FIT_MARGIN);
        self.restore_active_pane();
        idx
    }

    pub fn duplicate_sheet(&mut self, index: usize) -> Option<usize> {
        self.duplicate_sheet_on(self.active_pane(), index)
    }

    pub fn duplicate_sheet_on(&mut self, pane: usize, index: usize) -> Option<usize> {
        if self.component_edit.is_some() || index >= self.doc.sheets.len() {
            return None;
        }
        self.store_active_pane();
        self.push_project_undo();
        let idx = self.doc.duplicate_sheet(index, &self.locale)?;
        let pane = if self.split { pane.min(1) } else { 0 };
        self.active_pane = pane;
        self.panes[pane].sheet_index = idx;
        self.panes[pane].selected.clear();
        self.restore_active_pane();
        Some(idx)
    }

    pub fn rename_sheet(&mut self, index: usize, name: &str) -> bool {
        if self.component_edit.is_some() {
            return false;
        }
        let next = crate::document::sanitize_sheet_name(name);
        if next.is_empty()
            || index >= self.doc.sheets.len()
            || self.doc.sheet_name_taken(&next, Some(index))
        {
            return false;
        }
        if self.doc.sheets[index].name == next {
            return true;
        }
        self.push_project_undo();
        self.doc.rename_sheet(index, &next)
    }

    pub fn delete_sheet(&mut self, index: usize) -> bool {
        if self.component_edit.is_some()
            || self.doc.sheets.len() <= 1
            || index >= self.doc.sheets.len()
        {
            return false;
        }
        self.store_active_pane();
        self.push_project_undo();
        if !self.doc.remove_sheet(index) {
            return false;
        }
        for pane in &mut self.panes {
            pane.sheet_index = remap_after_remove(pane.sheet_index, index, self.doc.sheets.len());
            if pane.sheet_index == index {
                pane.selected.clear();
            }
        }
        self.restore_active_pane();
        true
    }

    pub fn reorder_sheets(&mut self, from: usize, to: usize) -> bool {
        if self.component_edit.is_some() || from == to {
            return false;
        }
        self.store_active_pane();
        self.push_project_undo();
        if !self.doc.reorder_sheets(from, to) {
            return false;
        }
        for pane in &mut self.panes {
            pane.sheet_index = remap_reorder(pane.sheet_index, from, to);
        }
        self.restore_active_pane();
        true
    }

    pub fn sheet_is_empty(&self, index: usize) -> bool {
        self.doc
            .sheet(index)
            .is_none_or(|s| s.primitives.is_empty())
    }

    pub(super) fn clamp_pane_sheets(&mut self) {
        let n = self.doc.sheets.len().saturating_sub(1);
        for pane in &mut self.panes {
            if pane.sheet_index > n {
                pane.sheet_index = n;
                pane.selected.clear();
            }
        }
        self.sync_view_sheet();
    }

    pub fn isolate_active_sheet_doc(&self) -> Document {
        self.doc.isolate_sheet(self.doc.view_index())
    }

    /// Append every sheet from `text` into this project, uniquifying names, and merge
    /// the incoming `[FIDOLIB project]` into the current project library.
    pub fn import_fcd_sheets(&mut self, text: &str) -> Result<usize, EditorError> {
        if self.component_edit.is_some() {
            return Err(EditorError::EditingComponent);
        }
        let (mut incoming, incoming_project) =
            crate::parse::parse_document_with_project_library(text)?;
        if incoming.sheets.is_empty() {
            return Ok(self.doc.view_index());
        }
        self.store_active_pane();
        self.push_project_undo();
        self.libs.ensure_user_libraries();

        let mut incoming_defs = incoming_project
            .map(|lib| lib.components)
            .unwrap_or_default();
        if !incoming_defs.is_empty() {
            merge_project_components(&mut self.libs, &mut incoming.sheets, &mut incoming_defs);
            self.bump_libs_rev();
        }

        let mut max_layer = 0;
        for sheet in &incoming.sheets {
            max_layer = max_layer.max(max_used_layer_index(&sheet.primitives, &self.libs));
        }
        if let Some(project) = self.libs.project() {
            for def in &project.components {
                max_layer = max_layer.max(max_used_layer_index(&def.primitives, &self.libs));
            }
        }
        self.doc.layers.ensure_len(max_layer + 1);

        let first_idx = self.doc.sheets.len();
        for mut sheet in incoming.sheets {
            sheet.name = self.doc.unique_sheet_name(&sheet.name);
            self.doc.sheets.push(sheet);
        }

        let pane = self.active_pane();
        self.active_pane = pane;
        self.panes[pane].sheet_index = first_idx;
        self.panes[pane].selected.clear();
        self.panes[pane].hover = None;
        self.panes[pane].hover_hit = false;
        self.panes[pane].hover_index = None;
        self.restore_active_pane();
        Ok(first_idx)
    }
}

fn merge_project_components(
    libs: &mut crate::library::LibrarySet,
    sheets: &mut [crate::document::Sheet],
    incoming_defs: &mut Vec<ComponentDef>,
) {
    let project = libs
        .project()
        .cloned()
        .unwrap_or_else(Library::empty_project);
    let new_keys = allocate_import_keys(&project, incoming_defs);
    let mut taken_names: HashSet<String> =
        project.components.iter().map(|c| c.name.clone()).collect();
    let mut map = HashMap::new();
    for (def, new_key) in incoming_defs.iter_mut().zip(new_keys) {
        let old_full = component_full_name(PROJECT_STEM, &def.key);
        let new_full = component_full_name(PROJECT_STEM, &new_key);
        if !old_full.eq_ignore_ascii_case(&new_full) {
            map.insert(old_full.to_ascii_lowercase(), new_full);
        }
        def.key = new_key;
        let name = unique_import_name(&taken_names, &def.name);
        taken_names.insert(name.clone());
        def.name = name;
    }
    for sheet in sheets.iter_mut() {
        rewrite_component_names_map(&mut sheet.primitives, &map);
    }
    for def in incoming_defs.iter_mut() {
        rewrite_component_names_map(&mut def.primitives, &map);
    }
    if let Some(project) = libs.project_mut() {
        project.components.append(incoming_defs);
    }
}

fn allocate_import_keys(dest: &Library, incoming: &[ComponentDef]) -> Vec<String> {
    let dest_taken: HashSet<String> = dest
        .components
        .iter()
        .map(|c| c.key.to_ascii_lowercase())
        .collect();
    let mut used = dest_taken.clone();
    for def in incoming {
        let k = def.key.to_ascii_lowercase();
        if !dest_taken.contains(&k) {
            used.insert(k);
        }
    }
    let mut out = Vec::with_capacity(incoming.len());
    for def in incoming {
        let k = def.key.to_ascii_lowercase();
        if !dest_taken.contains(&k) {
            out.push(def.key.clone());
            continue;
        }
        let mut n = 1u32;
        loop {
            let cand = format!("C{n:02}");
            if used.insert(cand.to_ascii_lowercase()) {
                out.push(cand);
                break;
            }
            n += 1;
        }
    }
    out
}

fn unique_import_name(taken: &HashSet<String>, base: &str) -> String {
    if !taken.contains(base) {
        return base.to_string();
    }
    let mut n = 2u32;
    loop {
        let name = format!("{base} {n}");
        if !taken.contains(&name) {
            return name;
        }
        n += 1;
    }
}

fn remap_after_remove(current: usize, removed: usize, last: usize) -> usize {
    if current == removed {
        removed.min(last).saturating_sub(0).min(last)
    } else if current > removed {
        current - 1
    } else {
        current
    }
}

fn remap_reorder(current: usize, from: usize, to: usize) -> usize {
    if current == from {
        to
    } else if from < to && current > from && current <= to {
        current - 1
    } else if to < from && current >= to && current < from {
        current + 1
    } else {
        current
    }
}
