//! Component placement, split, library editing, and prefab-style definition editor.

use super::{history::HistorySnapshot, Drag, Editor, Tool};
use crate::geom::{Point, Transform};
use crate::hit::hit_test;
use crate::library::{
    component_full_name, drawing_uses_user_library_components, explode_library_instances,
    explode_named_everywhere, primitives_use_nonzero_layers, rewrite_component_names,
    rewrite_component_names_in_libs, translate_primitives, ComponentDef, Library, LibraryKind,
    PROJECT_STEM,
};
use crate::primitive::{ComponentRef, Primitive};
use crate::COMPONENT_ORIGIN;

#[derive(Clone, Debug)]
pub(super) struct ComponentEditSession {
    pub stem: String,
    pub key: String,
    pub saved_doc: crate::document::Document,
    pub saved_selected: Vec<usize>,
    pub saved_zoom: f32,
    pub saved_pan: (f32, f32),
    pub saved_undo: Vec<HistorySnapshot>,
    pub saved_redo: Vec<HistorySnapshot>,
    pub saved_tool: Tool,
    pub saved_pending: Option<String>,
    pub saved_layer: crate::layers::LayerId,
    pub saved_project: Library,
    pub saved_user: Vec<Library>,
    pub original_primitives: Vec<Primitive>,
}

impl Editor {
    pub fn paste_primitives(&mut self, text: &str) -> Result<(), crate::parse::ParseError> {
        let incoming = crate::parse::parse_document(text)?;
        if incoming.primitives.is_empty() {
            return Ok(());
        }
        self.push_undo();
        self.selected.clear();
        for mut p in incoming.primitives {
            p.set_layer(self.doc.layers.clamp_id(p.layer()));
            let i = self.doc.insert(p);
            self.selected.push(i);
        }
        Ok(())
    }

    pub fn split_selected_components(&mut self) {
        let set: std::collections::HashSet<usize> = self.selected.iter().copied().collect();
        let has_component = set
            .iter()
            .any(|&i| self.doc.primitives.get(i).is_some_and(|p| p.is_component()));
        if !has_component {
            return;
        }
        self.push_undo();
        let mut out = Vec::new();
        let mut new_sel = Vec::new();
        for (i, p) in self.doc.primitives.iter().enumerate() {
            if set.contains(&i) {
                if p.is_component() {
                    for q in crate::library::expand_primitive(p, &self.libs) {
                        new_sel.push(out.len());
                        out.push(q);
                    }
                    continue;
                }
                new_sel.push(out.len());
            }
            out.push(p.clone());
        }
        self.doc.primitives = out;
        self.selected = new_sel;
    }

    pub fn can_split_component(&self) -> bool {
        self.selected
            .iter()
            .any(|&i| self.doc.primitives.get(i).is_some_and(|p| p.is_component()))
    }

    pub fn can_create_component(&self) -> bool {
        !self.selected.is_empty()
    }

    /// Writable component definition shared by the current selection, if any.
    pub fn selected_editable_component(&self) -> Option<(String, String)> {
        if self.component_edit.is_some() {
            return None;
        }
        let mut found: Option<(String, String)> = None;
        for &i in &self.selected {
            let Some(Primitive::Component(c)) = self.doc.primitives.get(i) else {
                continue;
            };
            let (lib, def) = self.libs.lookup(&c.name)?;
            if !lib.writable() {
                return None;
            }
            let id = (lib.file_stem.clone(), def.key.clone());
            match &found {
                None => found = Some(id),
                Some(prev) if *prev != id => return None,
                Some(_) => {}
            }
        }
        found
    }

    pub fn can_edit_component(&self) -> bool {
        self.selected_editable_component().is_some()
    }

