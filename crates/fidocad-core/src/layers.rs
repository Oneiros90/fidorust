//! Drawing layers stored in the document (`.fcd` `LD` lines).

use serde::{Deserialize, Deserializer, Serialize};

use crate::primitive::Primitive;

/// Opaque RGB (`alpha = 255`).
pub const fn rgb(r: u8, g: u8, b: u8) -> [u8; 4] {
    [r, g, b, 255]
}

fn deserialize_rgba<'de, D>(deserializer: D) -> Result<[u8; 4], D::Error>
where
    D: Deserializer<'de>,
{
    let vals = Vec::<u8>::deserialize(deserializer)?;
    match vals.as_slice() {
        [r, g, b] => Ok([*r, *g, *b, 255]),
        [r, g, b, a] => Ok([*r, *g, *b, *a]),
        _ => Err(serde::de::Error::custom(
            "color must be [r,g,b] or [r,g,b,a]",
        )),
    }
}

/// 127 µm per logical unit (0.96 `DEFAULT_MICRON_PER_LU`).
pub const MICRON_PER_LU: i32 = 127;
/// Maximum number of layers (`LayerId` is `u8`).
pub const MAX_LAYERS: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u8);

impl LayerId {
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

    pub fn remap_after_remove(self, removed: u8) -> Self {
        Self(remap_after_remove(self.0, removed))
    }

    pub fn remap_after_reorder(self, from: u8, to: u8) -> Self {
        Self(remap_after_reorder(self.0, from, to))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerInfo {
    pub name: String,
    #[serde(deserialize_with = "deserialize_rgba")]
    pub color: [u8; 4],
    pub show: bool,
}

impl LayerInfo {
    pub fn new(name: impl Into<String>, color: [u8; 4], show: bool) -> Self {
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

fn palette_color(index: usize) -> [u8; 4] {
    const PALETTE: [[u8; 4]; 8] = [
        rgb(0, 0, 0),
        rgb(0, 0, 192),
        rgb(0, 192, 0),
        rgb(0, 150, 150),
        rgb(192, 0, 0),
        rgb(160, 0, 160),
        rgb(192, 128, 0),
        rgb(80, 80, 80),
    ];
    PALETTE[index % PALETTE.len()]
}

/// The four classic FidoCAD layers used for new documents and files without `LD`.
pub fn fidocad_fallback() -> Vec<LayerInfo> {
    vec![
        LayerInfo::new("Schema", rgb(0, 0, 0), true),
        LayerInfo::new("PCB lato rame", rgb(0, 0, 192), true),
        LayerInfo::new("PCB lato componenti", rgb(0, 192, 0), true),
        LayerInfo::new("Serigrafie", rgb(0, 150, 150), true),
    ]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerSet {
    layers: Vec<LayerInfo>,
}

impl Default for LayerSet {
    fn default() -> Self {
        Self {
            layers: fidocad_fallback(),
        }
    }
}

impl LayerSet {
    pub fn from_vec(layers: Vec<LayerInfo>) -> Self {
        Self { layers }
    }

    /// Single always-visible sheet used only while editing a component definition.
    /// Not written to FCD; colour is overridden at tessellate time from the canvas theme.
    pub fn component_edit() -> Self {
        Self {
            layers: vec![LayerInfo::new("", rgb(0, 0, 0), true)],
        }
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &LayerInfo> {
        self.layers.iter()
    }

    pub fn get(&self, index: usize) -> Option<&LayerInfo> {
        self.layers.get(index)
    }

    pub fn remove(&mut self, index: usize) -> LayerInfo {
        self.layers.remove(index)
    }

    pub fn move_item(&mut self, from: usize, to: usize) {
        let item = self.layers.remove(from);
        self.layers.insert(to, item);
    }

    pub fn update(&mut self, index: usize, f: impl FnOnce(&mut LayerInfo)) -> bool {
        match self.layers.get_mut(index) {
            Some(layer) => {
                f(layer);
                true
            }
            None => false,
        }
    }

    pub fn visible(&self, id: LayerId) -> bool {
        self.layers.get(id.index()).map(|l| l.show).unwrap_or(true)
    }

    pub fn color(&self, id: LayerId) -> [u8; 4] {
        self.layers
            .get(id.index())
            .map(|l| l.color)
            .unwrap_or(rgb(0, 0, 0))
    }

    pub fn ensure_len(&mut self, n: usize) {
        let n = n.clamp(1, MAX_LAYERS);
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
        let next = f(p.layer());
        if next != p.layer() {
            p.set_layer(next);
        }
    }
}

pub fn count_on_layer(prims: &[Primitive], id: LayerId) -> usize {
    prims.iter().filter(|p| p.layer() == id).count()
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
