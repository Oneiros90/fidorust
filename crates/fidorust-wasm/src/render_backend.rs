//! Render backend: WebGL on wasm32, no-op elsewhere.

use fidorust_core::{CanvasTheme, Editor};
use fidorust_gpu::Theme;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
use fidorust_gpu::renderer::Renderer;

pub struct Backend {
    #[cfg(target_arch = "wasm32")]
    renderers: [Option<Renderer>; 2],
}

impl Backend {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            renderers: [None, None],
        }
    }

    #[allow(dead_code)]
    pub fn attach_canvas(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        self.attach_pane_canvas(0, canvas)
    }

    pub fn attach_pane_canvas(
        &mut self,
        pane: usize,
        canvas: HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        let pane = pane.min(1);
        #[cfg(target_arch = "wasm32")]
        {
            self.renderers[pane] = Some(Renderer::from_canvas(&canvas).map_err(JsValue::from)?);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = canvas;
            let _ = pane;
        }
        Ok(())
    }

    pub fn detach_pane(&mut self, pane: usize) {
        #[cfg(target_arch = "wasm32")]
        {
            self.renderers[pane.min(1)] = None;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = pane;
        }
    }

    pub fn apply_theme(&mut self, editor: &mut Editor, theme: &str) {
        let canvas = CanvasTheme::parse(theme);
        editor.set_canvas_theme(canvas);
        self.apply_effective_theme(editor);
    }

    fn apply_effective_theme(&mut self, editor: &Editor) {
        let palette = [0, 1]
            .iter()
            .find_map(|&pane| editor.view_overlay_for_pane(pane).and_then(|v| v.canvas))
            .map(|p| Theme {
                bg: p.bg,
                grid: p.grid,
                selection: p.selection,
            })
            .unwrap_or_else(|| Theme::from_canvas(editor.canvas_theme()));
        #[cfg(target_arch = "wasm32")]
        for r in self.renderers.iter_mut().flatten() {
            r.set_theme_enum(&palette);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = palette;
        }
    }

    #[allow(dead_code)]
    pub fn draw(&mut self, editor: &Editor, size: (f32, f32), show_grid: bool) {
        self.draw_pane(0, editor, size, show_grid);
    }

    pub fn draw_pane(&mut self, pane: usize, editor: &Editor, size: (f32, f32), show_grid: bool) {
        self.apply_effective_theme(editor);
        let pane = pane.min(1);
        let scene = crate::tessellate::tessellate_view_pane(editor, pane, Some(size));
        let sheet = editor.pane_sheet(pane);
        #[cfg(target_arch = "wasm32")]
        if let Some(r) = self.renderers[pane].as_mut() {
            r.draw(
                &scene,
                editor.pane_pan(pane),
                editor.pane_zoom(pane),
                size,
                (sheet.grid as f32, sheet.grid_y as f32),
                show_grid && sheet.show_grid,
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (scene, size, show_grid, sheet);
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
