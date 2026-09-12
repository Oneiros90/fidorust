//! Map `Editor` into GPU tessellation input (gpu crate does not import Editor).

use fidorust_core::Editor;
use fidorust_gpu::{tessellate, DraftParams, Scene, TessellateInput};

pub fn tessellate_input<'a>(ed: &'a Editor, viewport: Option<(f32, f32)>) -> TessellateInput<'a> {
    TessellateInput {
        primitives: &ed.doc().primitives,
        layers: &ed.doc().layers,
        libs: ed.libs(),
        selected: ed.selected(),
        hover: ed.hover_index(),
        editing_text: ed.editing_text(),
        hide_component_origin: ed.hide_component_origin(),
        stroke_w: ed.doc().stroke_width(),
        zoom: ed.zoom(),
        pan: ed.pan(),
        layer: ed.layer(),
        viewport,
        theme: ed.canvas_theme(),
        pending: {
            let mut pending = ed.pending_component_preview();
            pending.extend(ed.duplicate_drag_preview());
            pending
        },
        marquee: ed.marquee_screen_rect(),
        tool: ed.tool(),
        ruler_segments: ed.ruler_segments(),
    }
}

pub fn draft_params(ed: &Editor) -> DraftParams<'_> {
    DraftParams {
        points: ed.draft_points(),
        tool: ed.draft_tool(),
        filled: ed.filled(),
        hover: ed.hover(),
    }
}

pub fn tessellate_view(ed: &Editor, viewport: Option<(f32, f32)>) -> Scene {
    tessellate(tessellate_input(ed, viewport), &draft_params(ed))
}

#[allow(dead_code)]
pub fn tessellate_editor(ed: &Editor) -> Scene {
    tessellate_view(ed, None)
}
