//! Drawing layers stored in the document (`.fcd` `LD` lines).

use serde::{Deserialize, Serialize};

use crate::primitive::Primitive;

/// 127 µm per logical unit (0.96 `DEFAULT_MICRON_PER_LU`).
pub const MICRON_PER_LU: i32 = 127;
/// Maximum number of layers (`LayerId` is `u8`).
pub const MAX_LAYERS: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u8);

impl LayerId {
    pub const SCHEMATIC: Self = Self(0);
    pub const PCB_COPPER: Self = Self(1);
    pub const PCB_COMPONENTS: Self = Self(2);
    pub const SILK: Self = Self(3);

    pub fn from_i32(v: i32) -> Self {
        if (0..=u8::MAX as i32).contains(&v) {
            Self(v as u8)
        } else {
            Self(0)
        }
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerInfo {
    pub name: String,
    pub color: [u8; 3],
    pub show: bool,
}

impl LayerInfo {
    pub fn new(name: impl Into<String>, color: [u8; 3], show: bool) -> Self {
        Self {
            name: name.into(),
            color,
            show,
        }
    }

    fn generic(index: usize) -> Self {
        Self::new(format!("Layer {}", index + 1), palette_color(index), true)
    }
}

fn palette_color(index: usize) -> [u8; 3] {
    const PALETTE: [[u8; 3]; 8] = [
        [0, 0, 0],
        [0, 0, 192],
        [0, 192, 0],
        [0, 150, 150],
        [192, 0, 0],
        [160, 0, 160],
        [192, 128, 0],
        [80, 80, 80],
    ];
    PALETTE[index % PALETTE.len()]
}

/// The four classic FidoCAD layers used for new documents and files without `LD`.
pub fn fidocad_fallback() -> Vec<LayerInfo> {
    vec![
        LayerInfo::new("Schema", [0, 0, 0], true),
        LayerInfo::new("PCB lato rame", [0, 0, 192], true),
        LayerInfo::new("PCB lato componenti", [0, 192, 0], true),
        LayerInfo::new("Serigrafie", [0, 150, 150], true),
    ]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerSet {
    pub layers: Vec<LayerInfo>,
}

impl Default for LayerSet {
    fn default() -> Self {
        Self {
            layers: fidocad_fallback(),
        }
    }
}

impl LayerSet {
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn visible(&self, id: LayerId) -> bool {
        self.layers.get(id.index()).map(|l| l.show).unwrap_or(true)
    }

    pub fn color(&self, id: LayerId) -> [u8; 3] {
        self.layers
            .get(id.index())
            .map(|l| l.color)
            .unwrap_or([0, 0, 0])
    }

    pub fn ensure_len(&mut self, n: usize) {
        let n = n.min(MAX_LAYERS).max(1);
        while self.layers.len() < n {
            let i = self.layers.len();
            self.layers.push(LayerInfo::generic(i));
        }
    }

    pub fn add(&mut self) -> Option<LayerId> {
        if self.layers.len() >= MAX_LAYERS {
            return None;
        }
        let i = self.layers.len();
        self.layers.push(LayerInfo::generic(i));
        Some(LayerId(i as u8))
    }

    pub fn clamp_id(&self, id: LayerId) -> LayerId {
        let max = self.layers.len().saturating_sub(1) as u8;
        if id.0 > max {
            LayerId(0)
        } else {
            id
        }
    }
}

pub fn remap_after_remove(id: u8, removed: u8) -> u8 {
    if id > removed {
        id - 1
    } else {
        id
    }
}

/// Remap a layer index after `Vec::remove(from)` + `Vec::insert(to, …)`.
pub fn remap_after_reorder(id: u8, from: u8, to: u8) -> u8 {
    if from == to {
        return id;
    }
    if from < to {
        if id == from {
            to
        } else if id > from && id <= to {
            id - 1
        } else {
            id
        }
    } else if id == from {
        to
    } else if id >= to && id < from {
        id + 1
    } else {
        id
    }
}

pub fn remap_primitive_layers(prims: &mut [Primitive], f: impl Fn(LayerId) -> LayerId) {
    for p in prims {
        if matches!(p, Primitive::Macro { .. }) {
            continue;
        }
        let next = f(p.layer());
        if next != p.layer() {
            p.set_layer(next);
        }
    }
}

pub fn count_on_layer(prims: &[Primitive], id: LayerId) -> usize {
    prims
        .iter()
        .filter(|p| !matches!(p, Primitive::Macro { .. }) && p.layer() == id)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reorder_remap_move_down() {
        // [0,1,2,3] move 0 → 2  => [1,2,0,3]
        assert_eq!(remap_after_reorder(0, 0, 2), 2);
        assert_eq!(remap_after_reorder(1, 0, 2), 0);
        assert_eq!(remap_after_reorder(2, 0, 2), 1);
        assert_eq!(remap_after_reorder(3, 0, 2), 3);
    }

    #[test]
    fn reorder_remap_move_up() {
        // [0,1,2,3] move 2 → 0  => [2,0,1,3]
        assert_eq!(remap_after_reorder(2, 2, 0), 0);
        assert_eq!(remap_after_reorder(0, 2, 0), 1);
        assert_eq!(remap_after_reorder(1, 2, 0), 2);
        assert_eq!(remap_after_reorder(3, 2, 0), 3);
    }

    #[test]
    fn remove_remap() {
        assert_eq!(remap_after_remove(0, 1), 0);
        assert_eq!(remap_after_remove(1, 1), 1);
        assert_eq!(remap_after_remove(2, 1), 1);
        assert_eq!(remap_after_remove(3, 1), 2);
    }
}
