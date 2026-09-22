//! In-memory drawing: a project with one or more sheets.

use std::ops::{Deref, DerefMut};

use crate::consts::{
    DEFAULT_STROKE_HUNDREDTHS, GRID_MAX, GRID_MIN, SNAP_MAX, SNAP_MIN, STROKE_HUNDREDTHS_MAX,
    STROKE_HUNDREDTHS_MIN,
};
use crate::geom::Aabb;
use crate::layers::LayerSet;
use crate::library::LibrarySet;
use crate::primitive::Primitive;
use serde::{Deserialize, Serialize};

/// Default name of the first sheet (Italian). English UI uses [`default_sheet_name`].
pub const DEFAULT_SHEET_NAME: &str = "Foglio 1";

/// Localized default name for a new sheet (`index` is 0-based).
pub fn default_sheet_name(locale: &str, index: usize) -> String {
    let n = index + 1;
    if locale.eq_ignore_ascii_case("en") {
        format!("Sheet {n}")
    } else {
        format!("Foglio {n}")
    }
}

/// Localized duplicate suffix (`Foglio 1 copia` / `Sheet 1 copy`).
pub fn sheet_copy_name(locale: &str, base: &str) -> String {
    if locale.eq_ignore_ascii_case("en") {
        format!("{base} copy")
    } else {
        format!("{base} copia")
    }
}

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

/// One drawing sheet: primitives plus grid/snap.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sheet {
    pub name: String,
    pub primitives: Vec<Primitive>,
    pub grid: i32,
    pub grid_y: i32,
    pub snap: i32,
    pub snap_y: i32,
    pub show_grid: bool,
    pub snap_enable: bool,
}

impl Default for Sheet {
    fn default() -> Self {
        let s = ProjectSettings::default();
        Self {
            name: DEFAULT_SHEET_NAME.into(),
            primitives: Vec::new(),
            grid: s.grid,
            grid_y: s.grid_y,
            snap: s.snap,
            snap_y: s.snap_y,
            show_grid: s.show_grid,
            snap_enable: s.snap_enable,
        }
    }
}

impl Sheet {
    pub fn apply_grid_snap(&mut self, s: ProjectSettings) {
        let s = s.clamped();
        self.grid = s.grid;
        self.grid_y = s.grid_y;
        self.snap = s.snap;
        self.snap_y = s.snap_y;
        self.show_grid = s.show_grid;
        self.snap_enable = s.snap_enable;
    }

    pub fn aabb(&self, libs: &LibrarySet) -> Aabb {
        let mut bb = Aabb::empty();
        for p in &self.primitives {
            bb.include_aabb(&crate::library::expanded_aabb(p, libs));
        }
        bb
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub layers: LayerSet,
    pub pcb_mode: bool,
    pub hide_component_origin: bool,
    pub stroke_hundredths: i32,
    pub default_filled: bool,
    pub sheets: Vec<Sheet>,
    pub warnings: u32,
    /// True when the layer table was inferred (no `LD` lines in the file).
    #[serde(default, skip_serializing)]
    pub inferred_layers: bool,
    /// Which sheet `Deref` exposes. Session-only; not written to FCD.
    #[serde(default, skip_serializing)]
    view_index: usize,
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.layers == other.layers
            && self.pcb_mode == other.pcb_mode
            && self.hide_component_origin == other.hide_component_origin
            && self.stroke_hundredths == other.stroke_hundredths
            && self.default_filled == other.default_filled
            && self.sheets == other.sheets
            && self.warnings == other.warnings
            && self.inferred_layers == other.inferred_layers
    }
}

impl Default for Document {
    fn default() -> Self {
        let s = ProjectSettings::default();
        Self {
            title: String::new(),
            layers: LayerSet::default(),
            pcb_mode: false,
            hide_component_origin: s.hide_component_origin,
            stroke_hundredths: s.stroke_hundredths,
            default_filled: s.default_filled,
            sheets: vec![Sheet::default()],
            warnings: 0,
            inferred_layers: false,
            view_index: 0,
        }
    }
}

impl Deref for Document {
    type Target = Sheet;

    fn deref(&self) -> &Sheet {
        let i = self.view_index.min(self.sheets.len().saturating_sub(1));
        &self.sheets[i]
    }
}

impl DerefMut for Document {
    fn deref_mut(&mut self) -> &mut Sheet {
        let i = self.view_index.min(self.sheets.len().saturating_sub(1));
        &mut self.sheets[i]
    }
}

impl Document {
    pub fn view_index(&self) -> usize {
        self.view_index.min(self.sheets.len().saturating_sub(1))
    }

    pub fn set_view_index(&mut self, i: usize) {
        if self.sheets.is_empty() {
            self.sheets.push(Sheet::default());
        }
        self.view_index = i.min(self.sheets.len() - 1);
    }

    pub fn sheet(&self, i: usize) -> Option<&Sheet> {
        self.sheets.get(i)
    }

    pub fn sheet_mut(&mut self, i: usize) -> Option<&mut Sheet> {
        self.sheets.get_mut(i)
    }

    pub fn project_settings(&self) -> ProjectSettings {
        let sheet = &**self;
        ProjectSettings {
            grid: sheet.grid,
            grid_y: sheet.grid_y,
            snap: sheet.snap,
            snap_y: sheet.snap_y,
            show_grid: sheet.show_grid,
            snap_enable: sheet.snap_enable,
            hide_component_origin: self.hide_component_origin,
            stroke_hundredths: self.stroke_hundredths,
            default_filled: self.default_filled,
        }
    }

