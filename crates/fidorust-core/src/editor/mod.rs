//! Interactive editor: view, history, tools, layers, and text edit.

mod components;
mod history;
mod layers;
mod pointer;
mod selection;
mod sheets;
mod text_edit;
mod tools;
mod view;
mod view_overlay;

pub use tools::{DblClickAction, EditorError, TextEditSession, Tool};
pub use view_overlay::{
    CanvasPalette, OverlayMarker, OverlayPad, OverlayStroke, OverlayTint, ViewOverlay,
};

use crate::consts::{FIT_MARGIN, GRID_MAX, GRID_MIN, SNAP_MAX, SNAP_MIN};
use crate::document::{Document, PrimitiveDefaults, ProjectSettings};
use crate::geom::Point;
use crate::layers::LayerId;
use crate::library::LibrarySet;
use crate::primitive::Primitive;
use crate::properties::{apply_selection_props, selection_props_form, PropPatch};

use components::ComponentEditSession;
use history::HistoryOp;
use tools::{Draft, Drag};

#[derive(Clone, Debug)]
pub struct PaneState {
    pub sheet_index: usize,
    pub zoom: f32,
    pub pan: (f32, f32),
    pub selected: Vec<usize>,
    pub hover: Option<Point>,
    pub hover_hit: bool,
    pub hover_index: Option<usize>,
}

impl PaneState {
    pub(super) fn new() -> Self {
        Self {
            sheet_index: 0,
            zoom: 4.0,
            pan: (FIT_MARGIN, FIT_MARGIN),
            selected: Vec::new(),
            hover: None,
            hover_hit: false,
            hover_index: None,
        }
    }
}

/// Screen chrome only: overlay colours. Not saved with the document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasTheme {
    #[default]
    Light,
    Dark,
    White,
}

