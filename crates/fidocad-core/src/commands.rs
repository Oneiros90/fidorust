//! Editing tools, undo, snap.

use crate::document::Document;
use crate::geom::{snap, Point, Transform};
use crate::hit::{hit_test, marquee_select};
use crate::layers::LayerId;
use crate::library::LibrarySet;
use crate::primitive::{PadStyle, Primitive};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tool {
    Select,
    Line,
    Rect,
    Ellipse,
    Poly,
    Bezier,
    Text,
    Connection,
    PcbTrack,
    PcbPad,
    Macro,
    Zoom,
    Pan,
}

impl Tool {
    pub fn from_id(id: &str) -> Self {
        match id {
            "line" => Self::Line,
            "rect" => Self::Rect,
            "ellipse" => Self::Ellipse,
            "poly" => Self::Poly,
            "bezier" => Self::Bezier,
            "text" => Self::Text,
            "connection" => Self::Connection,
            "pcb-track" => Self::PcbTrack,
            "pcb-pad" => Self::PcbPad,
            "macro" => Self::Macro,
            "zoom" => Self::Zoom,
            "pan" => Self::Pan,
            _ => Self::Select,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Line => "line",
            Self::Rect => "rect",
            Self::Ellipse => "ellipse",
            Self::Poly => "poly",
            Self::Bezier => "bezier",
            Self::Text => "text",
            Self::Connection => "connection",
            Self::PcbTrack => "pcb-track",
            Self::PcbPad => "pcb-pad",
            Self::Macro => "macro",
            Self::Zoom => "zoom",
            Self::Pan => "pan",
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Command {
    Insert(Primitive),
    Delete(Vec<(usize, Primitive)>),
    Replace(usize, Primitive, Primitive),
    Move {
        indices: Vec<usize>,
        delta: Point,
    },
}

#[derive(Clone, Debug)]
struct Draft {
    tool: Tool,
    points: Vec<Point>,
}

#[derive(Clone, Debug)]
pub struct Editor {
    pub doc: Document,
    pub libs: LibrarySet,
    pub tool: Tool,
    pub layer: LayerId,
    pub selected: Vec<usize>,
    pub zoom: f32,
    pub pan: (f32, f32),
    pub split_nonstandard: bool,
    pub filled: bool,
    pub track_width: i32,
    pub pad_dx: i32,
    pub pad_dy: i32,
    pub pad_hole: i32,
    pub pad_style: PadStyle,
    pub pending_macro: Option<String>,
    pub pending_text: String,
    /// Primitive index whose glyphs are hidden while the UI overlay edits them.
    pub editing_text: Option<usize>,
    undo: Vec<Document>,
    redo: Vec<Document>,
    draft: Option<Draft>,
    drag: Option<Drag>,
    pub hover: Option<Point>,
    /// Screen theme only: invert near-black layer colours when drawing. Not saved.
    pub canvas_dark: bool,
}

/// Layout of a text primitive for an in-scene editor overlay.
#[derive(Clone, Debug, Serialize)]
pub struct TextEditSession {
    pub index: usize,
    pub text: String,
    pub wx: i32,
    pub wy: i32,
    pub sx: i32,
    pub sy: i32,
    pub angle: i32,
    pub style: u32,
}

#[derive(Clone, Debug)]
enum Drag {
    Move { last: Point },
    Marquee { start: Point, current: Point },
    Handle { index: usize, handle: usize },
    Pan { start_screen: (f32, f32), pan0: (f32, f32) },
}

impl Editor {
    pub fn new(libs: LibrarySet) -> Self {
        Self {
            doc: Document::default(),
            libs,
            tool: Tool::Select,
            layer: LayerId(0),
            selected: Vec::new(),
            zoom: 4.0,
            pan: (40.0, 40.0),
            split_nonstandard: true,
            filled: false,
            track_width: 4,
            pad_dx: 18,
            pad_dy: 18,
            pad_hole: 8,
            pad_style: PadStyle::Oval,
            pending_macro: None,
            pending_text: "TEXT".into(),
            editing_text: None,
            undo: Vec::new(),
            redo: Vec::new(),
            draft: None,
            drag: None,
            hover: None,
            canvas_dark: false,
        }
    }

    pub fn snap_pt(&self, p: Point) -> Point {
        Point::new(snap(p.x, self.doc.snap), snap(p.y, self.doc.snap))
    }

    fn push_undo(&mut self) {
        self.undo.push(self.doc.clone());
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(self.doc.clone());
            self.doc = prev;
            self.selected.clear();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.doc.clone());
            self.doc = next;
            self.selected.clear();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn delete_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.push_undo();
        let mut sel = self.selected.clone();
        sel.sort_unstable();
        sel.reverse();
        for i in sel {
            self.doc.remove(i);
        }
        self.selected.clear();
    }

    pub fn select_all(&mut self) {
        self.selected = (0..self.doc.primitives.len()).collect();
    }

    pub fn rotate_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.push_undo();
        let origin = {
            let bb = self.doc.selected_aabb(&self.selected, &self.libs);
            Point::new((bb.min.x + bb.max.x) / 2, (bb.min.y + bb.max.y) / 2)
        };
        for &i in &self.selected {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                p.transform(|q| q.rotate90_cw(origin));
                if let Primitive::Macro { rotations, .. } = p {
                    *rotations = (*rotations + 1) % 4;
                }
            }
        }
    }