    /// Right-click: rotate while dragging/placing, otherwise the caller shows the context menu.
    pub fn right_click(&mut self, world: Point) -> bool {
        match &self.drag {
            Some(Drag::Move { .. }) => {
                let pt = self.snap_pt(world);
                self.hover = Some(pt);
                self.rotate_at(pt);
                return true;
            }
            Some(Drag::Marquee { .. }) => {
                self.drag = None;
                return true;
            }
            Some(Drag::Handle { .. } | Drag::Pan { .. }) => return true,
            None => {}
        }
        if self.draft.is_some() {
            self.cancel_draft();
            return true;
        }
        if self.tool == Tool::Component && self.pending_follow && self.pending_component.is_some() {
            self.pending_rotations = (self.pending_rotations + 1) % 4;
            true
        } else {
            false
        }
    }

    pub fn prepare_context_menu(&mut self, world: Point) {
        if let Some(hit) = hit_test(
            &self.doc.primitives,
            &self.libs,
            &self.doc.layers,
            world,
            self.zoom,
        ) {
            if !self.selected.contains(&hit.index) {
                self.selected.clear();
                self.selected.push(hit.index);
            }
        } else {
            self.selected.clear();
        }
    }

    pub fn pending_component_preview(&self) -> Vec<Primitive> {
        if self.tool != Tool::Component || !self.pending_follow {
            return Vec::new();
        }
        let Some(name) = self.pending_component.as_deref() else {
            return Vec::new();
        };
        let Some(pos) = self.hover else {
            return Vec::new();
        };
        let Some((_, def)) = self.libs.lookup(name) else {
            return Vec::new();
        };
        let mut prims = crate::library::expand_component(
            def,
            Transform {
                origin: pos,
                rotations: self.pending_rotations,
                mirrored: false,
            },
            &self.libs,
            0,
        );
        crate::library::paint_primitives(&mut prims, self.layer);
        prims
    }

    pub fn clear_hover(&mut self) {
        self.hover = None;
    }

    pub fn insert_pending_component_at(&mut self, world: Point) -> Option<usize> {
        let name = self.pending_component.clone()?;
        self.push_undo();
        let pt = self.snap_pt(world);
        let standard = self.libs.is_standard(&name);
        Some(self.doc.insert(Primitive::Component(ComponentRef {
            pos: pt,
            rotations: self.pending_rotations,
            mirrored: false,
            name,
            standard,
            layer: self.layer,
            use_component_layers: false,
        })))
    }

    /// Drag-and-drop from the library: place, select the instance, return to Select.
    pub fn place_dropped_component(&mut self, world: Point) {
        let Some(i) = self.insert_pending_component_at(world) else {
            return;
        };
        self.selected = vec![i];
        self.set_tool(Tool::Select);
    }

    pub fn set_pending_component(&mut self, name: Option<String>) {
        self.pending_component = name;
        self.pending_rotations = 0;
        self.pending_follow = false;
    }

    pub fn set_pending_follow(&mut self, on: bool) {
        self.pending_follow = on && self.pending_component.is_some();
    }

    pub fn create_component_from_selection(
        &mut self,
        stem: &str,
        display_name: &str,
    ) -> Option<(String, String)> {
        if self.selected.is_empty() {
            return None;
        }
        self.libs.ensure_user_libraries();
        let writable = self.libs.library(stem).is_some_and(|l| l.writable());
        if !writable {
            return None;
        }
        let bb = self.doc.selected_aabb(&self.selected, &self.libs);
        if bb.is_empty() {
            return None;
        }
        self.push_undo();
        let origin = bb.min;
        let set: std::collections::HashSet<usize> = self.selected.iter().copied().collect();
        let mut body = Vec::new();
        let mut remaining = Vec::new();
        for (i, p) in self.doc.primitives.iter().enumerate() {
            if set.contains(&i) {
                body.push(p.clone());
            } else {
                remaining.push(p.clone());
            }
        }
        translate_primitives(
            &mut body,
            COMPONENT_ORIGIN.x - origin.x,
            COMPONENT_ORIGIN.y - origin.y,
        );
        let lib = self.libs.library_mut(stem)?;
        let key = lib.next_key();
        let name = lib.unique_display_name(display_name);
        lib.components.push(ComponentDef {
            key: key.clone(),
            name,
            description: String::new(),
            category: String::new(),
            primitives: body,
        });
        self.bump_libs_rev();
        let full = component_full_name(stem, &key);
        remaining.push(Primitive::Component(ComponentRef {
            pos: origin,
            rotations: 0,
            mirrored: false,
            name: full,
            standard: false,
            layer: self.layer,
            use_component_layers: false,
        }));
        let new_index = remaining.len() - 1;
        self.doc.primitives = remaining;
        self.selected = vec![new_index];
        Some((stem.to_string(), key))
    }

