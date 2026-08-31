use fidocad_core::parse::builtin_libraries;
use fidocad_core::serialize::{serialize_clipboard, serialize_document};
use fidocad_core::{Editor, LayerId, SaveOptions, TextEditSession, Tool};
use fidocad_gpu::tessellate::{
    scene_to_svg, scene_to_thumb_svg, tessellate_editor, tessellate_primitives, tessellate_view,
};
#[cfg(target_arch = "wasm32")]
use fidocad_gpu::renderer::Renderer;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct App {
    editor: Editor,
    #[cfg(target_arch = "wasm32")]
    renderer: Option<Renderer>,
    width: f32,
    height: f32,
    show_grid: bool,
    locale: String,
    theme: String,
    dirty: bool,
}

#[derive(Serialize)]
struct Status {
    tool: String,
    layer: u8,
    x: i32,
    y: i32,
    xmm: f64,
    ymm: f64,
    zoom: f32,
    pcb: bool,
    n: usize,
    selected: usize,
    can_undo: bool,
    can_redo: bool,
    title: String,
    snap: i32,
    grid: i32,
    pending_macro: Option<String>,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> App {
        App {
            editor: Editor::new(builtin_libraries()),
            #[cfg(target_arch = "wasm32")]
            renderer: None,
            width: 800.0,
            height: 600.0,
            show_grid: true,
            locale: "it".into(),
            theme: "light".into(),
            dirty: true,
        }
    }

