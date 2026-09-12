//! Create, rename, delete, and move component definitions.

use super::is_valid_component_key;
use super::Editor;
use crate::library::{
    component_full_name, explode_named_everywhere, rewrite_component_names,
    rewrite_component_names_in_libs, translate_primitives, ComponentDef,
};
use crate::primitive::{ComponentRef, Primitive};
use crate::COMPONENT_ORIGIN;

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
        self.push_undo();
        let old_full = component_full_name(stem, key);
        let new_full = component_full_name(stem, new_key);
        if let Some(def) = self.libs.library_mut(stem).and_then(|l| l.find_mut(key)) {
            def.key = new_key.to_string();
        } else {
            return false;
        }
        if old_full != new_full {
            rewrite_component_names(&mut self.doc.primitives, &old_full, &new_full);
            rewrite_component_names_in_libs(&mut self.libs, &old_full, &new_full);
            if let Some(session) = &mut self.component_edit {
                rewrite_component_names(&mut session.saved_doc.primitives, &old_full, &new_full);
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
}
