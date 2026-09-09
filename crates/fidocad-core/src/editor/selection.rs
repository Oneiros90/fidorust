//! Selection transforms.

use super::{Drag, Editor};
use crate::geom::Point;
use crate::layers::LayerId;
use crate::library::{expand_primitive, translate_primitives};
use crate::primitive::{ComponentRef, Primitive};

impl Editor {
    pub(super) fn translate_selected(&mut self, delta: Point) {
        if delta == Point::new(0, 0) {
            return;
        }
        let sel = self.selected.clone();
        for i in sel {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                p.transform(|q| q + delta);
            }
        }
    }

    fn clone_selected(&self) -> Vec<Primitive> {
        self.selected
            .iter()
            .filter_map(|&i| self.doc.primitives.get(i).cloned())
            .collect()
    }

    pub(super) fn insert_translated_clones(&mut self, dx: i32, dy: i32) {
        let mut clones = self.clone_selected();
        if clones.is_empty() {
            return;
        }
        translate_primitives(&mut clones, dx, dy);
        self.selected.clear();
        for p in clones {
            let i = self.doc.insert(p);
            self.selected.push(i);
        }
    }

    fn insert_clones_keeping_selection(&mut self, dx: i32, dy: i32) {
        let mut clones = self.clone_selected();
        if clones.is_empty() {
            return;
        }
        translate_primitives(&mut clones, dx, dy);
        for p in clones {
            self.doc.insert(p);
        }
    }

    /// Stamp a copy at the dragged selection. Keeps dragging the originals.
    pub fn stamp_drag_copy(&mut self) -> bool {
        let Some(Drag::Move {
            start,
            last,
            duplicate,
        }) = &self.drag
        else {
            return false;
        };
        let (dx, dy) = if *duplicate {
            (last.x - start.x, last.y - start.y)
        } else {
            (0, 0)
        };
        if self.selected.is_empty() {
            return false;
        }
        self.insert_clones_keeping_selection(dx, dy);
        true
    }

    pub fn duplicate_drag(&self) -> bool {
        matches!(
            self.drag,
            Some(Drag::Move {
                duplicate: true,
                ..
            })
        )
    }

    pub fn set_move_duplicate(&mut self, on: bool) {
        let Some(Drag::Move {
            last,
            start,
            duplicate,
        }) = &self.drag
        else {
            return;
        };
        if *duplicate == on {
            return;
        }
        let last = *last;
        let start = *start;
        if on {
            self.translate_selected(Point::new(start.x - last.x, start.y - last.y));
        } else {
            self.translate_selected(Point::new(last.x - start.x, last.y - start.y));
        }
        if let Some(Drag::Move { duplicate, .. }) = &mut self.drag {
            *duplicate = on;
        }
    }

    pub fn duplicate_selection(&mut self) {
        if self.selected.is_empty() || self.drag.is_some() {
            return;
        }
        let Some(hover) = self.hover else {
            return;
        };
        let pt = self.snap_pt(hover);
        let bb = self.doc.selected_aabb(&self.selected, &self.libs);
        if bb.is_empty() {
            return;
        }
        self.push_undo();
        self.insert_translated_clones(pt.x - bb.min.x, pt.y - bb.min.y);
    }

    pub fn duplicate_drag_preview(&self) -> Vec<Primitive> {
        let Some(Drag::Move {
            start,
            last,
            duplicate: true,
        }) = &self.drag
        else {
            return Vec::new();
        };
        let mut prims: Vec<Primitive> = self
            .selected
            .iter()
            .filter_map(|&i| self.doc.primitives.get(i).cloned())
            .collect();
        translate_primitives(&mut prims, last.x - start.x, last.y - start.y);
        prims
            .iter()
            .flat_map(|p| expand_primitive(p, &self.libs))
            .collect()
    }

    pub fn delete_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.push_undo();
        let mut sel = self.selected.clone();
        sel.sort_unstable();
        sel.reverse();
        for i in sel {
            self.doc.remove(i);
        }
        self.selected.clear();
    }

    pub fn select_all(&mut self) {
        self.selected = (0..self.doc.primitives.len()).collect();
    }

    pub fn rotate_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.push_undo();
        let origin = {
            let bb = self.doc.selected_aabb(&self.selected, &self.libs);
            Point::new((bb.min.x + bb.max.x) / 2, (bb.min.y + bb.max.y) / 2)
        };
        self.rotate_at(origin);
    }

    pub(super) fn rotate_at(&mut self, origin: Point) {
        for &i in &self.selected {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                p.transform(|q| q.rotate90_cw(origin));
                if let Primitive::Component(ComponentRef { rotations, .. }) = p {
                    *rotations = (*rotations + 1) % 4;
                }
            }
        }
    }

    pub fn invert_selection(&mut self) {
        let n = self.doc.primitives.len();
        let sel: std::collections::HashSet<usize> = self.selected.iter().copied().collect();
        self.selected = (0..n).filter(|i| !sel.contains(i)).collect();
    }

    pub fn mirror_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.push_undo();
        let origin = {
            let bb = self.doc.selected_aabb(&self.selected, &self.libs);
            (bb.min.x + bb.max.x) / 2
        };
        for &i in &self.selected {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                p.transform(|q| q.mirror_vertical(origin));
                if let Primitive::Component(ComponentRef { mirrored, .. }) = p {
                    *mirrored = !*mirrored;
                }
            }
        }
    }

    pub fn set_selected_layer(&mut self, layer: LayerId) {
        let layer = self.doc.layers.clamp_id(layer);
        self.push_undo();
        for &i in &self.selected {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                p.set_layer(layer);
            }
        }
    }
}