    pub fn rename_component(&mut self, stem: &str, key: &str, new_name: &str) -> bool {
        let name = new_name.trim();
        if name.is_empty() {
            return false;
        }
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if !lib.writable() {
            return false;
        }
        let Some(def) = lib.find(key) else {
            return false;
        };
        if def.name == name {
            return false;
        }
        self.push_undo();
        if let Some(def) = self.libs.library_mut(stem).and_then(|l| l.find_mut(key)) {
            def.name = name.to_string();
            self.bump_libs_rev();
            true
        } else {
            false
        }
    }

    pub fn set_component_description(&mut self, stem: &str, key: &str, description: &str) -> bool {
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if !lib.writable() {
            return false;
        }
        let Some(def) = lib.find(key) else {
            return false;
        };
        if def.description == description {
            return false;
        }
        self.push_undo();
        if let Some(def) = self.libs.library_mut(stem).and_then(|l| l.find_mut(key)) {
            def.description = description.to_string();
            self.bump_libs_rev();
            true
        } else {
            false
        }
    }

    pub fn delete_component(&mut self, stem: &str, key: &str) -> bool {
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if !lib.writable() || lib.find(key).is_none() {
            return false;
        }
        self.push_undo();
        let full = component_full_name(stem, key);
        explode_named_everywhere(&mut self.doc.primitives, &mut self.libs, &full);
        if let Some(lib) = self.libs.library_mut(stem) {
            lib.components.retain(|c| !c.key.eq_ignore_ascii_case(key));
        }
        self.selected.clear();
        self.bump_libs_rev();
        true
    }

    pub fn move_component(&mut self, stem: &str, key: &str, dest_stem: &str) -> Option<String> {
        if stem.eq_ignore_ascii_case(dest_stem) {
            return None;
        }
        let src_writable = self.libs.library(stem).is_some_and(|l| l.writable());
        let dest_writable = self.libs.library(dest_stem).is_some_and(|l| l.writable());
        if !src_writable || !dest_writable {
            return None;
        }
        let def = self.libs.library(stem)?.find(key)?.clone();
        self.push_undo();
        let dest_key = {
            let dest = self.libs.library_mut(dest_stem)?;
            if dest.find(&def.key).is_none() {
                def.key.clone()
            } else {
                dest.next_key()
            }
        };
        let mut moved = def;
        let old_full = component_full_name(stem, key);
        let new_full = component_full_name(dest_stem, &dest_key);
        moved.key = dest_key.clone();
        if let Some(src) = self.libs.library_mut(stem) {
            src.components.retain(|c| !c.key.eq_ignore_ascii_case(key));
        }
        if let Some(dest) = self.libs.library_mut(dest_stem) {
            dest.components.push(moved);
        }
        if old_full != new_full {
            rewrite_component_names(&mut self.doc.primitives, &old_full, &new_full);
            rewrite_component_names_in_libs(&mut self.libs, &old_full, &new_full);
        }
        self.bump_libs_rev();
        Some(dest_key)
    }

