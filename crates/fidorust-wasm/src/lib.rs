//! WASM façade: JSON glue around `fidorust-core::Editor` and the GPU backend.

mod json;
mod render_backend;

use fidorust_core::parse::{builtin_libraries, parse_library};
use fidorust_core::properties::{
    PropField, PropFieldKind, PropFieldValue, PropFormField, PropPatch,
};
use fidorust_core::serialize::{
    serialize_clipboard, serialize_document, serialize_document_with_policy, serialize_library,
    SaveLibraryPolicy,
};
use fidorust_core::{Editor, EditorError, LibraryKind, Tool};
use fidorust_gpu::tessellate::{export_svg, scene_to_thumb_svg, tessellate_primitives};
use render_backend::Backend;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use json::{
    dblclick_json, text_edit_json, to_json, ComponentCursorDto, CreatedComponentDto, ExportSvgOpts,
    StatusDto, UnresolvedComponentDto, UserLibBlob,
};

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
        self.backend.draw(
            &self.editor,
            (self.width, self.height),
            self.editor.show_grid(),
        );
    }

    #[wasm_bindgen]
    pub fn load_fcd(&mut self, text: &str) -> Result<(), JsValue> {
        self.editor.load_text(text).map_err(to_js)?;
        self.editor.fit_view(self.width, self.height);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn load_fcd_bytes(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let text = fidorust_core::parse::decode_bytes(bytes);
        self.load_fcd(&text)
    }

    #[wasm_bindgen]
    pub fn save_fcd(&self) -> String {
        serialize_document(self.editor.persistent_doc(), Some(self.editor.libs()))
    }

    #[wasm_bindgen]
    pub fn save_fcd_with_policy(&self, policy: &str) -> String {
        serialize_document_with_policy(
            self.editor.persistent_doc(),
            self.editor.libs(),
            SaveLibraryPolicy::from_str(policy).unwrap_or_default(),
        )
    }

    #[wasm_bindgen]
    pub fn uses_user_library_components(&self) -> bool {
        self.editor.uses_user_library_components()
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
            serialize_document(self.editor.doc(), Some(self.editor.libs()))
        } else {
            serialize_clipboard(&prims)
        }
    }

    /// `opts_json` is `{ margin_lu, bw, layers: [{ show, invert }] }`. Empty / invalid JSON uses document layers.
    #[wasm_bindgen]
    pub fn export_svg(&self, opts_json: &str) -> String {
        let opts: ExportSvgOpts = serde_json::from_str(opts_json).unwrap_or_default();
        let mut layers = self.editor.doc().layers.clone();
        if opts.layers.is_empty() {
            if opts.bw {
                for i in 0..layers.len() {
                    layers.update(i, |info| {
                        if info.show {
                            info.color = [0, 0, 0, info.color[3]];
                        }
                    });
                }
            }
        } else {
            for (i, overlay) in opts.layers.iter().enumerate() {
                layers.update(i, |info| {
                    info.show = overlay.show;
                    if opts.bw {
                        info.color = [0, 0, 0, info.color[3]];
                    } else if overlay.invert {
                        info.color = [
                            255 - info.color[0],
                            255 - info.color[1],
                            255 - info.color[2],
                            info.color[3],
                        ];
                    }
                });
            }
        }
        export_svg(
            &self.editor.doc().primitives,
            &layers,
            self.editor.libs(),
            opts.margin_lu.max(0.0),
            self.editor.doc().stroke_width(),
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
        let action = self.editor.handle_dblclick(w);
        dblclick_json(&self.editor, action)
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
    pub fn set_hide_component_origin(&mut self, on: bool) {
        self.editor.set_hide_component_origin(on);
    }

    #[wasm_bindgen]
    pub fn set_show_grid(&mut self, on: bool) {
        self.editor.set_show_grid(on);
    }

    #[wasm_bindgen]
    pub fn apply_project_settings(&mut self, json: &str) -> Result<(), JsValue> {
        let s: fidorust_core::ProjectSettings = serde_json::from_str(json).map_err(to_js)?;
        self.editor.apply_project_settings(s);
        Ok(())
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
    pub fn register_font(&mut self, name: &str, data: &[u8]) -> bool {
        fidorust_gpu::font::register_font(name, data)
    }

    #[wasm_bindgen]
    pub fn registered_fonts_json(&self) -> String {
        to_json(&fidorust_gpu::font::registered_families(), "[]")
    }

    #[wasm_bindgen]
    pub fn selection_props_form_json(&self) -> String {
        let mut fields = self.editor.selection_props_form();
        patch_font_face_choices(&mut fields);
        to_json(&fields, "[]")
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
    pub fn set_pending_component(&mut self, name: &str) {
        self.editor.set_pending_component(Some(name.to_string()));
        self.editor.adopt_component_tool();
        self.editor.clear_hover();
    }

    #[wasm_bindgen]
    pub fn set_pending_follow(&mut self, on: bool) {
        self.editor.set_pending_follow(on);
    }

    #[wasm_bindgen]
    pub fn place_component_at(&mut self, name: &str, sx: f32, sy: f32) {
        if self.editor.pending_component() != Some(name) {
            self.editor.set_pending_component(Some(name.to_string()));
        }
        self.editor.adopt_component_tool();
        let w = self.editor.screen_to_world(sx, sy);
        self.editor.place_dropped_component(w);
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
    pub fn split_selected_components(&mut self) {
        self.editor.split_selected_components();
    }

    #[wasm_bindgen]
    pub fn paste_selection(&mut self, text: &str) {
        let _ = self.editor.paste_primitives(text);
    }

    #[wasm_bindgen]
    pub fn duplicate_selection(&mut self) {
        self.editor.duplicate_selection();
    }

    #[wasm_bindgen]
    pub fn set_move_duplicate(&mut self, on: bool) {
        self.editor.set_move_duplicate(on);
    }

    #[wasm_bindgen]
    pub fn stamp_drag_copy(&mut self) -> bool {
        self.editor.stamp_drag_copy()
    }

    #[wasm_bindgen]
    pub fn clear_hover(&mut self) {
        self.editor.clear_hover();
    }

    #[wasm_bindgen]
    pub fn component_preview_svg(&self, name: &str) -> String {
        let scene = self.component_scene(name);
        scene_to_thumb_svg(&scene, 40.0)
    }

    #[wasm_bindgen]
    pub fn component_cursor_json(&self, name: &str) -> String {
        use fidorust_core::COMPONENT_ORIGIN;
        use fidorust_gpu::scene_to_cursor_svg;
        let scene = self.component_scene(name);
        let cur = scene_to_cursor_svg(&scene, COMPONENT_ORIGIN);
        to_json(
            &ComponentCursorDto {
                svg: cur.svg,
                ox: cur.ox,
                oy: cur.oy,
                w: cur.w,
                h: cur.h,
            },
            "{}",
        )
    }

    fn component_scene(&self, name: &str) -> fidorust_gpu::Scene {
        use fidorust_core::geom::Transform;
        use fidorust_core::library::expand_component;
        use fidorust_core::COMPONENT_ORIGIN;
        let Some((_, def)) = self.editor.libs().lookup(name) else {
            return fidorust_gpu::Scene::default();
        };
        let prims = expand_component(
            def,
            Transform {
                origin: COMPONENT_ORIGIN,
                rotations: 0,
                mirrored: false,
            },
            self.editor.libs(),
            0,
        );
        tessellate_primitives(&prims, &self.editor.doc().layers)
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
    pub fn set_view(&mut self, zoom: f32, pan_x: f32, pan_y: f32) {
        self.editor.set_view(zoom, (pan_x, pan_y));
    }

    #[wasm_bindgen]
    pub fn new_doc(&mut self) {
        let blob = self.user_libraries_blob();
        self.editor = Editor::new(builtin_libraries());
        self.load_user_libraries(&blob);
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
    pub fn set_layer_color(&mut self, n: u8, r: u8, g: u8, b: u8, a: u8) {
        let _ = self.editor.set_layer_color(n as usize, [r, g, b, a]);
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
        to_json(&StatusDto::from_editor(&self.editor), "{}")
    }

    #[wasm_bindgen]
    pub fn unresolved_components_json(&self) -> String {
        let items: Vec<UnresolvedComponentDto> = self
            .editor
            .unresolved_components()
            .into_iter()
            .map(|(name, count)| UnresolvedComponentDto { name, count })
            .collect();
        to_json(&items, "[]")
    }

    #[wasm_bindgen]
    pub fn library_json(&self) -> String {
        to_json(&self.editor.libs().tree(), "[]")
    }

    #[wasm_bindgen]
    pub fn user_libraries_json(&self) -> String {
        to_json(&self.editor.libs().user_library_list(), "[]")
    }

    #[wasm_bindgen]
    pub fn create_component_from_selection(&mut self, target: &str, display_name: &str) -> String {
        match self
            .editor
            .create_component_from_selection(target, display_name)
        {
            Some((stem, key)) => to_json(&CreatedComponentDto { stem, key }, "{}"),
            None => String::new(),
        }
    }

    #[wasm_bindgen]
    pub fn enter_component_edit(&mut self, stem: &str, key: &str) -> bool {
        let ok = self.editor.enter_component_edit(stem, key);
        if ok {
            self.editor.fit_view(self.width, self.height);
        }
        ok
    }

    #[wasm_bindgen]
    pub fn edit_selected_component(&mut self) -> bool {
        let Some((stem, key)) = self.editor.selected_editable_component() else {
            return false;
        };
        self.enter_component_edit(&stem, &key)
    }

    #[wasm_bindgen]
    pub fn save_component_edit(&mut self) -> bool {
        self.editor.save_component_edit()
    }

    #[wasm_bindgen]
    pub fn local_component_uses_nonzero_layers(&self, stem: &str, key: &str) -> bool {
        self.editor.local_component_uses_nonzero_layers(stem, key)
    }

    #[wasm_bindgen]
    pub fn editing_local_component_uses_nonzero_layers(&self) -> bool {
        self.editor.editing_local_component_uses_nonzero_layers()
    }

    #[wasm_bindgen]
    pub fn cancel_component_edit(&mut self) -> bool {
        self.editor.cancel_component_edit()
    }

    #[wasm_bindgen]
    pub fn rename_component(&mut self, stem: &str, key: &str, name: &str) -> bool {
        self.editor.rename_component(stem, key, name)
    }

    #[wasm_bindgen]
    pub fn rename_component_key(&mut self, stem: &str, key: &str, new_key: &str) -> bool {
        self.editor.rename_component_key(stem, key, new_key)
    }

    #[wasm_bindgen]
    pub fn delete_component(&mut self, stem: &str, key: &str) -> bool {
        self.editor.delete_component(stem, key)
    }

    #[wasm_bindgen]
    pub fn move_component(&mut self, stem: &str, key: &str, dest_stem: &str) -> String {
        self.editor
            .move_component(stem, key, dest_stem)
            .unwrap_or_default()
    }

    #[wasm_bindgen]
    pub fn load_user_libraries(&mut self, json: &str) {
        let blobs: Vec<UserLibBlob> = serde_json::from_str(json).unwrap_or_default();
        let mut libs = Vec::new();
        for b in blobs {
            if b.fcl.trim().is_empty() {
                if !b.stem.is_empty() {
                    let mut lib = fidorust_core::Library::empty_user(
                        b.stem,
                        if b.title.is_empty() {
                            "Library".into()
                        } else {
                            b.title
                        },
                    );
                    lib.aliases = b.aliases;
                    libs.push(lib);
                }
                continue;
            }
            if let Ok(mut lib) = parse_library(&b.fcl) {
                if !b.stem.is_empty() {
                    lib.file_stem = b.stem;
                }
                if !b.title.is_empty() {
                    lib.name = b.title;
                }
                lib.aliases = b.aliases;
                lib.kind = LibraryKind::Local;
                lib.standard = false;
                libs.push(lib);
            }
        }
        self.editor.load_user_libraries(libs);
    }

    #[wasm_bindgen]
    pub fn user_libraries_blob(&self) -> String {
        let blobs: Vec<UserLibBlob> = self
            .editor
            .libs()
            .user_libraries()
            .map(|lib| UserLibBlob {
                stem: lib.file_stem.clone(),
                title: lib.name.clone(),
                fcl: serialize_library(lib),
                aliases: lib.aliases.clone(),
            })
            .collect();
        to_json(&blobs, "[]")
    }

    #[wasm_bindgen]
    pub fn create_user_library(&mut self, title: &str) -> String {
        self.editor.create_user_library(title)
    }

    #[wasm_bindgen]
    pub fn import_library(&mut self, text: &str, filename: &str) -> Result<String, JsValue> {
        let mut lib = parse_library(text).map_err(to_js)?;
        lib.add_filename_alias(filename);
        Ok(self.editor.import_library(lib))
    }

    #[wasm_bindgen]
    pub fn rename_library(&mut self, stem: &str, title: &str) -> String {
        self.editor.rename_library(stem, title).unwrap_or_default()
    }

    #[wasm_bindgen]
    pub fn export_library_fcl(&self, stem: &str) -> String {
        self.editor
            .libs()
            .library(stem)
            .map(serialize_library)
            .unwrap_or_default()
    }

    #[wasm_bindgen]
    pub fn clear_project_library(&mut self) -> bool {
        self.editor.clear_project_library()
    }

    #[wasm_bindgen]
    pub fn remove_user_library(&mut self, stem: &str) -> bool {
        self.editor.remove_user_library(stem)
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

fn patch_font_face_choices(fields: &mut [PropFormField]) {
    let mut options = fidorust_gpu::font::registered_families();
    for f in fields.iter_mut() {
        if f.id != PropField::FontFace {
            continue;
        }
        if let PropFieldValue::String { value } = &f.value {
            if !value.is_empty() && !options.iter().any(|o| o.eq_ignore_ascii_case(value)) {
                options.push(value.clone());
            }
        }
        f.kind = PropFieldKind::Choice {
            options: options.clone(),
        };
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
