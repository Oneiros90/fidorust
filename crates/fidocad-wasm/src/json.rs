//! JSON DTOs for the WASM ↔ UI bridge.

use fidocad_core::{Document, Editor, TextEditSession, Tool};
use serde::{Deserialize, Serialize};

pub fn to_json<T: Serialize>(v: &T, fallback: &'static str) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| fallback.into())
}

#[derive(Serialize)]
pub struct StatusDto {
    pub tool: String,
    pub layer: u8,
    pub x: i32,
    pub y: i32,
    pub xmm: f64,
    pub ymm: f64,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub pcb: bool,
    pub n: usize,
    pub selected: usize,
    pub can_undo: bool,
    pub can_redo: bool,
    pub title: String,
    pub snap: i32,
    pub snap_y: i32,
    pub grid: i32,
    pub grid_y: i32,
    pub snap_enable: bool,
    pub show_grid: bool,
    pub hide_macro_origin: bool,
    pub pending_macro: Option<String>,
}

impl StatusDto {
    pub fn from_editor(ed: &Editor, show_grid: bool) -> Self {
        let hover = ed.hover().unwrap_or(fidocad_core::Point::new(0, 0));
        Self {
            tool: ed.tool().id().into(),
            layer: ed.layer().0,
            x: hover.x,
            y: hover.y,
            xmm: Document::lu_to_mm(hover.x),
            ymm: Document::lu_to_mm(hover.y),
            zoom: ed.zoom(),
            pan_x: ed.pan().0,
            pan_y: ed.pan().1,
            pcb: ed.doc().pcb_mode,
            n: ed.doc().primitives.len(),
            selected: ed.selected().len(),
            can_undo: ed.can_undo(),
            can_redo: ed.can_redo(),
            title: ed.doc().title.clone(),
            snap: ed.doc().snap,
            snap_y: ed.doc().snap_y,
            grid: ed.doc().grid,
            grid_y: ed.doc().grid_y,
            snap_enable: ed.snap_enable(),
            show_grid,
            hide_macro_origin: ed.hide_macro_origin(),
            pending_macro: if ed.tool() == Tool::Macro {
                ed.pending_macro().map(str::to_string)
            } else {
                None
            },
        }
    }
}

#[derive(Serialize)]
pub struct TextEditDto {
    #[serde(flatten)]
    pub session: TextEditSession,
    pub screen_x: f32,
    pub screen_y: f32,
    pub zoom: f32,
}

pub fn text_edit_json(ed: &Editor, session: TextEditSession) -> String {
    let (screen_x, screen_y) = ed.world_to_screen(session.wx as f32, session.wy as f32);
    to_json(
        &TextEditDto {
            session,
            screen_x,
            screen_y,
            zoom: ed.zoom(),
        },
        "null",
    )
}

#[derive(Serialize)]
pub struct MacroCursorDto {
    pub svg: String,
    pub ox: f32,
    pub oy: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ExportSvgOpts {
    #[serde(default)]
    pub margin_lu: f32,
    #[serde(default)]
    pub bw: bool,
    #[serde(default)]
    pub layers: Vec<ExportLayerOpt>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExportLayerOpt {
    pub show: bool,
    #[serde(default)]
    pub invert: bool,
}