    pub fn enter_component_edit(&mut self, stem: &str, key: &str) -> bool {
        if self.component_edit.is_some() {
            return false;
        }
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if !lib.writable() {
            return false;
        }
        let Some(def) = lib.find(key) else {
            return false;
        };
        let primitives = def.primitives.clone();
        self.cancel_draft();
        let session = ComponentEditSession {
            stem: stem.to_string(),
            key: key.to_string(),
            saved_doc: self.doc.clone(),
            saved_selected: self.selected.clone(),
            saved_zoom: self.zoom,
            saved_pan: self.pan,
            saved_undo: std::mem::take(&mut self.undo),
            saved_redo: std::mem::take(&mut self.redo),
            saved_tool: self.tool,
            saved_pending: self.pending_component.clone(),
            saved_layer: self.layer,
            saved_project: self
                .libs
                .project()
                .cloned()
                .unwrap_or_else(Library::empty_project),
            saved_user: self.libs.user_libraries_cloned(),
            original_primitives: primitives.clone(),
        };
        self.doc.primitives = primitives;
        self.selected.clear();
        self.pending_component = None;
        self.tool = Tool::Select;
        self.drag = None;
        self.draft = None;
        self.drag_checkpoint = None;
        self.component_edit = Some(session);
        self.fit_view(800.0, 600.0);
        true
    }

    pub fn save_component_edit(&mut self) -> bool {
        let Some(session) = self.component_edit.take() else {
            return false;
        };
        let primitives = self.doc.primitives.clone();
        let layers = self.doc.layers.clone();
        let layer = self.layer;
        let stem = session.stem.clone();
        let key = session.key.clone();
        self.restore_from_component_edit(session);
        self.push_undo();
        self.doc.layers = layers;
        self.layer = layer;
        self.clamp_current_layer();
        if let Some(def) = self.libs.library_mut(&stem).and_then(|l| l.find_mut(&key)) {
            def.primitives = primitives;
            self.bump_libs_rev();
            true
        } else {
            false
        }
    }

    pub fn cancel_component_edit(&mut self) -> bool {
        let Some(session) = self.component_edit.take() else {
            return false;
        };
        self.libs.set_project(session.saved_project.clone());
        self.libs.replace_user_libraries(session.saved_user.clone());
        self.bump_libs_rev();
        self.restore_from_component_edit(session);
        true
    }

    fn restore_from_component_edit(&mut self, session: ComponentEditSession) {
        self.doc = session.saved_doc;
        self.selected = session.saved_selected;
        self.zoom = session.saved_zoom;
        self.pan = session.saved_pan;
        self.undo = session.saved_undo;
        self.redo = session.saved_redo;
        self.tool = session.saved_tool;
        self.pending_component = session.saved_pending;
        self.layer = session.saved_layer;
        self.drag = None;
        self.draft = None;
        self.drag_checkpoint = None;
        self.clamp_current_layer();
    }

    pub fn editing_component(&self) -> Option<(&str, &str)> {
        self.component_edit
            .as_ref()
            .map(|s| (s.stem.as_str(), s.key.as_str()))
    }

    pub fn editing_component_name(&self) -> Option<String> {
        let (stem, key) = self.editing_component()?;
        self.libs.library(stem)?.find(key).map(|d| d.name.clone())
    }

    pub fn editing_component_dirty(&self) -> bool {
        self.component_edit
            .as_ref()
            .is_some_and(|s| s.original_primitives != self.doc.primitives || !self.undo.is_empty())
    }

    /// Document used when writing FCD (the main sheet, even while editing a definition).
    pub fn persistent_doc(&self) -> &crate::document::Document {
        self.component_edit
            .as_ref()
            .map(|s| &s.saved_doc)
            .unwrap_or(&self.doc)
    }

    pub fn load_user_libraries(&mut self, libs: Vec<Library>) {
        self.libs.replace_user_libraries(Vec::new());
        for lib in libs {
            self.libs.add_user_library(lib);
        }
        self.bump_libs_rev();
    }

    pub fn load_local_library(&mut self, lib: Library) {
        self.libs.set_local(lib);
        self.bump_libs_rev();
    }

