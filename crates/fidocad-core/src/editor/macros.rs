//! Macro placement, split, paste, and context-menu helpers.

use super::{Drag, Editor, Tool};
use crate::geom::{Point, Transform};
use crate::hit::hit_test;
use crate::primitive::{MacroRef, Primitive};

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

    pub fn split_selected_macros(&mut self) {
        let set: std::collections::HashSet<usize> = self.selected.iter().copied().collect();
        let has_macro = set
            .iter()
            .any(|&i| self.doc.primitives.get(i).is_some_and(|p| p.is_macro()));
        if !has_macro {
            return;
        }
        self.push_undo();
        let mut out = Vec::new();
        let mut new_sel = Vec::new();
        for (i, p) in self.doc.primitives.iter().enumerate() {
            if set.contains(&i) {
                if p.is_macro() {
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
        if self.tool == Tool::Macro && self.pending_macro.is_some() {
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

    pub fn pending_macro_preview(&self) -> Vec<Primitive> {
        if self.tool != Tool::Macro {
            return Vec::new();
        }
        let Some(name) = self.pending_macro.as_deref() else {
            return Vec::new();
        };
        let Some(pos) = self.hover else {
            return Vec::new();
        };
        let Some((_, def)) = self.libs.lookup(name) else {
            return Vec::new();
        };
        crate::library::expand_macro(
            def,
            Transform {
                origin: pos,
                rotations: self.pending_rotations,
                mirrored: false,
            },
            &self.libs,
            0,
        )
    }

    pub fn clear_hover(&mut self) {
        self.hover = None;
    }

    pub fn insert_pending_macro_at(&mut self, world: Point) {
        let Some(name) = self.pending_macro.clone() else {
            return;
        };
        self.push_undo();
        let pt = self.snap_pt(world);
        let standard = self.libs.is_standard(&name);
        self.doc.insert(Primitive::Macro(MacroRef {
            pos: pt,
            rotations: self.pending_rotations,
            mirrored: false,
            name,
            standard,
        }));
    }

    pub fn set_pending_macro(&mut self, name: Option<String>) {
        self.pending_macro = name;
        self.pending_rotations = 0;
    }
}
