//! File-export helpers used by the WASM façade.

use fidorust_gpu::tessellate::export_svg as export_svg_gpu;

use crate::json::ExportSvgOpts;
use crate::App;

/// `opts_json` is `{ margin_lu, bw, layers: [{ show, color }] }`. Empty / invalid JSON uses document layers.
pub(crate) fn export_svg(app: &App, opts_json: &str) -> String {
    let opts: ExportSvgOpts = serde_json::from_str(opts_json).unwrap_or_default();
    let mut layers = app.editor.doc().layers.clone();
    layers.apply_export_overlay(opts.bw, &opts.overlays());
    export_svg_gpu(
        &app.editor.doc().primitives,
        &layers,
        app.editor.libs(),
        opts.margin_lu.max(0.0),
        app.editor.doc().stroke_width(),
    )
}
