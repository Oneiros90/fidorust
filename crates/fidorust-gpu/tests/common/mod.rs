#![allow(dead_code)]

use std::path::PathBuf;

use fidorust_core::Editor;
use fidorust_gpu::{tessellate, DraftParams, Scene, TessellateInput};

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

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

pub fn tessellate_editor(ed: &Editor) -> Scene {
    tessellate_view(ed, None)
}

pub fn tessellate_export(ed: &Editor, layers: &fidorust_core::LayerSet) -> Scene {
    fidorust_gpu::tessellate_export(
        &ed.doc().primitives,
        layers,
        ed.libs(),
        ed.doc().stroke_width(),
    )
}

pub fn assert_snapshot(name: &str, actual: &str) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
    std::fs::create_dir_all(&dir).expect("create snapshots dir");
    let path = dir.join(name);
    if std::env::var("UPDATE_SNAPSHOTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, normalize_newlines(actual))
            .unwrap_or_else(|e| panic!("write {name}: {e}"));
        return;
    }
    let expected =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing snapshot {name}: {e}"));
    assert_eq!(
        normalize_newlines(&expected),
        normalize_newlines(actual),
        "snapshot mismatch: {name}"
    );
}
