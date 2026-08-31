//! In-memory drawing.

use crate::geom::{Aabb, Point};
use crate::layers::LayerSet;
use crate::library::LibrarySet;
use crate::primitive::Primitive;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct SaveOptions {
    pub split_nonstandard_macros: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            split_nonstandard_macros: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub primitives: Vec<Primitive>,
    pub layers: LayerSet,
    pub pcb_mode: bool,
    /// Grid pitch in LU (X). Original `m_xgrid`, 1..=40.
    pub grid: i32,
    /// Grid pitch in LU (Y). Original `m_ygrid`, 1..=40.
    pub grid_y: i32,
    /// Snap pitch in LU (X). Original `m_xsnap`, 1..=20.
    pub snap: i32,
    /// Snap pitch in LU (Y). Original `m_ysnap`, 1..=20.
    pub snap_y: i32,
    pub warnings: u32,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            title: String::new(),
            primitives: Vec::new(),
            layers: LayerSet::default(),
            pcb_mode: false,
            grid: 5,
            grid_y: 5,
            snap: 5,
            snap_y: 5,
            warnings: 0,
        }
    }
}

impl Document {
    pub fn aabb(&self, libs: &LibrarySet) -> Aabb {
        let mut bb = Aabb::empty();
        for p in &self.primitives {
            for q in crate::library::expand_primitive(p, libs) {
                bb.include_aabb(&q.aabb());
            }
        }
        bb
    }

    pub fn insert(&mut self, p: Primitive) -> usize {
        self.primitives.push(p);
        self.primitives.len() - 1
    }

    pub fn remove(&mut self, index: usize) -> Option<Primitive> {
        if index < self.primitives.len() {
            Some(self.primitives.remove(index))
        } else {
            None
        }
    }

    pub fn selected_aabb(&self, selected: &[usize], libs: &LibrarySet) -> Aabb {
        let mut bb = Aabb::empty();
        for &i in selected {
            if let Some(p) = self.primitives.get(i) {
                for q in crate::library::expand_primitive(p, libs) {
                    bb.include_aabb(&q.aabb());
                }
            }
        }
        bb
    }

    pub fn lu_to_mm(lu: i32) -> f64 {
        lu as f64 * crate::layers::MICRON_PER_LU as f64 / 1000.0
    }

    pub fn origin_mm(&self, p: Point) -> (f64, f64) {
        (Self::lu_to_mm(p.x), Self::lu_to_mm(p.y))
    }
}
