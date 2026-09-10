//! Layer CRUD on the editor.

use super::Editor;
use crate::layers::{count_on_layer, remap_primitive_layers, LayerId, LayerInfo, MAX_LAYERS};

impl Editor {
    pub fn layer_object_count(&self, index: usize) -> usize {
        count_on_layer(&self.doc.primitives, LayerId(index as u8))
    }

    pub fn add_layer(&mut self) -> Option<LayerId> {
        if self.doc.layers.len() >= MAX_LAYERS {
            return None;
        }
        self.push_undo();
        let id = self.doc.layers.add()?;
        self.layer = id;
        Some(id)
    }

    /// Remove layer `index`. `move_to` relocates its objects; `None` deletes them.
    pub fn delete_layer(&mut self, index: usize, move_to: Option<usize>) -> bool {
        let n = self.doc.layers.len();
        if n <= 1 || index >= n {
            return false;
        }
        if let Some(dest) = move_to {
            if dest >= n || dest == index {
                return false;
            }
        }
        self.push_undo();
        let removed = LayerId(index as u8);
        if let Some(dest) = move_to {
            let dest_id = LayerId(dest as u8);
            for p in &mut self.doc.primitives {
                if p.assigned_layer() == Some(removed) {
                    p.set_layer(dest_id);
                }
            }
        } else {
            self.doc
                .primitives
                .retain(|p| p.assigned_layer() != Some(removed));
        }
        remap_primitive_layers(&mut self.doc.primitives, |id| {
            id.remap_after_remove(index as u8)
        });
        self.doc.layers.remove(index);
        let cur = self.layer.0 as usize;
        if cur == index {
            self.layer = LayerId(index.min(self.doc.layers.len() - 1) as u8);
        } else if cur > index {
            self.layer = LayerId((cur - 1) as u8);
        }
        self.selected.clear();
        true
    }

    pub fn reorder_layer(&mut self, from: usize, to: usize) -> bool {
        let n = self.doc.layers.len();
        if from >= n || to >= n || from == to {
            return false;
        }
        self.push_undo();
        self.doc.layers.move_item(from, to);
        remap_primitive_layers(&mut self.doc.primitives, |id| {
            id.remap_after_reorder(from as u8, to as u8)
        });
        self.layer = self.layer.remap_after_reorder(from as u8, to as u8);
        true
    }

    pub fn update_layer(&mut self, index: usize, f: impl FnOnce(&mut LayerInfo)) -> bool {
        let Some(before) = self.doc.layers.get(index).cloned() else {
            return false;
        };
        let mut after = before.clone();
        f(&mut after);
        if after == before {
            return false;
        }
        self.push_undo();
        self.doc.layers.update(index, |l| *l = after);
        true
    }

    pub fn set_layer_show(&mut self, index: usize, show: bool) -> bool {
        self.update_layer(index, |l| l.show = show)
    }

    pub fn set_layer_name(&mut self, index: usize, name: String) -> bool {
        self.update_layer(index, |l| l.name = name)
    }

    pub fn set_layer_color(&mut self, index: usize, color: [u8; 4]) -> bool {
        let Some(before) = self.doc.layers.get(index).cloned() else {
            return false;
        };
        if before.color == color {
            return false;
        }
        let coalesce = self.layer_color_edit == Some(index);
        if !coalesce {
            self.push_undo();
        }
        self.layer_color_edit = Some(index);
        self.doc.layers.update(index, |l| l.color = color);
        true
    }

    pub(super) fn clamp_current_layer(&mut self) {
        let max = self.doc.layers.len().saturating_sub(1) as u8;
        if self.layer.0 > max {
            self.layer = LayerId(max);
        }
    }
}
