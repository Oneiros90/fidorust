//! WASM façade: JSON glue around `fidocad-core::Editor` and the GPU backend.

mod json;
mod render_backend;

use fidocad_core::parse::builtin_libraries;
use fidocad_core::serialize::{serialize_clipboard, serialize_document};
use fidocad_core::{Editor, EditorError, PropPatch, SaveOptions, Tool};
use fidocad_gpu::tessellate::{
    scene_to_svg, scene_to_thumb_svg, tessellate_editor, tessellate_primitives,
};
use render_backend::Backend;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use json::{text_edit_json, to_json, MacroCursorDto, StatusDto};

fn to_js(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

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

#[wasm_bindgen]
pub struct App {
    editor: Editor,
    backend: Backend,
    width: f32,
    height: f32,
    show_grid: bool,
    #[allow(dead_code)]
    locale: String,
    theme: String,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> App {
        App {
            editor: Editor::new(builtin_libraries()),
            backend: Backend::new(),
            width: 800.0,
            height: 600.0,
            show_grid: true,
            locale: "it".into(),
            theme: "light".into(),
        }
    }

    #[wasm_bindgen]
    pub fn attach_canvas(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        self.backend.attach_canvas(canvas)?;
        self.backend.apply_theme(&mut self.editor, &self.theme);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, w: f32, h: f32) {
        self.width = w.max(1.0);
        self.height = h.max(1.0);
    }

    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.backend
            .draw(&self.editor, (self.width, self.height), self.show_grid);
    }

    #[wasm_bindgen]
    pub fn load_fcd(&mut self, text: &str) -> Result<(), JsValue> {
        self.editor.load_text(text).map_err(to_js)?;
        self.editor.fit_view(self.width, self.height);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn load_fcd_bytes(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let text = fidocad_core::parse::decode_bytes(bytes);
        self.load_fcd(&text)
    }

    #[wasm_bindgen]
    pub fn save_fcd(&self) -> String {
        serialize_document(
            self.editor.doc(),
            SaveOptions {
                split_nonstandard_macros: self.editor.split_nonstandard(),
            },
            Some(self.editor.libs()),
        )
    }

    #[wasm_bindgen]
    pub fn clipboard_fcd(&self) -> String {
        let prims: Vec<_> = self
            .editor
            .selected()
            .iter()
            .filter_map(|&i| self.editor.doc().primitives.get(i).cloned())
            .collect();
        if prims.is_empty() {
            serialize_document(
                self.editor.doc(),
                SaveOptions::default(),
                Some(self.editor.libs()),
            )
        } else {
            serialize_clipboard(&prims)
        }
    }

    #[wasm_bindgen]
    pub fn export_svg(&self) -> String {
        let scene = tessellate_editor(&self.editor);
        scene_to_svg(
            &scene,
            self.width,
            self.height,
            self.editor.zoom(),
            self.editor.pan(),
        )
    }

    #[wasm_bindgen]
    pub fn pointer_down(&mut self, sx: f32, sy: f32, shift: bool, pan: bool) {
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.pointer_down(w, (sx, sy), shift, pan);
    }

    #[wasm_bindgen]
    pub fn pointer_move(&mut self, sx: f32, sy: f32) {
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.pointer_move(w, (sx, sy));
    }

    #[wasm_bindgen]
    pub fn pointer_up(&mut self, sx: f32, sy: f32) {
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.pointer_up(w);
    }

    #[wasm_bindgen]
    pub fn dblclick(&mut self, sx: f32, sy: f32) -> String {
        let w = self.editor.screen_to_world(sx, sy);
        if let Some(session) = self.editor.begin_text_edit_at(w) {
            return text_edit_json(&self.editor, session);
        }
        "null".into()
    }

    #[wasm_bindgen]
    pub fn begin_selected_text_edit(&mut self) -> String {
        match self.editor.begin_text_edit_selected() {
            Some(session) => text_edit_json(&self.editor, session),
            None => "null".into(),
        }
    }

    #[wasm_bindgen]
    pub fn commit_text_edit(&mut self, text: &str) {
        self.editor.commit_text_edit(text.to_string());
    }

    #[wasm_bindgen]
    pub fn cancel_text_edit(&mut self) {
        self.editor.cancel_text_edit();
    }

    #[wasm_bindgen]
    pub fn world_to_screen_json(&self, wx: f32, wy: f32) -> String {
        let (x, y) = self.editor.world_to_screen(wx, wy);
        // `format!` (not serde_json) so whole-number f32 keep a trailing `.0`.
        format!("{{\"x\":{x},\"y\":{y},\"zoom\":{}}}", self.editor.zoom())
    }

    #[wasm_bindgen]
    pub fn wheel(&mut self, sx: f32, sy: f32, delta: f32) {
        self.editor.wheel_zoom((sx, sy), delta);
    }

    #[wasm_bindgen]
    pub fn key(&mut self, key: &str, meta: bool) -> bool {
        match key {
            "Delete" | "Backspace" => {
                self.editor.delete_selected();
                true
            }
            "Escape" => {
                self.editor.cancel_draft();
                true
            }
            "a" | "A" if meta => {
                self.editor.select_all();
                true
            }
            "z" | "Z" if meta => {
                self.editor.undo();
                true
            }
            "y" | "Y" if meta => {
                self.editor.redo();
                true
            }
            "r" | "R" if meta => {
                self.editor.rotate_selected();
                true
            }
            "m" | "M" if meta => {
                self.editor.mirror_selected();
                true
            }
            "Enter" if meta => true,
            _ => false,
        }
    }

    #[wasm_bindgen]
    pub fn set_tool(&mut self, id: &str) {
        self.editor.set_tool(Tool::from_id(id));
    }

    #[wasm_bindgen]
    pub fn set_layer(&mut self, n: u8) {
        self.editor.set_layer(n);
    }

    #[wasm_bindgen]
    pub fn set_pcb_mode(&mut self, on: bool) {
        self.editor.set_pcb_mode(on);
    }

    #[wasm_bindgen]
    pub fn set_grid(&mut self, x: i32, y: i32) {
        self.editor.set_grid(x, y);
    }

    #[wasm_bindgen]
    pub fn set_snap(&mut self, x: i32, y: i32) {
        self.editor.set_snap(x, y);
    }

    #[wasm_bindgen]
    pub fn set_snap_enable(&mut self, on: bool) {
        self.editor.set_snap_enable(on);
    }

    #[wasm_bindgen]
    pub fn set_hide_macro_origin(&mut self, on: bool) {
        self.editor.set_hide_macro_origin(on);
    }

    #[wasm_bindgen]
    pub fn set_show_grid(&mut self, on: bool) {
        self.show_grid = on;
    }

    #[wasm_bindgen]
    pub fn set_filled(&mut self, on: bool) {
        self.editor.set_filled(on);
    }

    #[wasm_bindgen]
    pub fn set_track_width(&mut self, w: i32) {
        self.editor.set_track_width(w);
    }

    #[wasm_bindgen]
    pub fn selection_props_form_json(&self) -> String {
        to_json(&self.editor.selection_props_form(), "[]")
    }

    #[wasm_bindgen]
    pub fn apply_selection_props(&mut self, patch_json: &str) -> Result<(), JsValue> {
        let patch: PropPatch = serde_json::from_str(patch_json)
            .map_err(|e| to_js(EditorError::InvalidPatch(e.to_string())))?;
        self.editor
            .apply_selection_props_patch(&patch)
            .map_err(to_js)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_split_macros(&mut self, on: bool) {
        self.editor.set_split_nonstandard(on);
    }

    #[wasm_bindgen]
    pub fn set_pending_macro(&mut self, name: &str) {
        self.editor.set_pending_macro(Some(name.to_string()));
        self.editor.adopt_macro_tool();
        self.editor.clear_hover();
    }

    #[wasm_bindgen]
    pub fn place_macro_at(&mut self, name: &str, sx: f32, sy: f32) {
        self.editor.set_pending_macro(Some(name.to_string()));
        self.editor.adopt_macro_tool();
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.insert_pending_macro_at(w);
    }

    #[wasm_bindgen]
    pub fn pointer_right(&mut self, sx: f32, sy: f32) -> bool {
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.right_click(w)
    }

    #[wasm_bindgen]
    pub fn prepare_context_menu(&mut self, sx: f32, sy: f32) {
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.prepare_context_menu(w);
    }

    #[wasm_bindgen]
    pub fn invert_selection(&mut self) {
        self.editor.invert_selection();
    }

    #[wasm_bindgen]
    pub fn split_selected_macros(&mut self) {
        self.editor.split_selected_macros();
    }

    #[wasm_bindgen]
    pub fn paste_selection(&mut self, text: &str) {
        let _ = self.editor.paste_primitives(text);
    }

    #[wasm_bindgen]
    pub fn clear_hover(&mut self) {
        self.editor.clear_hover();
    }

    #[wasm_bindgen]
    pub fn macro_preview_svg(&self, name: &str) -> String {
        let scene = self.macro_scene(name);
        scene_to_thumb_svg(&scene, 40.0)
    }

    #[wasm_bindgen]
    pub fn macro_cursor_json(&self, name: &str) -> String {
        use fidocad_core::MACRO_ORIGIN;
        use fidocad_gpu::scene_to_cursor_svg;
        let scene = self.macro_scene(name);
        let cur = scene_to_cursor_svg(&scene, MACRO_ORIGIN);
        to_json(
            &MacroCursorDto {
                svg: cur.svg,
                ox: cur.ox,
                oy: cur.oy,
                w: cur.w,
                h: cur.h,
            },
            "{}",
        )
    }

    fn macro_scene(&self, name: &str) -> fidocad_gpu::Scene {
        use fidocad_core::geom::Transform;
        use fidocad_core::library::expand_macro;
        use fidocad_core::MACRO_ORIGIN;
        let Some((_, def)) = self.editor.libs().lookup(name) else {
            return fidocad_gpu::Scene::default();
        };
        let prims = expand_macro(
            def,
            Transform {
                origin: MACRO_ORIGIN,
                rotations: 0,
                mirrored: false,
            },
            self.editor.libs(),
            0,
        );
        tessellate_primitives(&prims, &self.editor.doc().layers, self.editor.canvas_dark())
    }

    #[wasm_bindgen]
    pub fn set_pending_text(&mut self, text: &str) {
        self.editor.set_pending_text(text.to_string());
        if !self.editor.selected().is_empty() {
            self.editor.replace_selected_text(text.to_string());
        }
    }

    #[wasm_bindgen]
    pub fn undo(&mut self) {
        self.editor.undo();
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) {
        self.editor.redo();
    }

    #[wasm_bindgen]
    pub fn rotate(&mut self) {
        self.editor.rotate_selected();
    }

    #[wasm_bindgen]
    pub fn mirror(&mut self) {
        self.editor.mirror_selected();
    }

    #[wasm_bindgen]
    pub fn fit(&mut self) {
        self.editor.fit_view(self.width, self.height);
    }

    #[wasm_bindgen]
    pub fn new_doc(&mut self) {
        self.editor = Editor::new(builtin_libraries());
        self.backend.apply_theme(&mut self.editor, &self.theme);
    }

    #[wasm_bindgen]
    pub fn set_locale(&mut self, loc: &str) {
        self.locale = loc.to_string();
    }

    #[wasm_bindgen]
    pub fn set_theme(&mut self, theme: &str) {
        self.theme = theme.to_string();
        self.backend.apply_theme(&mut self.editor, &self.theme);
    }

    #[wasm_bindgen]
    pub fn set_layer_show(&mut self, n: u8, show: bool) {
        let _ = self.editor.set_layer_show(n as usize, show);
    }

    #[wasm_bindgen]
    pub fn set_layer_name(&mut self, n: u8, name: &str) {
        self.editor.set_layer_name(n as usize, name.to_string());
    }

    #[wasm_bindgen]
    pub fn set_layer_color(&mut self, n: u8, r: u8, g: u8, b: u8) {
        let _ = self.editor.set_layer_color(n as usize, [r, g, b]);
    }

    #[wasm_bindgen]
    pub fn add_layer(&mut self) {
        let _ = self.editor.add_layer();
    }

    /// `mode` is `"objects"` (delete primitives) or `"move"` (relocate to `move_to`).
    #[wasm_bindgen]
    pub fn delete_layer(&mut self, n: u8, mode: &str, move_to: u8) {
        let dest = match mode.parse().unwrap_or(DeleteLayerMode::Objects) {
            DeleteLayerMode::Move => Some(move_to as usize),
            DeleteLayerMode::Objects => None,
        };
        let _ = self.editor.delete_layer(n as usize, dest);
    }

    #[wasm_bindgen]
    pub fn reorder_layer(&mut self, from: u8, to: u8) {
        let _ = self.editor.reorder_layer(from as usize, to as usize);
    }

    #[wasm_bindgen]
    pub fn layer_object_count(&self, n: u8) -> u32 {
        self.editor.layer_object_count(n as usize) as u32
    }

    #[wasm_bindgen]
    pub fn status_json(&self) -> String {
        to_json(&StatusDto::from_editor(&self.editor, self.show_grid), "{}")
    }

    #[wasm_bindgen]
    pub fn library_json(&self) -> String {
        to_json(&self.editor.libs().tree(), "[]")
    }

    #[wasm_bindgen]
    pub fn layers_json(&self) -> String {
        to_json(&self.editor.doc().layers, "{}")
    }

    #[wasm_bindgen]
    pub fn selection_props_json(&self) -> String {
        if let Some(&i) = self.editor.selected().first() {
            if let Some(p) = self.editor.doc().primitives.get(i) {
                return to_json(p, "null");
            }
        }
        "null".into()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