    #[wasm_bindgen]
    pub fn attach_canvas(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        #[cfg(target_arch = "wasm32")]
        {
            self.renderer = Some(Renderer::from_canvas(&canvas).map_err(JsValue::from)?);
            self.apply_theme();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = canvas;
        }
        self.dirty = true;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, w: f32, h: f32) {
        self.width = w.max(1.0);
        self.height = h.max(1.0);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn render(&mut self) {
        let scene = tessellate_view(&self.editor, Some((self.width, self.height)));
        #[cfg(target_arch = "wasm32")]
        if let Some(r) = self.renderer.as_mut() {
            r.draw(
                &scene,
                self.editor.pan,
                self.editor.zoom,
                (self.width, self.height),
                self.editor.doc.grid as f32,
                self.show_grid,
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = scene;
        self.dirty = false;
    }

    #[wasm_bindgen]
    pub fn load_fcd(&mut self, text: &str) -> Result<(), JsValue> {
        self.editor
            .load_text(text)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.editor.fit_view(self.width, self.height);
        self.dirty = true;
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
            &self.editor.doc,
            SaveOptions {
                split_nonstandard_macros: self.editor.split_nonstandard,
            },
            Some(&self.editor.libs),
        )
    }

    #[wasm_bindgen]
    pub fn clipboard_fcd(&self) -> String {
        let prims: Vec<_> = self
            .editor
            .selected
            .iter()
            .filter_map(|&i| self.editor.doc.primitives.get(i).cloned())
            .collect();
        if prims.is_empty() {
            serialize_document(&self.editor.doc, SaveOptions::default(), Some(&self.editor.libs))
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
            self.editor.zoom,
            self.editor.pan,
        )
    }

    #[wasm_bindgen]
    pub fn pointer_down(&mut self, sx: f32, sy: f32, shift: bool, pan: bool) {
        let w = self.editor.screen_to_world(sx, sy);
        if matches!(self.editor.tool, Tool::Select) && pan == false {
            // snapshot before potential move
            if self.editor.selected.is_empty() {
                // hit may select
            } else {
                self.editor.snapshot_before_move();
            }
        }
        self.editor.pointer_down(w, (sx, sy), shift, pan);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn pointer_move(&mut self, sx: f32, sy: f32) {
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.pointer_move(w, (sx, sy));
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn pointer_up(&mut self, sx: f32, sy: f32) {
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.pointer_up(w);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn dblclick(&mut self, sx: f32, sy: f32) -> String {
        let w = self.editor.screen_to_world(sx, sy);
        if let Some(session) = self.editor.begin_text_edit_at(w) {
            self.dirty = true;
            return text_edit_json(&self.editor, session);
        }
        self.dirty = true;
        "null".into()
    }

    #[wasm_bindgen]
    pub fn begin_selected_text_edit(&mut self) -> String {
        match self.editor.begin_text_edit_selected() {
            Some(session) => {
                self.dirty = true;
                text_edit_json(&self.editor, session)
            }
            None => "null".into(),
        }
    }

    #[wasm_bindgen]
    pub fn commit_text_edit(&mut self, text: &str) {
        self.editor.commit_text_edit(text.to_string());
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn cancel_text_edit(&mut self) {
        self.editor.cancel_text_edit();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn world_to_screen_json(&self, wx: f32, wy: f32) -> String {
        let (x, y) = self.editor.world_to_screen(wx, wy);
        format!("{{\"x\":{x},\"y\":{y},\"zoom\":{}}}", self.editor.zoom)
    }

    #[wasm_bindgen]
    pub fn wheel(&mut self, sx: f32, sy: f32, delta: f32) {
        self.editor.wheel_zoom((sx, sy), delta);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn key(&mut self, key: &str, meta: bool) -> bool {
        let handled = match key {
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
        };
        if handled {
            self.dirty = true;
        }
        handled
    }

    #[wasm_bindgen]
    pub fn set_tool(&mut self, id: &str) {
        self.editor.tool = Tool::from_id(id);
        self.editor.cancel_draft();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn set_layer(&mut self, n: u8) {
        self.editor.layer = LayerId(n.min(15));
        if !self.editor.selected.is_empty() {
            self.editor.set_selected_layer(self.editor.layer);
        }
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn set_pcb_mode(&mut self, on: bool) {
        self.editor.doc.pcb_mode = on;
    }

    #[wasm_bindgen]
    pub fn set_grid(&mut self, n: i32) {
        self.editor.doc.grid = n.clamp(1, 40);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn set_snap(&mut self, n: i32) {
        self.editor.doc.snap = n.clamp(0, 20);
    }

    #[wasm_bindgen]
    pub fn set_show_grid(&mut self, on: bool) {
        self.show_grid = on;
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn set_filled(&mut self, on: bool) {
        self.editor.filled = on;
    }

    #[wasm_bindgen]
    pub fn set_track_width(&mut self, w: i32) {
        self.editor.set_track_width_selected(w);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn set_split_macros(&mut self, on: bool) {
        self.editor.split_nonstandard = on;
    }

    #[wasm_bindgen]
    pub fn set_pending_macro(&mut self, name: &str) {
        self.editor.pending_macro = Some(name.to_string());
        self.editor.tool = Tool::Macro;
        self.editor.clear_hover();
    }

    #[wasm_bindgen]
    pub fn place_macro_at(&mut self, name: &str, sx: f32, sy: f32) {
        self.editor.pending_macro = Some(name.to_string());
        self.editor.tool = Tool::Macro;
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.insert_pending_macro_at(w);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn clear_hover(&mut self) {
        self.editor.clear_hover();
        self.dirty = true;
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
        serde_json::to_string(&serde_json::json!({
            "svg": cur.svg,
            "ox": cur.ox,
            "oy": cur.oy,
            "w": cur.w,
            "h": cur.h,
        }))
        .unwrap_or_else(|_| "{}".into())
    }

    fn macro_scene(&self, name: &str) -> fidocad_gpu::Scene {
        use fidocad_core::geom::Transform;
        use fidocad_core::library::expand_macro;
        use fidocad_core::MACRO_ORIGIN;
        let Some((_, def)) = self.editor.libs.lookup(name) else {
            return fidocad_gpu::Scene::default();
        };
        let prims = expand_macro(
            def,
            Transform {
                origin: MACRO_ORIGIN,
                rotations: 0,
                mirrored: false,
            },
            &self.editor.libs,
            0,
        );
        tessellate_primitives(&prims, &self.editor.doc.layers, self.editor.canvas_dark)
    }

    #[wasm_bindgen]
    pub fn set_pending_text(&mut self, text: &str) {
        self.editor.pending_text = text.to_string();
        if !self.editor.selected.is_empty() {
            self.editor.replace_selected_text(text.to_string());
            self.dirty = true;
        }
    }

    #[wasm_bindgen]
    pub fn undo(&mut self) {
        self.editor.undo();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) {
        self.editor.redo();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn rotate(&mut self) {
        self.editor.rotate_selected();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn mirror(&mut self) {
        self.editor.mirror_selected();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn fit(&mut self) {
        self.editor.fit_view(self.width, self.height);
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn new_doc(&mut self) {
        self.editor = Editor::new(builtin_libraries());
        self.apply_theme();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn set_locale(&mut self, loc: &str) {
        self.locale = loc.to_string();
    }

    #[wasm_bindgen]
    pub fn set_theme(&mut self, theme: &str) {
        self.theme = theme.to_string();
        self.apply_theme();
        self.dirty = true;
    }

    #[wasm_bindgen]
    pub fn set_layer_show(&mut self, n: u8, show: bool) {
        if (n as usize) < 16 {
            self.editor.doc.layers.layers[n as usize].show = show;
            self.dirty = true;
        }
    }

    #[wasm_bindgen]
    pub fn set_layer_print(&mut self, n: u8, print: bool) {
        if (n as usize) < 16 {
            self.editor.doc.layers.layers[n as usize].print = print;
        }
    }

    #[wasm_bindgen]
    pub fn set_layer_name(&mut self, n: u8, name: &str) {
        if (n as usize) < 16 {
            self.editor.doc.layers.layers[n as usize].name = name.to_string();
        }
    }

    #[wasm_bindgen]
    pub fn set_layer_color(&mut self, n: u8, r: u8, g: u8, b: u8) {
        if (n as usize) < 16 {
            self.editor.doc.layers.layers[n as usize].color = [r, g, b];
            self.dirty = true;
        }
    }

    #[wasm_bindgen]
    pub fn status_json(&self) -> String {
        let hover = self.editor.hover.unwrap_or(fidocad_core::Point::new(0, 0));
        let st = Status {
            tool: self.editor.tool.id().into(),
            layer: self.editor.layer.0,
            x: hover.x,
            y: hover.y,
            xmm: fidocad_core::Document::lu_to_mm(hover.x),
            ymm: fidocad_core::Document::lu_to_mm(hover.y),
            zoom: self.editor.zoom,
            pcb: self.editor.doc.pcb_mode,
            n: self.editor.doc.primitives.len(),
            selected: self.editor.selected.len(),
            can_undo: self.editor.can_undo(),
            can_redo: self.editor.can_redo(),
            title: self.editor.doc.title.clone(),
            snap: self.editor.doc.snap,
            grid: self.editor.doc.grid,
            pending_macro: if self.editor.tool == Tool::Macro {
                self.editor.pending_macro.clone()
            } else {
                None
            },
        };
        serde_json::to_string(&st).unwrap_or_else(|_| "{}".into())
    }

    #[wasm_bindgen]
    pub fn library_json(&self) -> String {
        serde_json::to_string(&self.editor.libs.tree()).unwrap_or_else(|_| "[]".into())
    }

    #[wasm_bindgen]
    pub fn layers_json(&self) -> String {
        serde_json::to_string(&self.editor.doc.layers).unwrap_or_else(|_| "{}".into())
    }

    #[wasm_bindgen]
    pub fn selection_props_json(&self) -> String {
        if let Some(&i) = self.editor.selected.first() {
            if let Some(p) = self.editor.doc.primitives.get(i) {
                return serde_json::to_string(p).unwrap_or_else(|_| "null".into());
            }
        }
        "null".into()
    }

    fn apply_theme(&mut self) {
        self.editor.canvas_dark = self.theme == "dark";
        #[cfg(target_arch = "wasm32")]
        if let Some(r) = self.renderer.as_mut() {
            if self.editor.canvas_dark {
                r.set_theme([0.10, 0.09, 0.08], [0.32, 0.26, 0.22]);
            } else {
                r.set_theme([0.97, 0.95, 0.92], [0.78, 0.72, 0.66]);
            }
        }
    }
}

#[derive(Serialize)]
struct TextEditJson {
    index: usize,
    text: String,
    wx: i32,
    wy: i32,
    sx: i32,
    sy: i32,
    angle: i32,
    style: u32,
    screen_x: f32,
    screen_y: f32,
    zoom: f32,
}

fn text_edit_json(ed: &Editor, session: TextEditSession) -> String {
    let (screen_x, screen_y) = ed.world_to_screen(session.wx as f32, session.wy as f32);
    serde_json::to_string(&TextEditJson {
        index: session.index,
        text: session.text,
        wx: session.wx,
        wy: session.wy,
        sx: session.sx,
        sy: session.sy,
        angle: session.angle,
        style: session.style,
        screen_x,
        screen_y,
        zoom: ed.zoom,
    })
    .unwrap_or_else(|_| "null".into())
}
