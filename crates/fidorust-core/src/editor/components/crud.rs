//! Create, rename, delete, and move component definitions.

use super::is_valid_component_key;
use super::Editor;
use crate::document::Document;
use crate::library::{
    component_full_name, embed_user_component, embed_user_components_in_primitives,
    explode_named_everywhere, rewrite_component_names, rewrite_component_names_in_libs,
    translate_primitives, ComponentDef, LibraryKind, LibrarySet,
};
use crate::primitive::{ComponentRef, Primitive};
use crate::COMPONENT_ORIGIN;

fn document_references_name(doc: &Document, name: &str) -> bool {
    doc.sheets.iter().any(|sheet| {
        sheet
            .primitives
            .iter()
            .any(|p| matches!(p, Primitive::Component(c) if c.name.eq_ignore_ascii_case(name)))
    })
}

fn project_nested_references_name(libs: &LibrarySet, name: &str) -> bool {
    libs.project().is_some_and(|project| {
        project.components.iter().any(|def| {
            def.primitives
                .iter()
                .any(|p| matches!(p, Primitive::Component(c) if c.name.eq_ignore_ascii_case(name)))
        })
    })
}

impl Editor {
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
        self.push_project_undo();
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
            category: String::new(),
            primitives: body,
        });
        self.bump_libs_rev();
        let full = component_full_name(stem, &key);
        let dest_is_local = self
            .libs
            .library(stem)
            .is_some_and(|l| l.kind == LibraryKind::Local);
        let instance_name = if dest_is_local {
            embed_user_component(&full, &mut self.libs).unwrap_or(full)
        } else {
            let mut body = self
                .libs
                .library(stem)
                .and_then(|l| l.find(&key))
                .map(|d| d.primitives.clone())
                .unwrap_or_default();
            embed_user_components_in_primitives(&mut body, &mut self.libs);
            full
        };
        remaining.push(Primitive::Component(ComponentRef {
            pos: origin,
            rotations: 0,
            mirrored: false,
            name: instance_name,
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
        self.push_project_undo();
        if let Some(def) = self.libs.library_mut(stem).and_then(|l| l.find_mut(key)) {
            def.name = name.to_string();
            self.bump_libs_rev();
            true
        } else {
            false
        }
    }

    pub fn rename_component_key(&mut self, stem: &str, key: &str, new_key: &str) -> bool {
        let new_key = new_key.trim();
        if !is_valid_component_key(new_key) {
            return false;
        }
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if !lib.writable() || lib.find(key).is_none() {
            return false;
        }
        if lib.find(key).is_some_and(|d| d.key == new_key) {
            return false;
        }
        if lib
            .components
            .iter()
            .any(|c| c.key.eq_ignore_ascii_case(new_key) && !c.key.eq_ignore_ascii_case(key))
        {
            return false;
        }
        self.push_project_undo();
        let old_full = component_full_name(stem, key);
        let new_full = component_full_name(stem, new_key);
        if let Some(def) = self.libs.library_mut(stem).and_then(|l| l.find_mut(key)) {
            def.key = new_key.to_string();
        } else {
            return false;
        }
        if old_full != new_full {
            rewrite_component_names_in_libs(&mut self.libs, &old_full, &new_full);
            self.doc.for_each_primitives_mut(|prims| {
                rewrite_component_names(prims, &old_full, &new_full);
            });
            if let Some(session) = &mut self.component_edit {
                session.saved_doc.for_each_primitives_mut(|prims| {
                    rewrite_component_names(prims, &old_full, &new_full);
                });
                if session.stem.eq_ignore_ascii_case(stem) && session.key.eq_ignore_ascii_case(key)
                {
                    session.key = new_key.to_string();
                }
                if session
                    .saved_pending
                    .as_deref()
                    .is_some_and(|p| p.eq_ignore_ascii_case(&old_full))
                {
                    session.saved_pending = Some(new_full.clone());
                }
            }
            if self
                .pending_component
                .as_deref()
                .is_some_and(|p| p.eq_ignore_ascii_case(&old_full))
            {
                self.pending_component = Some(new_full);
            }
        }
        self.bump_libs_rev();
        true
    }

    pub fn delete_component(&mut self, stem: &str, key: &str) -> bool {
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if !lib.writable() || lib.find(key).is_none() {
            return false;
        }
        self.push_project_undo();
        let full = component_full_name(stem, key);
        self.doc.for_each_primitives_mut(|prims| {
            explode_named_everywhere(prims, &mut self.libs, &full);
        });
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
        let src_kind = self.libs.library(stem)?.kind;
        let dest_kind = self.libs.library(dest_stem)?.kind;
        let def = self.libs.library(stem)?.find(key)?.clone();
        self.push_project_undo();
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
        let keep_src = src_kind == LibraryKind::Project
            && dest_kind == LibraryKind::Local
            && (document_references_name(&self.doc, &old_full)
                || project_nested_references_name(&self.libs, &old_full));
        if !keep_src {
            if let Some(src) = self.libs.library_mut(stem) {
                src.components.retain(|c| !c.key.eq_ignore_ascii_case(key));
            }
        }
        if let Some(dest) = self.libs.library_mut(dest_stem) {
            dest.components.push(moved);
        }
        if dest_kind == LibraryKind::Local && src_kind == LibraryKind::Project {
            // Catalog copy: drawing instances stay on project.*
        } else if old_full != new_full {
            self.doc.for_each_primitives_mut(|prims| {
                rewrite_component_names(prims, &old_full, &new_full);
            });
            rewrite_component_names_in_libs(&mut self.libs, &old_full, &new_full);
            if dest_kind == LibraryKind::Project {
                let mut body = self
                    .libs
                    .library(dest_stem)
                    .and_then(|l| l.find(&dest_key))
                    .map(|d| d.primitives.clone())
                    .unwrap_or_default();
                embed_user_components_in_primitives(&mut body, &mut self.libs);
            }
        }
        self.bump_libs_rev();
        Some(dest_key)
    }
}