    pub fn mirror_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.push_undo();
        let origin = {
            let bb = self.doc.selected_aabb(&self.selected, &self.libs);
            (bb.min.x + bb.max.x) / 2
        };
        for &i in &self.selected {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                p.transform(|q| q.mirror_vertical(origin));
                if let Primitive::Macro { mirrored, .. } = p {
                    *mirrored = !*mirrored;
                }
            }
        }
    }

    pub fn set_selected_layer(&mut self, layer: LayerId) {
        self.push_undo();
        for &i in &self.selected {
            if let Some(p) = self.doc.primitives.get_mut(i) {
                p.set_layer(layer);
            }
        }
    }

    pub fn draft_points(&self) -> &[Point] {
        self.draft.as_ref().map(|d| d.points.as_slice()).unwrap_or(&[])
    }

    /// Live rubber-band while dragging a selection marquee.
    pub fn marquee_rect(&self) -> Option<(Point, Point)> {
        match &self.drag {
            Some(Drag::Marquee { start, current }) => Some((*start, *current)),
            _ => None,
        }
    }

    pub fn draft_tool(&self) -> Option<Tool> {
        self.draft.as_ref().map(|d| d.tool)
    }

    pub fn pending_macro_preview(&self) -> Vec<Primitive> {
        if self.tool != Tool::Macro {
            return Vec::new();
        }
        let Some(name) = self.pending_macro.as_deref() else {
            return Vec::new();
        };
        let Some(pos) = self.hover else {
            return Vec::new();
        };
        let Some((_, def)) = self.libs.lookup(name) else {
            return Vec::new();
        };
        crate::library::expand_macro(
            def,
            Transform {
                origin: pos,
                rotations: 0,
                mirrored: false,
            },
            &self.libs,
            0,
        )
    }

    pub fn clear_hover(&mut self) {
        self.hover = None;
    }

    pub fn insert_pending_macro_at(&mut self, world: Point) {
        let Some(name) = self.pending_macro.clone() else {
            return;
        };
        self.push_undo();
        let pt = self.snap_pt(world);
        let standard = self.libs.is_standard(&name);
        self.doc.insert(Primitive::Macro {
            pos: pt,
            rotations: 0,
            mirrored: false,
            name,
            standard,
        });
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
                        self.drag = Some(Drag::Handle {
                            index: hit.index,
                            handle: h,
                        });
                    } else {
                        self.drag = Some(Drag::Move { last: pt });
                    }
                } else {
                    if !shift {
                        self.selected.clear();
                    }
                    self.drag = Some(Drag::Marquee {
                        start: world,
                        current: world,
                    });
                }
            }
            Tool::Connection => {
                self.push_undo();
                self.doc.insert(Primitive::Connection {
                    pos: pt,
                    layer: self.layer,
                });
            }
            Tool::PcbPad => {
                self.push_undo();
                self.doc.insert(Primitive::PcbPad {
                    pos: pt,
                    dx: self.pad_dx,
                    dy: self.pad_dy,
                    hole: self.pad_hole,
                    style: self.pad_style,
                    layer: self.layer,
                });
            }
            Tool::Text => {
                self.push_undo();
                self.doc.insert(Primitive::Text {
                    pos: pt,
                    sy: 4,
                    sx: 3,
                    angle: 0,
                    style: 0,
                    layer: self.layer,
                    font: "Courier New".into(),
                    text: self.pending_text.clone(),
                    simple: false,
                });
            }
            Tool::Macro => {
                self.insert_pending_macro_at(pt);
            }
            Tool::Zoom => {
                self.zoom = (self.zoom * 1.3).min(40.0);
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
            match d.points.len() {
                0 => d.points.push(pt),
                1 => d.points.push(pt),
                n => {
                    if d.tool == Tool::Poly || d.tool == Tool::Bezier {
                        // rubber-band last
                    } else {
                        d.points[n - 1] = pt;
                    }
                }
            }
            if matches!(d.tool, Tool::Line | Tool::Rect | Tool::Ellipse | Tool::PcbTrack) {
                if d.points.len() == 1 {
                    d.points.push(pt);
                } else {
                    d.points[1] = pt;
                }
            }
        }
        match &self.drag {
            Some(Drag::Move { last }) => {
                let delta = Point::new(pt.x - last.x, pt.y - last.y);
                if delta != Point::new(0, 0) {
                    if self.undo.last().map(|d| d.primitives.len()) != Some(self.doc.primitives.len())
                    {
                        // already snapshot
                    }
                    let sel = self.selected.clone();
                    for i in sel {
                        if let Some(p) = self.doc.primitives.get_mut(i) {
                            p.transform(|q| q.add(delta));
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
                    *current = world;
                }
            }
            None => {}
        }
    }

    pub fn pointer_up(&mut self, world: Point) {
        let pt = self.snap_pt(world);
        if let Some(Drag::Marquee { start, current }) = self.drag.take() {
            let extra = marquee_select(&self.doc.primitives, &self.libs, start, current);
            for i in extra {
                if !self.selected.contains(&i) {
                    self.selected.push(i);
                }
            }
            return;
        }
        self.drag = None;
        if let Some(d) = self.draft.take() {
            match d.tool {
                // Original FidoCAD ignores a second click that coincides with the first
                // (ElecDrawView.cpp: "il click deve essere IGNORATO").
                Tool::Line if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::Line {
                        a: d.points[0],
                        b: d.points[1],
                        layer: self.layer,
                    });
                }
                Tool::Rect if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::Rect {
                        a: d.points[0],
                        b: d.points[1],
                        filled: self.filled,
                        layer: self.layer,
                    });
                }
                Tool::Ellipse if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::Ellipse {
                        a: d.points[0],
                        b: d.points[1],
                        filled: self.filled,
                        layer: self.layer,
                    });
                }
                Tool::PcbTrack if d.points.len() >= 2 && d.points[0] != d.points[1] => {
                    self.push_undo();
                    self.doc.insert(Primitive::PcbTrack {
                        a: d.points[0],
                        b: d.points[1],
                        width: self.track_width,
                        layer: self.layer,
                    });
                }
                Tool::Poly => {
                    let mut pts = d.points;
                    pts.push(pt);
                    if pts.len() >= 2 {
                        // wait for double-click via finish_poly
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
                        self.doc.insert(Primitive::Bezier {
                            p0: pts[0],
                            p1: pts[1],
                            p2: pts[2],
                            p3: pts[3],
                            layer: self.layer,
                        });
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
                self.doc.insert(Primitive::Poly {
                    pts: d.points,
                    filled: self.filled,
                    layer: self.layer,
                });
            }
        }
    }

    pub fn cancel_draft(&mut self) {
        self.draft = None;
        self.drag = None;
    }

    pub fn wheel_zoom(&mut self, screen: (f32, f32), delta: f32) {
        let old = self.zoom;
        let factor = if delta < 0.0 { 1.12 } else { 1.0 / 1.12 };
        self.zoom = (self.zoom * factor).clamp(0.4, 40.0);
        // Keep world point under cursor.
        let wx = (screen.0 - self.pan.0) / old;
        let wy = (screen.1 - self.pan.1) / old;
        self.pan.0 = screen.0 - wx * self.zoom;
        self.pan.1 = screen.1 - wy * self.zoom;
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> Point {
        Point::new(
            ((sx - self.pan.0) / self.zoom).round() as i32,
            ((sy - self.pan.1) / self.zoom).round() as i32,
        )
    }

    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (wx * self.zoom + self.pan.0, wy * self.zoom + self.pan.1)
    }

    pub fn text_edit_session(&self, index: usize) -> Option<TextEditSession> {
        match self.doc.primitives.get(index)? {
            Primitive::Text {
                pos,
                sy,
                sx,
                angle,
                style,
                text,
                ..
            } => Some(TextEditSession {
                index,
                text: text.clone(),
                wx: pos.x,
                wy: pos.y,
                sx: *sx,
                sy: *sy,
                angle: *angle,
                style: *style,
            }),
            _ => None,
        }
    }

    fn begin_text_edit_index(&mut self, index: usize) -> Option<TextEditSession> {
        let session = self.text_edit_session(index)?;
        self.selected = vec![index];
        self.drag = None;
        self.editing_text = Some(index);
        Some(session)
    }

    /// Double-click handler: finish a polygon draft, or start in-place text edit.
    pub fn begin_text_edit_at(&mut self, world: Point) -> Option<TextEditSession> {
        if self.draft.as_ref().is_some_and(|d| d.tool == Tool::Poly) {
            self.finish_poly();
            return None;
        }
        if self.draft.is_some() {
            return None;
        }
        self.drag = None;
        let hit = hit_test(
            &self.doc.primitives,
            &self.libs,
            &self.doc.layers,
            world,
            self.zoom,
        )?;
        self.begin_text_edit_index(hit.index)
    }

    pub fn begin_text_edit_selected(&mut self) -> Option<TextEditSession> {
        let index = *self.selected.first()?;
        self.begin_text_edit_index(index)
    }

    pub fn commit_text_edit(&mut self, text: String) {
        let Some(index) = self.editing_text.take() else {
            return;
        };
        if !self.selected.contains(&index) {
            self.selected = vec![index];
        }
        self.replace_selected_text(text);
    }

    pub fn cancel_text_edit(&mut self) {
        self.editing_text = None;
    }

    pub fn replace_selected_text(&mut self, text: String) {
        let changed = self.selected.iter().any(|&i| {
            matches!(
                self.doc.primitives.get(i),
                Some(Primitive::Text { text: t, .. }) if *t != text
            )
        });
        if !changed {
            return;
        }
        self.push_undo();
        for &i in &self.selected {
            if let Some(Primitive::Text { text: t, .. }) = self.doc.primitives.get_mut(i) {
                *t = text.clone();
            }
        }
    }

    pub fn set_track_width_selected(&mut self, w: i32) {
        self.track_width = w.max(1);
        self.push_undo();
        for &i in &self.selected {
            if let Some(Primitive::PcbTrack { width, .. }) = self.doc.primitives.get_mut(i) {
                *width = self.track_width;
            }
        }
    }

    pub fn load_text(&mut self, text: &str) -> Result<(), crate::parse::ParseError> {
        self.push_undo();
        self.doc = crate::parse::parse_document(text)?;
        self.selected.clear();
        self.fit_view(800.0, 600.0);
        Ok(())
    }

    pub fn fit_view(&mut self, w: f32, h: f32) {
        let bb = self.doc.aabb(&self.libs);
        if bb.is_empty() {
            self.zoom = 4.0;
            self.pan = (40.0, 40.0);
            return;
        }
        let margin = 40.0;
        let zw = (w - margin * 2.0) / bb.width().max(1) as f32;
        let zh = (h - margin * 2.0) / bb.height().max(1) as f32;
        self.zoom = zw.min(zh).clamp(0.4, 20.0);
        self.pan = (
            margin - bb.min.x as f32 * self.zoom,
            margin - bb.min.y as f32 * self.zoom,
        );
    }

    pub fn snapshot_before_move(&mut self) {
        self.push_undo();
    }
}
