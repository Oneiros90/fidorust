//! Render backend: WebGL on wasm32, no-op elsewhere.

use fidocad_core::Editor;
use fidocad_gpu::tessellate::tessellate_view;
use fidocad_gpu::{Scene, Theme};
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
use fidocad_gpu::renderer::Renderer;

pub struct Backend {
    #[cfg(target_arch = "wasm32")]
    renderer: Option<Renderer>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            renderer: None,
        }
    }

    pub fn attach_canvas(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        #[cfg(target_arch = "wasm32")]
        {
            self.renderer = Some(Renderer::from_canvas(&canvas).map_err(JsValue::from)?);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = canvas;
        }
        Ok(())
    }

    pub fn apply_theme(&mut self, editor: &mut Editor, theme: &str) {
        let palette = if theme == "dark" {
            Theme::DARK
        } else {
            Theme::LIGHT
        };
        editor.set_canvas_dark(theme == "dark");
        #[cfg(target_arch = "wasm32")]
        if let Some(r) = self.renderer.as_mut() {
            r.set_theme_enum(&palette);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = palette;
        }
    }

    pub fn draw(&mut self, editor: &Editor, size: (f32, f32), show_grid: bool) {
        let scene = tessellate_view(editor, Some(size));
        self.draw_scene(editor, &scene, size, show_grid);
    }

    fn draw_scene(&mut self, editor: &Editor, scene: &Scene, size: (f32, f32), show_grid: bool) {
        #[cfg(target_arch = "wasm32")]
        if let Some(r) = self.renderer.as_mut() {
            r.draw(
                scene,
                editor.pan(),
                editor.zoom(),
                size,
                (editor.doc().grid as f32, editor.doc().grid_y as f32),
                show_grid,
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (editor, scene, size, show_grid);
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
