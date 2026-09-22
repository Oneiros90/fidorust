//! Map `Editor` into GPU tessellation input (gpu crate does not import Editor).

use fidorust_core::Editor;
use fidorust_gpu::{tessellate, DraftParams, Scene, TessellateInput};

pub fn tessellate_input<'a>(ed: &'a Editor, viewport: Option<(f32, f32)>) -> TessellateInput<'a> {
    tessellate_input_pane(ed, ed.active_pane(), viewport)
}

pub fn tessellate_input_pane<'a>(
    ed: &'a Editor,
    pane: usize,
    viewport: Option<(f32, f32)>,
) -> TessellateInput<'a> {
    let sheet = ed.pane_sheet(pane);
    let active = pane == ed.active_pane();
    TessellateInput {
        primitives: &sheet.primitives,
        layers: &ed.doc().layers,
        libs: ed.libs(),
        selected: ed.pane_selected(pane),
        hover: ed.pane_hover_index(pane),
        editing_text: if active { ed.editing_text() } else { None },
        hide_component_origin: ed.hide_component_origin(),
        stroke_w: ed.doc().stroke_width(),
        zoom: ed.pane_zoom(pane),
        pan: ed.pane_pan(pane),
        layer: ed.layer(),
        viewport,
        theme: ed.canvas_theme(),
        pending: if active {
            let mut pending = ed.pending_component_preview();
            pending.extend(ed.duplicate_drag_preview());
            pending
        } else {
            Vec::new()
        },
        marquee: if active {
            ed.marquee_screen_rect()
        } else {
            None
        },
        tool: ed.tool(),
        ruler_segments: if active { ed.ruler_segments() } else { &[] },
        view: ed.view_overlay_for_pane(pane),
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

pub fn tessellate_view_pane(ed: &Editor, pane: usize, viewport: Option<(f32, f32)>) -> Scene {
    let draft = if pane == ed.active_pane() {
        draft_params(ed)
    } else {
        DraftParams {
            points: &[],
            tool: None,
            filled: false,
            hover: None,
        }
    };
    tessellate(tessellate_input_pane(ed, pane, viewport), &draft)
}

#[allow(dead_code)]
pub fn tessellate_editor(ed: &Editor) -> Scene {
    tessellate_view(ed, None)
}
