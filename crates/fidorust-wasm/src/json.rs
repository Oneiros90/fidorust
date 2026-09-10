//! JSON DTOs for the WASM ↔ UI bridge.

use fidorust_core::{DblClickAction, Document, Editor, TextEditSession, Tool};
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
    pub hide_component_origin: bool,
    pub stroke_hundredths: i32,
    pub default_filled: bool,
    pub pending_component: Option<String>,
    pub duplicate_drag: bool,
    pub can_create_component: bool,
    pub can_split_component: bool,
    pub can_edit_component: bool,
    pub editing_component: Option<String>,
    pub editing_component_name: Option<String>,
    pub editing_component_dirty: bool,
    pub libs_rev: u32,
}

impl StatusDto {
    pub fn from_editor(ed: &Editor) -> Self {
        let hover = ed.hover().unwrap_or(fidorust_core::Point::new(0, 0));
        let editing = ed
            .editing_component()
            .map(|(stem, key)| format!("{stem}.{key}"));
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
            show_grid: ed.show_grid(),
            hide_component_origin: ed.hide_component_origin(),
            stroke_hundredths: ed.doc().stroke_hundredths,
            default_filled: ed.filled(),
            pending_component: if ed.tool() == Tool::Component {
                ed.pending_component().map(str::to_string)
            } else {
                None
            },
            duplicate_drag: ed.duplicate_drag(),
            can_create_component: ed.can_create_component(),
            can_split_component: ed.can_split_component(),
            can_edit_component: ed.can_edit_component(),
            editing_component: editing,
            editing_component_name: ed.editing_component_name(),
            editing_component_dirty: ed.editing_component_dirty(),
            libs_rev: ed.libs_rev(),
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

pub fn dblclick_json(ed: &Editor, action: DblClickAction) -> String {
    match action {
        DblClickAction::TextEdit(session) => text_edit_json(ed, session),
        DblClickAction::OpenProperties => "{\"action\":\"properties\"}".into(),
        DblClickAction::None => "null".into(),
    }
}

#[derive(Serialize)]
pub struct ComponentCursorDto {
    pub svg: String,
    pub ox: f32,
    pub oy: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Serialize)]
pub struct CreatedComponentDto {
    pub stem: String,
    pub key: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnresolvedComponentDto {
    pub name: String,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserLibBlob {
    pub stem: String,
    pub title: String,
    pub fcl: String,
    #[serde(default)]
    pub aliases: Vec<String>,
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
