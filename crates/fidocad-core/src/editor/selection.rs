//! Selection transforms.

use super::Editor;
use crate::geom::Point;
use crate::layers::LayerId;
use crate::primitive::{ComponentRef, Primitive};

impl Editor {
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
