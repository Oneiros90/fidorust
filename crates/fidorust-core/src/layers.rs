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
        Self::new(standard_layer_name(index), palette_color(index), true)
    }
}

/// Classic FidoCAD sheets, then repeating palette colours for extra layers.
pub fn palette_color(index: usize) -> [u8; 4] {
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

/// Number of classic FidoCAD sheets used for new documents and files without `LD`.
pub const FALLBACK_LAYER_COUNT: usize = 4;

/// Stable English names stored in the document / `.fcd`. The UI localizes them.
pub fn standard_layer_name(index: usize) -> String {
    match index {
        0 => "Schematic".into(),
        1 => "PCB copper side".into(),
        2 => "PCB component side".into(),
        3 => "Silkscreen".into(),
        _ => format!("Layer {}", index + 1),
    }
}

/// The four classic FidoCAD layers used for new documents and files without `LD`.
pub fn fidocad_fallback() -> Vec<LayerInfo> {
    (0..FALLBACK_LAYER_COUNT)
        .map(|i| LayerInfo::new(standard_layer_name(i), palette_color(i), true))
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportLayerOverlay {
    pub show: bool,
    pub invert: bool,
    pub color: Option<[u8; 4]>,
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

    /// Overlay used by file export (`bw` / invert / per-layer color). Empty `overlays`
    /// with `bw` blacks every visible layer; otherwise each overlay is applied in order.
    pub fn apply_export_overlay(&mut self, bw: bool, overlays: &[ExportLayerOverlay]) {
        if overlays.is_empty() {
            if bw {
                for i in 0..self.len() {
                    self.update(i, |info| {
                        if info.show {
                            info.color = [0, 0, 0, info.color[3]];
                        }
                    });
                }
            }
            return;
        }
        for (i, overlay) in overlays.iter().enumerate() {
            self.update(i, |info| {
                info.show = overlay.show;
                if bw {
                    let a = overlay.color.map(|c| c[3]).unwrap_or(info.color[3]);
                    info.color = [0, 0, 0, a];
                } else if let Some(c) = overlay.color {
                    info.color = c;
                } else if overlay.invert {
                    info.color = [
                        255 - info.color[0],
                        255 - info.color[1],
                        255 - info.color[2],
                        info.color[3],
                    ];
                }
            });
        }
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
    prims
        .iter()
        .filter(|p| p.assigned_layer() == Some(id))
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

    #[test]
    fn fallback_uses_english_names_and_palette() {
        let layers = fidocad_fallback();
        assert_eq!(layers[0].name, "Schematic");
        assert_eq!(layers[1].name, "PCB copper side");
        assert_eq!(layers[2].name, "PCB component side");
        assert_eq!(layers[3].name, "Silkscreen");
        assert_eq!(layers[0].color, palette_color(0));
        assert_eq!(layers[3].color, palette_color(3));
        assert_eq!(standard_layer_name(4), "Layer 5");
        assert_eq!(palette_color(4), rgb(192, 0, 0));
        assert_eq!(palette_color(6), rgb(192, 128, 0));
    }
}
