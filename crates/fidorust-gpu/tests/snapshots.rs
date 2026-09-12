mod common;

use fidorust_core::parse::builtin_libraries;
use fidorust_core::{Editor, Point, Text, DEFAULT_FONT};
use fidorust_gpu::{
    export_svg, scene_to_cursor_svg, scene_to_svg, scene_to_thumb_svg, tessellate_primitives, Scene,
};

use common::tessellate_view;

fn scene_digest(scene: &Scene) -> String {
    format!(
        "lines={} fills={} circles={} handles={} holes={} marquee={:?}\nfill_end={:?}\nline_end={:?}\ncirc_end={:?}\nhole_end={:?}\n",
        scene.lines.len(),
        scene.fills.len(),
        scene.circles.len(),
        scene.handles.len(),
        scene.pad_holes.len(),
        scene.marquee,
        scene.layer_fill_end,
        scene.layer_line_end,
        scene.layer_circ_end,
        scene.layer_hole_end,
    )
}

#[test]
fn snapshot_tessellate_alimentatore() {
    let mut ed = Editor::new(builtin_libraries());
    ed.load_text(include_str!("../../fidorust-core/tests/Alimentatore.fcd"))
        .unwrap();
    ed.set_view(4.0, (40.0, 40.0));
    let scene = tessellate_view(&ed, Some((800.0, 600.0)));
    common::assert_snapshot("alimentatore_view.txt", &scene_digest(&scene));
    let svg = scene_to_svg(&scene, 800.0, 600.0, ed.zoom(), ed.pan());
    common::assert_snapshot("alimentatore_export.svg", &svg);
    let export = export_svg(
        &ed.doc().primitives,
        &ed.doc().layers,
        ed.libs(),
        8.0,
        ed.doc().stroke_width(),
    );
    common::assert_snapshot("alimentatore_full_export.svg", &export);
}

#[test]
fn snapshot_macro_svgs() {
    let libs = builtin_libraries();
    let (_, def) = libs.lookup("080").expect("resistor");
    let prims = fidorust_core::library::expand_component(
        def,
        fidorust_core::geom::Transform {
            origin: fidorust_core::COMPONENT_ORIGIN,
            rotations: 0,
            mirrored: false,
        },
        &libs,
        0,
    );
    let scene = tessellate_primitives(&prims, &fidorust_core::LayerSet::default());
    common::assert_snapshot("macro_080_thumb.svg", &scene_to_thumb_svg(&scene, 40.0));
    let cur = scene_to_cursor_svg(&scene, fidorust_core::COMPONENT_ORIGIN);
    common::assert_snapshot(
        "macro_080_cursor.txt",
        &format!(
            "ox={} oy={} w={} h={}\n{}\n",
            cur.ox, cur.oy, cur.w, cur.h, cur.svg
        ),
    );
}

#[test]
fn snapshot_draft_and_text() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(fidorust_core::Primitive::Text(Text {
        pos: Point::new(0, 0),
        sy: 10,
        sx: 6,
        angle: 0,
        style: 0,
        layer: fidorust_core::LayerId(0),
        font: DEFAULT_FONT.into(),
        text: "Vcc".into(),
        simple: false,
    }));
    let scene = tessellate_view(&ed, None);
    common::assert_snapshot("text_vcc.txt", &scene_digest(&scene));
}
