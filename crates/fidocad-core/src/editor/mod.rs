//! Interactive editor: view, history, tools, layers, and text edit.

mod components;
mod history;
mod layers;
mod selection;
mod text_edit;
mod tools;
mod view;

pub use tools::{EditorError, TextEditSession, Tool};

use crate::consts::{
    DEFAULT_PAD_DX, DEFAULT_PAD_DY, DEFAULT_PAD_HOLE, DEFAULT_TEXT_SX, DEFAULT_TEXT_SY,
    DEFAULT_TRACK_WIDTH, FIT_MARGIN, ZOOM_MAX_WHEEL, ZOOM_TOOL_FACTOR,
};
use crate::document::Document;
use crate::geom::Point;
use crate::hit::{hit_test, marquee_select};
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

#[derive(Clone, Debug)]
pub struct Editor {
    doc: Document,
    libs: LibrarySet,
    tool: Tool,
    layer: LayerId,
    selected: Vec<usize>,
    zoom: f32,
    pan: (f32, f32),
    split_nonstandard: bool,
    filled: bool,
    track_width: i32,
    pad_dx: i32,
    pad_dy: i32,
    pad_hole: i32,
    pad_style: PadStyle,
    pending_component: Option<String>,
    pending_rotations: u8,
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
    /// Screen theme only: preview stroke colour. Not saved.
    canvas_dark: bool,
    /// Original `m_bSnapEnable`. When false, coordinates are not quantized.
    snap_enable: bool,
    /// When true, skip the red origin handle on components.
    hide_component_origin: bool,
    libs_rev: u32,
    component_edit: Option<ComponentEditSession>,
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
            split_nonstandard: false,
            filled: false,
            track_width: DEFAULT_TRACK_WIDTH,
            pad_dx: DEFAULT_PAD_DX,
            pad_dy: DEFAULT_PAD_DY,
            pad_hole: DEFAULT_PAD_HOLE,
            pad_style: PadStyle::Oval,
            pending_component: None,
            pending_rotations: 0,
            pending_text: "TEXT".into(),
            editing_text: None,
            undo: Vec::new(),
            redo: Vec::new(),
            drag_checkpoint: None,
            layer_color_edit: None,
            draft: None,
            drag: None,
            hover: None,
            canvas_dark: false,
            snap_enable: true,
            hide_component_origin: true,
            libs_rev: 0,
            component_edit: None,
        }
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
        self.cancel_draft();
    }

    pub fn set_filled(&mut self, on: bool) {
        self.filled = on;
    }

    pub fn set_layer(&mut self, n: u8) {
        let max = self.doc.layers.len().saturating_sub(1) as u8;
        self.layer = LayerId(n.min(max));
        if !self.selected.is_empty() {
            self.set_selected_layer(self.layer);
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
        self.filled
    }

    pub fn hover(&self) -> Option<Point> {
        self.hover
    }

    pub fn set_hover(&mut self, hover: Option<Point>) {
        self.hover = hover;
    }

    pub fn canvas_dark(&self) -> bool {
        self.canvas_dark
    }

    pub fn set_canvas_dark(&mut self, on: bool) {
        self.canvas_dark = on;
    }

    pub fn snap_enable(&self) -> bool {
        self.snap_enable
    }

    pub fn set_snap_enable(&mut self, on: bool) {
        self.snap_enable = on;
    }

    pub fn hide_component_origin(&self) -> bool {
        self.hide_component_origin
    }

    pub fn set_hide_component_origin(&mut self, on: bool) {
        self.hide_component_origin = on;
    }

    pub fn split_nonstandard(&self) -> bool {
        self.split_nonstandard
    }

    pub fn set_split_nonstandard(&mut self, on: bool) {
        self.split_nonstandard = on;
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
        let x = x.clamp(1, 40);
        let y = y.clamp(1, 40);
        if self.doc.grid == x && self.doc.grid_y == y {
            return;
        }
        self.push_undo();
        self.doc.grid = x;
        self.doc.grid_y = y;
    }

    pub fn set_snap(&mut self, x: i32, y: i32) {
        let x = x.clamp(1, 20);
        let y = y.clamp(1, 20);
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
            Some(Drag::Marquee { start, current }) => {
                Some((start.0, start.1, current.0, current.1))
            }
            _ => None,
        }
    }

    pub fn draft_tool(&self) -> Option<Tool> {
        self.draft.as_ref().map(|d| d.tool)
    }

    pub fn pointer_down(&mut self, world: Point, screen: (f32, f32), shift: bool, pan_mod: bool) {
        let pt = self.snap_pt(world);
        if pan_mod || self.tool == Tool::Pan {
            self.drag = Some(Drag::Pan {
                start_screen: screen,
                pan0: self.pan,
            });
            return;
        }
        match self.tool {
            Tool::Select => {
                if let Some(hit) = hit_test(
                    &self.doc.primitives,
                    &self.libs,
                    &self.doc.layers,
                    world,
                    self.zoom,
                ) {
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
                        self.drag = Some(Drag::Move { last: pt });
                    }
                } else {
                    if !shift {
                        self.selected.clear();
                    }
                    self.drag = Some(Drag::Marquee {
                        start: screen,
                        current: screen,
                    });
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
                self.insert_pending_component_at(pt);
            }
            Tool::Zoom => {
                self.zoom = (self.zoom * ZOOM_TOOL_FACTOR).min(ZOOM_MAX_WHEEL);
            }
            Tool::Line | Tool::Rect | Tool::Ellipse | Tool::PcbTrack => {
                self.draft = Some(Draft {
                    tool: self.tool,
                    points: vec![pt],
                });
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
        let pt = self.snap_pt(world);
        self.hover = Some(pt);
        if let Some(d) = &mut self.draft {
            if matches!(
                d.tool,
                Tool::Line | Tool::Rect | Tool::Ellipse | Tool::PcbTrack
            ) {
                if d.points.len() < 2 {
                    d.points.push(pt);
                } else {
                    d.points[1] = pt;
                }
            } else if matches!(d.tool, Tool::Poly | Tool::Bezier) {
                match d.points.len() {
                    0 | 1 => d.points.push(pt),
                    _ => {}
                }
            }
        }
        match &self.drag {
            Some(Drag::Move { last }) => {
                let delta = Point::new(pt.x - last.x, pt.y - last.y);
                if delta != Point::new(0, 0) {
                    let sel = self.selected.clone();
                    for i in sel {
                        if let Some(p) = self.doc.primitives.get_mut(i) {
                            p.transform(|q| q + delta);
                        }
                    }
                    if let Some(Drag::Move { last }) = &mut self.drag {
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
            }
            None => {}
        }
    }

    pub fn pointer_up(&mut self, world: Point) {
        let pt = self.snap_pt(world);
        if let Some(Drag::Marquee { start, current }) = self.drag.take() {
            self.drag_checkpoint = None;
            let a = self.screen_to_world(start.0, start.1);
            let b = self.screen_to_world(current.0, current.1);
            let extra = marquee_select(&self.doc.primitives, &self.libs, a, b);
            for i in extra {
                if !self.selected.contains(&i) {
                    self.selected.push(i);
                }
            }
            return;
        }
        self.drag = None;
        self.commit_drag_checkpoint();
        if let Some(d) = self.draft.take() {
            match d.tool {
                Tool::Line if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::Line(Line {
                        a: d.points[0],
                        b: d.points[1],
                        layer: self.layer,
                    }));
                }
                Tool::Rect if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::Rect(Rect {
                        a: d.points[0],
                        b: d.points[1],
                        filled: self.filled,
                        layer: self.layer,
                    }));
                }
                Tool::Ellipse if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::Ellipse(Ellipse {
                        a: d.points[0],
                        b: d.points[1],
                        filled: self.filled,
                        layer: self.layer,
                    }));
                }
                Tool::PcbTrack if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::PcbTrack(PcbTrack {
                        a: d.points[0],
                        b: d.points[1],
                        width: self.track_width,
                        layer: self.layer,
                    }));
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

    pub fn finish_poly(&mut self) {
        if let Some(d) = self.draft.take() {
            if d.tool == Tool::Poly && d.points.len() >= 2 {
                self.push_undo();
                self.doc.insert(Primitive::Poly(Poly {
                    pts: d.points,
                    filled: self.filled,
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
        self.component_edit = None;
        self.clear_history();
        self.selected.clear();
        self.drag = None;
        self.draft = None;
        self.clamp_current_layer();
        self.fit_view(800.0, 600.0);
        Ok(())
    }
}