    /// Apply a `PS` line: grid/snap onto the view sheet; origin/stroke/fill onto the project.
    pub fn apply_project_settings(&mut self, s: ProjectSettings) {
        let s = s.clamped();
        self.hide_component_origin = s.hide_component_origin;
        self.stroke_hundredths = s.stroke_hundredths;
        self.default_filled = s.default_filled;
        self.apply_grid_snap(s);
    }

    /// Project-only fields (hide origin, stroke, fill) shared by every sheet.
    pub fn apply_drawing_defaults(&mut self, s: ProjectSettings) {
        let s = s.clamped();
        self.hide_component_origin = s.hide_component_origin;
        self.stroke_hundredths = s.stroke_hundredths;
        self.default_filled = s.default_filled;
    }

    pub fn apply_ps_to_sheet(&mut self, index: usize, s: ProjectSettings, take_project: bool) {
        let s = s.clamped();
        if let Some(sheet) = self.sheets.get_mut(index) {
            sheet.apply_grid_snap(s);
        }
        if take_project {
            self.hide_component_origin = s.hide_component_origin;
            self.stroke_hundredths = s.stroke_hundredths;
            self.default_filled = s.default_filled;
        }
    }

    pub fn stroke_width(&self) -> f32 {
        ProjectSettings {
            stroke_hundredths: self.stroke_hundredths,
            ..ProjectSettings::default()
        }
        .stroke_width()
    }

    pub fn aabb(&self, libs: &LibrarySet) -> Aabb {
        self.deref().aabb(libs)
    }

    pub fn aabb_sheet(&self, index: usize, libs: &LibrarySet) -> Aabb {
        self.sheets
            .get(index)
            .map(|s| s.aabb(libs))
            .unwrap_or_else(Aabb::empty)
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

    pub fn sheet_name_taken(&self, name: &str, except: Option<usize>) -> bool {
        self.sheets
            .iter()
            .enumerate()
            .any(|(i, s)| except != Some(i) && s.name.eq_ignore_ascii_case(name))
    }

    pub fn unique_sheet_name(&self, wanted: &str) -> String {
        let wanted = sanitize_sheet_name(wanted);
        if wanted.is_empty() {
            return self.unique_sheet_name(DEFAULT_SHEET_NAME);
        }
        if !self.sheet_name_taken(&wanted, None) {
            return wanted;
        }
        for n in 2..10_000 {
            let candidate = format!("{wanted} {n}");
            if !self.sheet_name_taken(&candidate, None) {
                return candidate;
            }
        }
        wanted
    }

    pub fn add_sheet(&mut self, name: &str) -> usize {
        let name = self.unique_sheet_name(name);
        let mut sheet = Sheet::default();
        sheet.name = name;
        if let Some(src) = self.sheets.first() {
            sheet.grid = src.grid;
            sheet.grid_y = src.grid_y;
            sheet.snap = src.snap;
            sheet.snap_y = src.snap_y;
            sheet.show_grid = src.show_grid;
            sheet.snap_enable = src.snap_enable;
        }
        self.sheets.push(sheet);
        self.sheets.len() - 1
    }

    pub fn duplicate_sheet(&mut self, index: usize, locale: &str) -> Option<usize> {
        let src = self.sheets.get(index)?.clone();
        let name = self.unique_sheet_name(&sheet_copy_name(locale, &src.name));
        let mut sheet = src;
        sheet.name = name;
        self.sheets.push(sheet);
        Some(self.sheets.len() - 1)
    }

    pub fn rename_sheet(&mut self, index: usize, name: &str) -> bool {
        let name = sanitize_sheet_name(name);
        if name.is_empty() || index >= self.sheets.len() {
            return false;
        }
        if self.sheet_name_taken(&name, Some(index)) {
            return false;
        }
        self.sheets[index].name = name;
        true
    }

    pub fn remove_sheet(&mut self, index: usize) -> bool {
        if self.sheets.len() <= 1 || index >= self.sheets.len() {
            return false;
        }
        self.sheets.remove(index);
        if self.view_index >= self.sheets.len() {
            self.view_index = self.sheets.len() - 1;
        } else if self.view_index > index {
            self.view_index -= 1;
        }
        true
    }

    pub fn reorder_sheets(&mut self, from: usize, to: usize) -> bool {
        let n = self.sheets.len();
        if from >= n || to >= n || from == to {
            return false;
        }
        let sheet = self.sheets.remove(from);
        self.sheets.insert(to, sheet);
        self.view_index = remap_index(self.view_index, from, to);
        true
    }

    /// Single-sheet clone used when copying the current sheet (no extra `FIDOSHEET`).
    pub fn isolate_sheet(&self, index: usize) -> Document {
        let mut out = self.clone();
        let sheet = self
            .sheets
            .get(index)
            .cloned()
            .unwrap_or_else(Sheet::default);
        out.sheets = vec![sheet];
        out.view_index = 0;
        out
    }

    pub fn for_each_primitives_mut(&mut self, mut f: impl FnMut(&mut Vec<Primitive>)) {
        for sheet in &mut self.sheets {
            f(&mut sheet.primitives);
        }
    }
}

pub fn sanitize_sheet_name(name: &str) -> String {
    name.trim().replace(']', "").to_string()
}

fn remap_index(current: usize, from: usize, to: usize) -> usize {
    if current == from {
        to
    } else if from < to && current > from && current <= to {
        current - 1
    } else if to < from && current >= to && current < from {
        current + 1
    } else {
        current
    }
}
