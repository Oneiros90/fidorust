//! Layer CRUD helpers used by the WASM façade.

use std::str::FromStr;

use crate::json::to_json;
use crate::App;

enum DeleteLayerMode {
    Objects,
    Move,
}

impl FromStr for DeleteLayerMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "move" => Self::Move,
            "objects" => Self::Objects,
            _ => Self::Objects,
        })
    }
}

pub(crate) fn set_layer_show(app: &mut App, n: u8, show: bool) {
    let _ = app.editor.set_layer_show(n as usize, show);
}

pub(crate) fn set_layer_name(app: &mut App, n: u8, name: &str) {
    app.editor.set_layer_name(n as usize, name.to_string());
}

pub(crate) fn set_layer_color(app: &mut App, n: u8, r: u8, g: u8, b: u8, a: u8) {
    let _ = app.editor.set_layer_color(n as usize, [r, g, b, a]);
}

pub(crate) fn add_layer(app: &mut App) {
    let _ = app.editor.add_layer();
}

/// `mode` is `"objects"` (delete primitives) or `"move"` (relocate to `move_to`).
pub(crate) fn delete_layer(app: &mut App, n: u8, mode: &str, move_to: u8) {
    let dest = match mode.parse().unwrap_or(DeleteLayerMode::Objects) {
        DeleteLayerMode::Move => Some(move_to as usize),
        DeleteLayerMode::Objects => None,
    };
    let _ = app.editor.delete_layer(n as usize, dest);
}

pub(crate) fn reorder_layer(app: &mut App, from: u8, to: u8) {
    let _ = app.editor.reorder_layer(from as usize, to as usize);
}

pub(crate) fn layer_object_count(app: &App, n: u8) -> u32 {
    app.editor.layer_object_count(n as usize) as u32
}

pub(crate) fn layers_json(app: &App) -> String {
    to_json(&app.editor.doc().layers, "{}")
}
