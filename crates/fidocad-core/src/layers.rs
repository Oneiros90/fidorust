//! Sixteen drawing layers as in FidoCAD 0.96.

use serde::{Deserialize, Serialize};

/// 127 µm per logical unit (0.96 `DEFAULT_MICRON_PER_LU`).
pub const MICRON_PER_LU: i32 = 127;
pub const LAYER_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u8);

impl LayerId {
    pub const SCHEMATIC: Self = Self(0);
    pub const PCB_COPPER: Self = Self(1);
    pub const PCB_COMPONENTS: Self = Self(2);
    pub const SILK: Self = Self(3);

    pub fn from_i32(v: i32) -> Self {
        if (0..LAYER_COUNT as i32).contains(&v) {
            Self(v as u8)
        } else {
            Self(0)
        }
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerInfo {
    pub name: String,
    pub color: [u8; 3],
    pub show: bool,
    pub print: bool,
}

impl LayerInfo {
    pub fn default_set() -> [LayerInfo; LAYER_COUNT] {
        let mut layers: [LayerInfo; LAYER_COUNT] =
            std::array::from_fn(|i| LayerInfo {
                name: format!("Layer {}", i + 1),
                color: [0, 0, 0],
                show: true,
                print: true,
            });
        layers[0].name = "Schema".into();
        layers[0].color = [0, 0, 0];
        layers[1].name = "PCB lato rame".into();
        layers[1].color = [0, 0, 192];
        layers[2].name = "PCB lato componenti".into();
        layers[2].color = [0, 192, 0];
        layers[3].name = "Serigrafie".into();
        layers[3].color = [0, 150, 150];
        layers
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerSet {
    pub layers: [LayerInfo; LAYER_COUNT],
}

impl Default for LayerSet {
    fn default() -> Self {
        Self {
            layers: LayerInfo::default_set(),
        }
    }
}

impl LayerSet {
    pub fn visible(&self, id: LayerId) -> bool {
        self.layers[id.index()].show
    }

    pub fn printable(&self, id: LayerId) -> bool {
        self.layers[id.index()].print
    }

    pub fn color(&self, id: LayerId) -> [u8; 3] {
        self.layers[id.index()].color
    }
}
