//! Interactive editor: view, history, tools, layers, and text edit.

mod components;
mod history;
mod layers;
mod selection;
mod text_edit;
mod tools;
mod view;

pub use tools::{DblClickAction, EditorError, TextEditSession, Tool};

use crate::consts::{
    DEFAULT_PAD_DX, DEFAULT_PAD_DY, DEFAULT_PAD_HOLE, DEFAULT_TEXT_SX, DEFAULT_TEXT_SY,
    DEFAULT_TRACK_WIDTH, FIT_MARGIN, GRID_MAX, GRID_MIN, SNAP_MAX, SNAP_MIN, ZOOM_MAX_WHEEL,
    ZOOM_TOOL_FACTOR,
};
use crate::document::{Document, ProjectSettings};
use crate::geom::Point;
use crate::hit::{hit_test, marquee_select, HitQuery};
use crate::layers::LayerId;
use crate::library::LibrarySet;
use crate::primitive::{
    Bezier, Connection, Ellipse, Line, PadStyle, PcbPad, PcbTrack, Poly, Primitive, Rect, Text,
    DEFAULT_FONT,
};
use crate::properties::{apply_selection_props, selection_props_form, PropPatch};

use components::ComponentEditSession;
use history::HistorySnapshot;
use tools::{Draft, Drag};

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
    track_width: i32,
    pad_dx: i32,
    pad_dy: i32,
    pad_hole: i32,
    pad_style: PadStyle,
    pending_component: Option<String>,
    pending_rotations: u8,
    /// Ghost-at-cursor and RMB rotate only while a library drag is in progress.
    pending_follow: bool,
    pending_text: String,
    /// Primitive index whose glyphs are hidden while the UI overlay edits them.
    editing_text: Option<usize>,
    undo: Vec<HistorySnapshot>,
    redo: Vec<HistorySnapshot>,
    /// Pre-move / pre-handle snapshot; committed on pointer_up if the document changed.
    drag_checkpoint: Option<Document>,
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
            track_width: DEFAULT_TRACK_WIDTH,
            pad_dx: DEFAULT_PAD_DX,
            pad_dy: DEFAULT_PAD_DY,
            pad_hole: DEFAULT_PAD_HOLE,
            pad_style: PadStyle::Oval,
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
        }
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
        self.cancel_draft();
        if tool != Tool::Component {
            self.pending_component = None;
            self.pending_follow = false;
        }
    }

    pub fn set_filled(&mut self, on: bool) {
        self.doc.default_filled = on;
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
    }

    pub fn filled(&self) -> bool {
        self.doc.default_filled
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

    pub fn set_hover(&mut self, hover: Option<Point>) {
        self.hover = hover;
        if hover.is_none() {
            self.hover_hit = false;
            self.hover_index = None;
        }
    }

    fn pick_at(&self, x: f64, y: f64) -> Option<crate::hit::Hit> {
        hit_test(
            &self.doc.primitives,
            &self.libs,
            &self.doc.layers,
            &self.selected,
            HitQuery {
                x,
                y,
                zoom: self.zoom,
                stroke_w: self.doc.stroke_width() as f64,
            },
        )
    }

    fn refresh_hover_hit(&mut self, x: f64, y: f64) {
        let hit = self.pick_at(x, y);
        self.hover_hit = hit.is_some();
        self.hover_index = if self.tool == Tool::Select {
            hit.map(|h| h.index)
        } else {
            None
        };
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
        self.push_undo();
        self.doc.apply_project_settings(s);
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
        self.push_undo();
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
        self.track_width = w.max(1);
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

    /// Start a selection marquee at `screen`, even if a primitive is under the cursor.
    /// Used by right-drag; left-click empty space shares the same path.
    pub fn begin_marquee(&mut self, screen: (f32, f32), shift: bool) {
        if self.tool != Tool::Select {
            return;
        }
        if !shift {
            self.selected.clear();
        }
        let kept = self.selected.clone();
        self.drag = Some(Drag::Marquee {
            start: screen,
            current: screen,
            kept,
        });
        self.apply_marquee_hits();
    }

    fn apply_marquee_hits(&mut self) {
        let Some(Drag::Marquee {
            start,
            current,
            kept,
        }) = &self.drag
        else {
            return;
        };
        let (start, current, kept) = (*start, *current, kept.clone());
        let a = self.screen_to_world(start.0, start.1);
        let b = self.screen_to_world(current.0, current.1);
        let extra = marquee_select(&self.doc.primitives, &self.libs, a, b);
        self.selected.clear();
        self.selected.extend(kept);
        for i in extra {
            if !self.selected.contains(&i) {
                self.selected.push(i);
            }
        }
    }

    pub fn draft_tool(&self) -> Option<Tool> {
        self.draft.as_ref().map(|d| d.tool)
    }

    pub fn ruler_segments(&self) -> &[(Point, Point)] {
        &self.ruler_segments
    }

    /// True while the scene must follow the pointer (drag, draft, or pending component).
    pub fn scene_follows_pointer(&self) -> bool {
        self.drag.is_some() || self.draft.is_some() || self.pending_follow
    }

    pub fn pointer_down(&mut self, world: Point, screen: (f32, f32), shift: bool, pan_mod: bool) {
        self.pointer_down_at(
            world.x as f64,
            world.y as f64,
            world,
            screen,
            shift,
            pan_mod,
        );
    }

    pub fn pointer_down_at(
        &mut self,
        hx: f64,
        hy: f64,
        world: Point,
        screen: (f32, f32),
        shift: bool,
        pan_mod: bool,
    ) {
        let pt = self.snap_pt(world);
        self.refresh_hover_hit(hx, hy);
        if matches!(&self.drag, Some(Drag::PlaceClone { .. })) {
            if !pan_mod {
                self.place_pending_clone_at(pt);
            }
            return;
        }
        if pan_mod || self.tool == Tool::Pan {
            self.drag = Some(Drag::Pan {
                start_screen: screen,
                pan0: self.pan,
            });
            return;
        }
        match self.tool {
            Tool::Select => {
                if let Some(hit) = self.pick_at(hx, hy) {
                    if !shift && !self.selected.contains(&hit.index) {
                        self.selected.clear();
                    }
                    if !self.selected.contains(&hit.index) {
                        self.selected.push(hit.index);
                    }
                    if let Some(h) = hit.handle {
                        self.begin_drag_checkpoint();
                        self.drag = Some(Drag::Handle {
                            index: hit.index,
                            handle: h,
                        });
                    } else {
                        self.begin_drag_checkpoint();
                        self.drag = Some(Drag::Move {
                            start: pt,
                            last: pt,
                            duplicate: false,
                        });
                    }
                } else {
                    self.begin_marquee(screen, shift);
                }
            }
            Tool::Connection => {
                self.push_undo();
                self.doc.insert(Primitive::Connection(Connection {
                    pos: pt,
                    layer: self.layer,
                }));
            }
            Tool::PcbPad => {
                self.push_undo();
                self.doc.insert(Primitive::PcbPad(PcbPad {
                    pos: pt,
                    dx: self.pad_dx,
                    dy: self.pad_dy,
                    hole: self.pad_hole,
                    style: self.pad_style,
                    layer: self.layer,
                }));
            }
            Tool::Text => {
                self.push_undo();
                self.doc.insert(Primitive::Text(Text {
                    pos: pt,
                    sy: DEFAULT_TEXT_SY,
                    sx: DEFAULT_TEXT_SX,
                    angle: 0,
                    style: 0,
                    layer: self.layer,
                    font: DEFAULT_FONT.into(),
                    text: self.pending_text.clone(),
                    simple: false,
                }));
            }
            Tool::Component => {
                let _ = self.insert_pending_component_at(pt);
            }
            Tool::Zoom => {
                self.zoom = (self.zoom * ZOOM_TOOL_FACTOR).min(ZOOM_MAX_WHEEL);
            }
            Tool::Line | Tool::Rect | Tool::Ellipse | Tool::PcbTrack | Tool::Ruler => {
                if let Some(d) = self.draft.as_mut() {
                    Self::set_draft_endpoint(d, pt);
                } else {
                    self.draft = Some(Draft {
                        tool: self.tool,
                        points: vec![pt],
                    });
                }
            }
            Tool::Poly | Tool::Bezier => {
                if let Some(d) = self.draft.as_mut() {
                    d.points.push(pt);
                } else {
                    self.draft = Some(Draft {
                        tool: self.tool,
                        points: vec![pt],
                    });
                }
            }
            Tool::Pan => {}
        }
    }

    pub fn pointer_move(&mut self, world: Point, screen: (f32, f32)) {
        self.pointer_move_at(world.x as f64, world.y as f64, world, screen);
    }

    pub fn pointer_move_at(&mut self, hx: f64, hy: f64, world: Point, screen: (f32, f32)) {
        let pt = self.snap_pt(world);
        self.hover = Some(pt);
        if self.drag.is_none() {
            self.refresh_hover_hit(hx, hy);
        }
        if let Some(d) = &mut self.draft {
            if d.tool.is_two_point_draw() || d.tool == Tool::Ruler {
                Self::set_draft_endpoint(d, pt);
            } else if matches!(d.tool, Tool::Poly | Tool::Bezier) {
                match d.points.len() {
                    0 | 1 => d.points.push(pt),
                    _ => {}
                }
            }
        }
        match &self.drag {
            Some(Drag::Move {
                last, duplicate, ..
            }) => {
                let duplicate = *duplicate;
                let delta = Point::new(pt.x - last.x, pt.y - last.y);
                if delta != Point::new(0, 0) {
                    if !duplicate {
                        self.translate_selected(delta);
                    }
                    if let Some(Drag::Move { last, .. }) = &mut self.drag {
                        *last = pt;
                    }
                }
            }
            Some(Drag::Handle { index, handle }) => {
                let (i, h) = (*index, *handle);
                if let Some(p) = self.doc.primitives.get_mut(i) {
                    p.set_control_point(h, pt);
                }
            }
            Some(Drag::Pan { start_screen, pan0 }) => {
                self.pan = (
                    pan0.0 + (screen.0 - start_screen.0),
                    pan0.1 + (screen.1 - start_screen.1),
                );
            }
            Some(Drag::Marquee { .. }) => {
                if let Some(Drag::Marquee { current, .. }) = &mut self.drag {
                    *current = screen;
                }
                self.apply_marquee_hits();
            }
            Some(Drag::PlaceClone { .. }) => self.move_place_clone(pt),
            None => {}
        }
    }

    pub fn pointer_up(&mut self, world: Point) {
        self.pointer_up_at(world.x as f64, world.y as f64, world);
    }

    pub fn pointer_up_at(&mut self, hx: f64, hy: f64, world: Point) {
        let pt = self.snap_pt(world);
        if matches!(&self.drag, Some(Drag::PlaceClone { .. })) {
            return;
        }
        if matches!(&self.drag, Some(Drag::Marquee { .. })) {
            self.apply_marquee_hits();
        }
        match self.drag.take() {
            Some(Drag::Marquee { .. }) => {
                self.drag_checkpoint = None;
                self.refresh_hover_hit(hx, hy);
                return;
            }
            Some(Drag::Move {
                start,
                duplicate: true,
                ..
            }) => {
                let dx = pt.x - start.x;
                let dy = pt.y - start.y;
                if dx != 0 || dy != 0 {
                    self.insert_translated_clones(dx, dy);
                }
            }
            _ => {}
        }
        self.commit_drag_checkpoint();
        if let Some(d) = self.draft.take() {
            if d.tool.is_two_point_draw() {
                if !self.commit_two_point_draft(&d) {
                    if let Some(&start) = d.points.first() {
                        self.draft = Some(Draft {
                            tool: d.tool,
                            points: vec![start],
                        });
                    }
                }
            } else {
                match d.tool {
                    Tool::Ruler => {
                        if d.points.len() >= 2 && d.points[0] != d.points[1] {
                            self.ruler_segments.push((d.points[0], d.points[1]));
                        } else {
                            self.draft = Some(d);
                        }
                    }
                    Tool::Poly => {
                        let mut pts = d.points;
                        pts.push(pt);
                        if pts.len() >= 2 {
                            self.draft = Some(Draft {
                                tool: Tool::Poly,
                                points: pts,
                            });
                        }
                    }
                    Tool::Bezier => {
                        let mut pts = d.points;
                        pts.push(pt);
                        let ready = pts.len() >= 4;
                        self.draft = Some(Draft {
                            tool: Tool::Bezier,
                            points: pts.clone(),
                        });
                        if ready {
                            self.push_undo();
                            self.doc.insert(Primitive::Bezier(Bezier {
                                p0: pts[0],
                                p1: pts[1],
                                p2: pts[2],
                                p3: pts[3],
                                layer: self.layer,
                            }));
                            self.draft = None;
                        }
                    }
                    _ => {}
                }
            }
        }
        self.refresh_hover_hit(hx, hy);
    }

    fn set_draft_endpoint(d: &mut Draft, pt: Point) {
        if d.points.len() < 2 {
            d.points.push(pt);
        } else {
            d.points[1] = pt;
        }
    }

    fn commit_two_point_draft(&mut self, d: &Draft) -> bool {
        if d.points.len() < 2 || d.points[0] == d.points[1] {
            return false;
        }
        let a = d.points[0];
        let b = d.points[1];
        let prim = match d.tool {
            Tool::Line => Primitive::Line(Line {
                a,
                b,
                layer: self.layer,
            }),
            Tool::Rect => Primitive::Rect(Rect {
                a,
                b,
                filled: self.doc.default_filled,
                layer: self.layer,
            }),
            Tool::Ellipse => Primitive::Ellipse(Ellipse {
                a,
                b,
                filled: self.doc.default_filled,
                layer: self.layer,
            }),
            Tool::PcbTrack => Primitive::PcbTrack(PcbTrack {
                a,
                b,
                width: self.track_width,
                layer: self.layer,
            }),
            _ => return false,
        };
        self.push_undo();
        self.doc.insert(prim);
        true
    }

    pub fn finish_poly(&mut self) {
        if let Some(d) = self.draft.take() {
            if d.tool == Tool::Poly && d.points.len() >= 2 {
                self.push_undo();
                self.doc.insert(Primitive::Poly(Poly {
                    pts: d.points,
                    filled: self.doc.default_filled,
                    layer: self.layer,
                }));
            }
        }
    }

    pub fn cancel_draft(&mut self) {
        self.draft = None;
        self.drag = None;
        self.commit_drag_checkpoint();
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
        self.push_undo();
        for (idx, &i) in self.selected.iter().enumerate() {
            if let Some(slot) = self.doc.primitives.get_mut(i) {
                if let Some(updated) = targets.get(idx) {
                    *slot = updated.clone();
                }
            }
        }
        Ok(true)
    }

    pub fn load_text(&mut self, text: &str) -> Result<(), crate::parse::ParseError> {
        let (doc, project) = crate::parse::parse_document_with_project_library(text)?;
        self.doc = doc;
        self.set_project_library(project);
        if self.doc.inferred_layers {
            let max = crate::library::max_used_layer_index(&self.doc.primitives, &self.libs);
            self.doc.layers.ensure_len(max + 1);
        }
        self.component_edit = None;
        self.clear_history();
        self.selected.clear();
        self.drag = None;
        self.draft = None;
        self.ruler_segments.clear();
        self.clamp_current_layer();
        self.fit_view(800.0, 600.0);
        Ok(())
    }
}