    pub fn create_user_library(&mut self, title: &str) -> String {
        self.push_undo();
        let title = self.libs.unique_library_title(title.trim());
        let stem = self.libs.unique_stem(&title);
        self.libs.add(Library::empty_user(&stem, title));
        self.bump_libs_rev();
        stem
    }

    pub fn import_library(&mut self, lib: Library) -> String {
        self.push_undo();
        let stem = self.libs.add_user_library(lib);
        self.bump_libs_rev();
        stem
    }

    pub fn rename_library(&mut self, stem: &str, title: &str) -> Option<String> {
        let title = title.trim();
        if title.is_empty() || stem.eq_ignore_ascii_case(PROJECT_STEM) {
            return None;
        }
        let lib = self.libs.library(stem)?;
        if lib.kind != LibraryKind::Local {
            return None;
        }
        if lib.name == title {
            return None;
        }
        self.push_undo();
        if let Some(lib) = self.libs.library_mut(stem) {
            lib.name = title.to_string();
        }
        self.bump_libs_rev();
        Some(stem.to_string())
    }

    pub fn clear_project_library(&mut self) -> bool {
        let Some(project) = self.libs.project() else {
            return false;
        };
        if project.components.is_empty() {
            return false;
        }
        if self
            .component_edit
            .as_ref()
            .is_some_and(|s| s.stem.eq_ignore_ascii_case(PROJECT_STEM))
        {
            self.cancel_component_edit();
        }
        self.push_undo();
        explode_library_instances(&mut self.doc.primitives, &mut self.libs, PROJECT_STEM);
        if let Some(project) = self.libs.project_mut() {
            project.components.clear();
        }
        self.selected.clear();
        self.bump_libs_rev();
        true
    }

    pub fn remove_user_library(&mut self, stem: &str) -> bool {
        if stem.eq_ignore_ascii_case(PROJECT_STEM) {
            return false;
        }
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if lib.kind != LibraryKind::Local {
            return false;
        }
        if self
            .component_edit
            .as_ref()
            .is_some_and(|s| s.stem.eq_ignore_ascii_case(stem))
        {
            self.cancel_component_edit();
        }
        self.push_undo();
        explode_library_instances(&mut self.doc.primitives, &mut self.libs, stem);
        if let Some(pending) = &self.pending_component {
            let prefix = format!("{stem}.");
            if pending.len() > prefix.len() && pending[..prefix.len()].eq_ignore_ascii_case(&prefix)
            {
                self.pending_component = None;
            }
        }
        self.libs.remove_library(stem);
        self.selected.clear();
        self.bump_libs_rev();
        true
    }

    pub fn uses_user_library_components(&self) -> bool {
        drawing_uses_user_library_components(&self.persistent_doc().primitives, &self.libs)
    }

    pub fn local_component_uses_nonzero_layers(&self, stem: &str, key: &str) -> bool {
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if lib.kind != LibraryKind::Local {
            return false;
        }
        lib.find(key)
            .is_some_and(|d| primitives_use_nonzero_layers(&d.primitives))
    }

    pub fn editing_local_component_uses_nonzero_layers(&self) -> bool {
        let Some(session) = &self.component_edit else {
            return false;
        };
        let Some(lib) = self.libs.library(&session.stem) else {
            return false;
        };
        if lib.kind != LibraryKind::Local {
            return false;
        }
        primitives_use_nonzero_layers(&self.doc.primitives)
    }

    pub fn unresolved_component_count(&self) -> usize {
        crate::library::unresolved_component_count(&self.doc.primitives, &self.libs)
    }

    pub fn unresolved_components(&self) -> Vec<(String, usize)> {
        crate::library::unresolved_components(&self.doc.primitives, &self.libs)
    }

    pub fn set_project_library(&mut self, lib: Option<Library>) {
        self.libs
            .set_project(lib.unwrap_or_else(Library::empty_project));
        self.bump_libs_rev();
    }

    pub(super) fn bump_libs_rev(&mut self) {
        self.libs_rev = self.libs_rev.wrapping_add(1);
    }
}
