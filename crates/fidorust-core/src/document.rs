//! In-memory drawing.

use crate::consts::{
    DEFAULT_STROKE_HUNDREDTHS, GRID_MAX, GRID_MIN, SNAP_MAX, SNAP_MIN, STROKE_HUNDREDTHS_MAX,
    STROKE_HUNDREDTHS_MIN,
};
use crate::geom::Aabb;
use crate::layers::LayerSet;
use crate::library::LibrarySet;
use crate::primitive::Primitive;
use serde::{Deserialize, Serialize};

/// Project settings stored in the `.fcd` as a `PS` line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub grid: i32,
    pub grid_y: i32,
    pub snap: i32,
    pub snap_y: i32,
    pub show_grid: bool,
    pub snap_enable: bool,
    pub hide_component_origin: bool,
    pub stroke_hundredths: i32,
    pub default_filled: bool,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            grid: 5,
            grid_y: 5,
            snap: 5,
            snap_y: 5,
            show_grid: true,
            snap_enable: true,
            hide_component_origin: true,
            stroke_hundredths: DEFAULT_STROKE_HUNDREDTHS,
            default_filled: false,
        }
    }
}

impl ProjectSettings {
    pub fn clamped(self) -> Self {
        Self {
            grid: self.grid.clamp(GRID_MIN, GRID_MAX),
            grid_y: self.grid_y.clamp(GRID_MIN, GRID_MAX),
            snap: self.snap.clamp(SNAP_MIN, SNAP_MAX),
            snap_y: self.snap_y.clamp(SNAP_MIN, SNAP_MAX),
            show_grid: self.show_grid,
            snap_enable: self.snap_enable,
            hide_component_origin: self.hide_component_origin,
            stroke_hundredths: self
                .stroke_hundredths
                .clamp(STROKE_HUNDREDTHS_MIN, STROKE_HUNDREDTHS_MAX),
            default_filled: self.default_filled,
        }
    }

    pub fn stroke_width(self) -> f32 {
        self.stroke_hundredths as f32 / 100.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub show_grid: bool,
    pub snap_enable: bool,
    pub hide_component_origin: bool,
    /// Schematic stroke in hundredths of LU. Original hairline is `25` (0.25 LU).
    pub stroke_hundredths: i32,
    /// Default fill for newly drawn rectangles, ellipses, and polygons.
    pub default_filled: bool,
    pub warnings: u32,
    /// True when the layer table was inferred (no `LD` lines in the file).
    #[serde(default, skip_serializing)]
    pub inferred_layers: bool,
}

impl Default for Document {
    fn default() -> Self {
        let s = ProjectSettings::default();
        Self {
            title: String::new(),
            primitives: Vec::new(),
            layers: LayerSet::default(),
            pcb_mode: false,
            grid: s.grid,
            grid_y: s.grid_y,
            snap: s.snap,
            snap_y: s.snap_y,
            show_grid: s.show_grid,
            snap_enable: s.snap_enable,
            hide_component_origin: s.hide_component_origin,
            stroke_hundredths: s.stroke_hundredths,
            default_filled: s.default_filled,
            warnings: 0,
            inferred_layers: false,
        }
    }
}

impl Document {
    pub fn project_settings(&self) -> ProjectSettings {
        ProjectSettings {
            grid: self.grid,
            grid_y: self.grid_y,
            snap: self.snap,
            snap_y: self.snap_y,
            show_grid: self.show_grid,
            snap_enable: self.snap_enable,
            hide_component_origin: self.hide_component_origin,
            stroke_hundredths: self.stroke_hundredths,
            default_filled: self.default_filled,
        }
    }

    pub fn apply_project_settings(&mut self, s: ProjectSettings) {
        let s = s.clamped();
        self.grid = s.grid;
        self.grid_y = s.grid_y;
        self.snap = s.snap;
        self.snap_y = s.snap_y;
        self.show_grid = s.show_grid;
        self.snap_enable = s.snap_enable;
        self.hide_component_origin = s.hide_component_origin;
        self.stroke_hundredths = s.stroke_hundredths;
        self.default_filled = s.default_filled;
    }

    pub fn stroke_width(&self) -> f32 {
        self.project_settings().stroke_width()
    }

    pub fn aabb(&self, libs: &LibrarySet) -> Aabb {
        let mut bb = Aabb::empty();
        for p in &self.primitives {
            bb.include_aabb(&crate::library::expanded_aabb(p, libs));
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
                bb.include_aabb(&crate::library::expanded_aabb(p, libs));
            }
        }
        bb
    }

    pub fn lu_to_mm(lu: i32) -> f64 {
        lu as f64 * crate::layers::MICRON_PER_LU as f64 / 1000.0
    }
}