impl CanvasTheme {
    pub fn parse(id: &str) -> Self {
        match id {
            "dark" => Self::Dark,
            "white" => Self::White,
            _ => Self::Light,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Editor {
    doc: Document,
    libs: LibrarySet,
    tool: Tool,
    layer: LayerId,
    selected: Vec<usize>,
    zoom: f32,
    pan: (f32, f32),
    pending_component: Option<String>,
    pending_rotations: u8,
    /// Ghost-at-cursor and RMB rotate only while a library drag is in progress.
    pending_follow: bool,
    pending_text: String,
    /// Primitive index whose glyphs are hidden while the UI overlay edits them.
    editing_text: Option<usize>,
    undo: Vec<HistoryOp>,
    redo: Vec<HistoryOp>,
    /// Pre-move / pre-handle snapshot of one sheet and the pad-link list.
    drag_checkpoint: Option<(usize, crate::document::Sheet, Vec<crate::document::PadLink>)>,
    /// Layer index of an in-progress color drag; consecutive updates share one undo frame.
    layer_color_edit: Option<usize>,
    draft: Option<Draft>,
    drag: Option<Drag>,
    hover: Option<Point>,
    /// True when the last pointer position is over selectable geometry.
    hover_hit: bool,
    /// Top-level primitive under the cursor (select tool); drives hover tint.
    hover_index: Option<usize>,
    /// Screen theme only: preview / selection overlay colours. Not saved.
    canvas_theme: CanvasTheme,
    libs_rev: u32,
    component_edit: Option<ComponentEditSession>,
    /// Ephemeral measure overlay; never serialized.
    ruler_segments: Vec<(Point, Point)>,
    /// Ephemeral presentation; never serialized or undoable. One slot per pane.
    view_overlay: [Option<ViewOverlay>; 2],
    panes: [PaneState; 2],
    split: bool,
    active_pane: usize,
    locale: String,
}

impl Editor {
    pub fn new(mut libs: LibrarySet) -> Self {
        libs.ensure_user_libraries();
        Self {
            doc: Document::default(),
            libs,
            tool: Tool::Select,
            layer: LayerId(0),
            selected: Vec::new(),
            zoom: 4.0,
            pan: (FIT_MARGIN, FIT_MARGIN),
            pending_component: None,
            pending_rotations: 0,
            pending_follow: false,
            pending_text: "TEXT".into(),
            editing_text: None,
            undo: Vec::new(),
            redo: Vec::new(),
            drag_checkpoint: None,
            layer_color_edit: None,
            draft: None,
            drag: None,
            hover: None,
            hover_hit: false,
            hover_index: None,
            canvas_theme: CanvasTheme::Light,
            libs_rev: 0,
            component_edit: None,
            ruler_segments: Vec::new(),
            view_overlay: [None, None],
            panes: [PaneState::new(), PaneState::new()],
            split: false,
            active_pane: 0,
            locale: "it".into(),
        }
    }

    pub fn set_tool(&mut self, tool: Tool) {
        let prev = self.tool;
        self.tool = tool;
        self.cancel_draft();
        if tool != Tool::Component {
            self.pending_component = None;
            self.pending_follow = false;
        }
        if prev == Tool::PadLink && tool != Tool::PadLink {
            let _ = crate::ext::dispatch(self, "padlink.cancel", "{}");
        }
    }

    pub fn set_filled(&mut self, on: bool) {
        if self.doc.defaults.rect.filled == on
            && self.doc.defaults.ellipse.filled == on
            && self.doc.defaults.poly.filled == on
        {
            return;
        }
        self.push_project_undo();
        self.doc.defaults.rect.filled = on;
        self.doc.defaults.ellipse.filled = on;
        self.doc.defaults.poly.filled = on;
    }

    pub fn set_layer(&mut self, n: u8) {
        let max = self.doc.layers.len().saturating_sub(1) as u8;
        self.layer = LayerId(n.min(max));
        if !self.selected.is_empty() {
            self.set_selected_layer(self.layer);
            self.selected.clear();
        }
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn doc_mut(&mut self) -> &mut Document {
        &mut self.doc
    }

    pub fn set_doc(&mut self, doc: Document) {
        self.doc = doc;
    }

    pub fn libs(&self) -> &LibrarySet {
        &self.libs
    }

    pub fn libs_mut(&mut self) -> &mut LibrarySet {
        &mut self.libs
    }

    pub fn libs_rev(&self) -> u32 {
        self.libs_rev
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub fn layer(&self) -> LayerId {
        self.layer
    }

    pub fn selected(&self) -> &[usize] {
        &self.selected
    }

    pub fn selected_mut(&mut self) -> &mut Vec<usize> {
        &mut self.selected
    }

    pub fn set_selected(&mut self, selected: Vec<usize>) {
        self.selected = selected;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn pan(&self) -> (f32, f32) {
        self.pan
    }

    pub fn set_view(&mut self, zoom: f32, pan: (f32, f32)) {
        self.zoom = zoom;
        self.pan = pan;
        self.store_active_pane();
    }

    pub fn filled(&self) -> bool {
        self.filled_for_tool(self.draft.as_ref().map(|d| d.tool).unwrap_or(self.tool))
    }

    pub fn filled_for_tool(&self, tool: Tool) -> bool {
        match tool {
            Tool::Rect => self.doc.defaults.rect.filled,
            Tool::Ellipse => self.doc.defaults.ellipse.filled,
            Tool::Poly => self.doc.defaults.poly.filled,
            _ => self.doc.defaults.rect.filled,
        }
    }

    pub fn stroke_width_for_tool(&self, tool: Tool) -> f32 {
        let h = match tool {
            Tool::Line => self.doc.defaults.line_stroke_hundredths,
            Tool::Bezier => self.doc.defaults.bezier_stroke_hundredths,
            Tool::Rect => self.doc.defaults.rect.stroke_hundredths,
            Tool::Ellipse => self.doc.defaults.ellipse.stroke_hundredths,
            Tool::Poly => self.doc.defaults.poly.stroke_hundredths,
            _ => self.doc.defaults.line_stroke_hundredths,
        };
        h as f32 / 100.0
    }

    pub fn hover(&self) -> Option<Point> {
        self.hover
    }

    pub fn hover_hit(&self) -> bool {
        self.hover_hit
    }

    pub fn hover_index(&self) -> Option<usize> {
        self.hover_index
    }

    pub fn hover_op(&self) -> Option<String> {
        let i = self.hover_index?;
        self.doc.primitives.get(i).map(Primitive::hover_label)
    }

    pub fn set_hover(&mut self, hover: Option<Point>) {
        self.hover = hover;
        if hover.is_none() {
            self.hover_hit = false;
            self.hover_index = None;
        }
    }

    pub fn canvas_theme(&self) -> CanvasTheme {
        self.canvas_theme
    }

    pub fn set_canvas_theme(&mut self, theme: CanvasTheme) {
        self.canvas_theme = theme;
    }

    pub fn snap_enable(&self) -> bool {
        self.doc.snap_enable
    }

    pub fn set_snap_enable(&mut self, on: bool) {
        self.doc.snap_enable = on;
    }

    pub fn hide_component_origin(&self) -> bool {
        self.doc.hide_component_origin
    }

    pub fn set_hide_component_origin(&mut self, on: bool) {
        if self.doc.hide_component_origin == on {
            return;
        }
        self.push_project_undo();
        self.doc.hide_component_origin = on;
    }

    pub fn show_grid(&self) -> bool {
        self.doc.show_grid
    }

    pub fn set_show_grid(&mut self, on: bool) {
        self.doc.show_grid = on;
    }

    pub fn apply_project_settings(&mut self, s: ProjectSettings) {
        let s = s.clamped();
        if self.doc.project_settings() == s {
            return;
        }
        let project_changed = self.doc.hide_component_origin != s.hide_component_origin
            || self.doc.defaults.line_stroke_hundredths != s.stroke_hundredths
            || self.doc.defaults.rect.filled != s.default_filled;
        if project_changed {
            self.push_project_undo();
        } else {
            self.push_undo();
        }
        self.doc.apply_project_settings(s);
    }

    pub fn apply_sheet_settings(&mut self, s: ProjectSettings) {
        let s = s.clamped();
        let sheet = self.doc();
        if sheet.grid == s.grid
            && sheet.grid_y == s.grid_y
            && sheet.snap == s.snap
            && sheet.snap_y == s.snap_y
            && sheet.show_grid == s.show_grid
            && sheet.snap_enable == s.snap_enable
        {
            return;
        }
        self.push_undo();
        self.doc.apply_sheet_settings(s);
    }

    pub fn apply_drawing_defaults(&mut self, hide_origin: bool, defaults: PrimitiveDefaults) {
        let defaults = defaults.clamped();
        if self.doc.hide_component_origin == hide_origin && self.doc.defaults == defaults {
            return;
        }
        self.push_project_undo();
        self.doc.apply_drawing_defaults(hide_origin, defaults);
    }

    pub fn pending_rotations(&self) -> u8 {
        self.pending_rotations
    }

    pub fn pending_component(&self) -> Option<&str> {
        self.pending_component.as_deref()
    }

    pub fn editing_text(&self) -> Option<usize> {
        self.editing_text
    }

    pub fn set_editing_text(&mut self, index: Option<usize>) {
        self.editing_text = index;
    }

    pub fn set_pcb_mode(&mut self, on: bool) {
        if self.doc.pcb_mode == on {
            return;
        }
        self.push_project_undo();
        self.doc.pcb_mode = on;
    }

    pub fn set_grid(&mut self, x: i32, y: i32) {
        let x = x.clamp(GRID_MIN, GRID_MAX);
        let y = y.clamp(GRID_MIN, GRID_MAX);
        if self.doc.grid == x && self.doc.grid_y == y {
            return;
        }
        self.push_undo();
        self.doc.grid = x;
        self.doc.grid_y = y;
    }

    pub fn set_snap(&mut self, x: i32, y: i32) {
        let x = x.clamp(SNAP_MIN, SNAP_MAX);
        let y = y.clamp(SNAP_MIN, SNAP_MAX);
        if self.doc.snap == x && self.doc.snap_y == y {
            return;
        }
        self.push_undo();
        self.doc.snap = x;
        self.doc.snap_y = y;
    }

    pub fn set_track_width(&mut self, w: i32) {
        let w = w.max(1);
        if self.doc.defaults.track_width == w {
            return;
        }
        self.push_project_undo();
        self.doc.defaults.track_width = w;
    }

    pub fn set_pending_text(&mut self, text: String) {
        self.pending_text = text;
    }

    /// Switch to the component tool without cancelling an in-progress draft.
    pub fn adopt_component_tool(&mut self) {
        self.tool = Tool::Component;
    }

    pub fn draft_points(&self) -> &[Point] {
        self.draft
            .as_ref()
            .map(|d| d.points.as_slice())
            .unwrap_or(&[])
    }

    pub fn marquee_screen_rect(&self) -> Option<(f32, f32, f32, f32)> {
        match &self.drag {
            Some(Drag::Marquee { start, current, .. }) => {
                Some((start.0, start.1, current.0, current.1))
            }
            _ => None,
        }
    }

    pub fn draft_tool(&self) -> Option<Tool> {
        self.draft.as_ref().map(|d| d.tool)
    }

    pub fn ruler_segments(&self) -> &[(Point, Point)] {
        &self.ruler_segments
    }

    pub fn view_overlay(&self) -> Option<&ViewOverlay> {
        self.view_overlay_for_pane(self.active_pane())
    }

    pub fn view_overlay_for_pane(&self, pane: usize) -> Option<&ViewOverlay> {
        self.view_overlay[pane.min(1)].as_ref()
    }

    pub fn set_view_overlay(&mut self, overlay: Option<ViewOverlay>) {
        self.view_overlay = [overlay.clone(), overlay];
    }

    pub fn set_view_overlay_for_pane(&mut self, pane: usize, overlay: Option<ViewOverlay>) {
        self.view_overlay[pane.min(1)] = overlay;
    }

    /// True while the scene must follow the pointer (drag, draft, or pending component).
    pub fn scene_follows_pointer(&self) -> bool {
        self.drag.is_some() || self.draft.is_some() || self.pending_follow
    }

    pub fn selection_props_form(&self) -> Vec<crate::properties::PropFormField> {
        let refs: Vec<&Primitive> = self
            .selected
            .iter()
            .filter_map(|&i| self.doc.primitives.get(i))
            .collect();
        selection_props_form(&refs)
    }

    pub fn apply_selection_props_patch(&mut self, patch: &PropPatch) -> Result<bool, EditorError> {
        if self.selected.is_empty() {
            return Ok(false);
        }
        let mut targets: Vec<Primitive> = self
            .selected
            .iter()
            .filter_map(|&i| self.doc.primitives.get(i).cloned())
            .collect();
        if !apply_selection_props(&mut targets, patch) {
            return Ok(false);
        }
        if patch.layer.is_some() {
            for p in &mut targets {
                p.set_layer(self.doc.layers.clamp_id(p.layer()));
            }
        }
        crate::ext::emit_edit(self, crate::ext::EditEvent::GeometryBegin);
        self.push_undo();
        for (idx, &i) in self.selected.iter().enumerate() {
            if let Some(slot) = self.doc.primitives.get_mut(i) {
                if let Some(updated) = targets.get(idx) {
                    *slot = updated.clone();
                }
            }
        }
        crate::ext::emit_edit(self, crate::ext::EditEvent::GeometryEnd);
        Ok(true)
    }

    pub fn load_text(&mut self, text: &str) -> Result<(), crate::parse::ParseError> {
        let (doc, project) = crate::parse::parse_document_with_project_library(text)?;
        self.doc = doc;
        self.set_project_library(project);
        if self.doc.inferred_layers {
            let mut max = 0;
            for sheet in &self.doc.sheets {
                max = max.max(crate::library::max_used_layer_index(
                    &sheet.primitives,
                    &self.libs,
                ));
            }
            self.doc.layers.ensure_len(max + 1);
        }
        self.component_edit = None;
        self.split = false;
        self.active_pane = 0;
        self.panes = [PaneState::new(), PaneState::new()];
        self.clear_history();
        self.selected.clear();
        self.drag = None;
        self.draft = None;
        self.ruler_segments.clear();
        self.clamp_current_layer();
        self.sync_view_sheet();
        self.fit_view(800.0, 600.0);
        Ok(())
    }
}
