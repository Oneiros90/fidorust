//! Selection transforms.

use super::{Drag, Editor};
use crate::geom::Point;
use crate::layers::LayerId;
use crate::library::{expand_primitive, translate_primitives};
use crate::primitive::{ComponentRef, Primitive, STYLE_MIRRORED};

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
            }) | Some(Drag::PlaceClone { .. })
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
        if self.selected.is_empty() {
            return;
        }
        if self.drag.is_some() && !matches!(self.drag, Some(Drag::PlaceClone { .. })) {
            return;
        }
        let mut clones = self.clone_selected();
        if clones.is_empty() {
            return;
        }
        let bb = self.doc.selected_aabb(&self.selected, &self.libs);
        if bb.is_empty() {
            return;
        }
        let dest = self.hover.map(|p| self.snap_pt(p)).unwrap_or(bb.min);
        let offset = self.snap_delta(dest.x - bb.min.x, dest.y - bb.min.y);
        translate_primitives(&mut clones, offset.x, offset.y);
        self.drag = Some(Drag::PlaceClone {
            prims: clones,
            origin: bb.min,
            last: Point::new(bb.min.x + offset.x, bb.min.y + offset.y),
        });
    }

    pub fn duplicate_drag_preview(&self) -> Vec<Primitive> {
        match &self.drag {
            Some(Drag::PlaceClone { prims, .. }) => prims
                .iter()
                .flat_map(|p| expand_primitive(p, &self.libs))
                .collect(),
            Some(Drag::Move {
                start,
                last,
                duplicate: true,
            }) => {
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
            _ => Vec::new(),
        }
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
                rotate_primitive(p, origin);
            }
        }
    }

    pub(super) fn move_place_clone(&mut self, pt: Point) {
        let Some(Drag::PlaceClone { origin, last, .. }) = &self.drag else {
            return;
        };
        let origin = *origin;
        let last = *last;
        let new_offset = self.snap_delta(pt.x - origin.x, pt.y - origin.y);
        let delta = Point::new(
            new_offset.x - (last.x - origin.x),
            new_offset.y - (last.y - origin.y),
        );
        if delta == Point::new(0, 0) {
            return;
        }
        if let Some(Drag::PlaceClone { prims, last, .. }) = &mut self.drag {
            translate_primitives(prims, delta.x, delta.y);
            *last = Point::new(origin.x + new_offset.x, origin.y + new_offset.y);
        }
    }

    pub(super) fn rotate_place_clone(&mut self, origin: Point) {
        self.move_place_clone(origin);
        {
            let Some(Drag::PlaceClone { prims, .. }) = &mut self.drag else {
                return;
            };
            for p in prims {
                rotate_primitive(p, origin);
            }
        }
        let bb = {
            let Some(Drag::PlaceClone { prims, .. }) = &self.drag else {
                return;
            };
            let mut bb = crate::geom::Aabb::empty();
            for p in prims {
                bb.include_aabb(&crate::library::expanded_aabb(p, &self.libs));
            }
            bb
        };
        if let Some(Drag::PlaceClone {
            origin: grab, last, ..
        }) = &mut self.drag
        {
            *grab = bb.min;
            *last = bb.min;
        }
    }

    /// Place click-to-drop clones. Returns true if a place-clone session was active.
    pub(super) fn place_pending_clone_at(&mut self, pt: Point) -> bool {
        self.move_place_clone(pt);
        let Some(Drag::PlaceClone { prims, .. }) = self.drag.take() else {
            return false;
        };
        if prims.is_empty() {
            return true;
        }
        self.push_undo();
        self.selected.clear();
        for p in prims {
            let i = self.doc.insert(p);
            self.selected.push(i);
        }
        true
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
                match p {
                    Primitive::Component(ComponentRef { mirrored, .. }) => {
                        *mirrored = !*mirrored;
                    }
                    Primitive::Text(t) => {
                        t.style ^= STYLE_MIRRORED;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn set_selected_layer(&mut self, layer: LayerId) {
        let layer = self.doc.layers.clamp_id(layer);
        self.push_undo();
        for &i in &self.selected {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                if p.uses_component_layers() {
                    continue;
                }
                p.set_layer(layer);
            }
        }
    }
}

fn rotate_primitive(p: &mut Primitive, origin: Point) {
    p.transform(|q| q.rotate90_cw(origin));
    match p {
        Primitive::Component(ComponentRef { rotations, .. }) => {
            // `rotate90_cw` is CCW in Y-down. FidoCadJ CCW uses
            // `o = (o + 3) % 4` so internals follow the same turn
            // as rotating the expanded primitives together.
            *rotations = (*rotations + 3) % 4;
        }
        Primitive::Text(t) => {
            t.angle = (t.angle + 90).rem_euclid(360);
        }
        _ => {}
    }
}
