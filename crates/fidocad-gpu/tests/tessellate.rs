use fidocad_core::parse::{builtin_libraries, parse_document};
use fidocad_core::{Editor, Point, Tool};
use fidocad_gpu::tessellate_editor;

#[test]
fn tessellate_alimentatore_has_strokes() {
    let libs = builtin_libraries();
    let mut ed = Editor::new(libs);
    ed.load_text(include_str!("../../fidocad-core/tests/Alimentatore.fcd"))
        .unwrap();
    let scene = tessellate_editor(&ed);
    assert!(
        scene.lines.len() + scene.fills.len() + scene.circles.len() > 50,
        "lines={} fills={} circles={}",
        scene.lines.len(),
        scene.fills.len(),
        scene.circles.len()
    );
}

#[test]
fn tessellate_heavy_grid() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    for x in (0..400).step_by(4) {
        doc.primitives.push(fidocad_core::Primitive::Line {
            a: fidocad_core::Point::new(x, 0),
            b: fidocad_core::Point::new(x, 400),
            layer: fidocad_core::LayerId(0),
        });
    }
    let mut ed = Editor::new(builtin_libraries());
    ed.doc = doc;
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.lines.len(), 100);
}

#[test]
fn tessellate_ellipse_is_stroked_not_annulus() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidocad_core::Primitive::Ellipse {
        a: fidocad_core::Point::new(0, 0),
        b: fidocad_core::Point::new(40, 20),
        filled: false,
        layer: fidocad_core::LayerId(0),
    });
    let mut ed = Editor::new(builtin_libraries());
    ed.doc = doc;
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.circles.len(), 1);
    assert!(scene.circles[0].stroke > 0.0);
    assert_eq!(scene.circles[0].inner, 0.0);
}

#[test]
fn draft_ellipse_previews_as_ellipse_not_line() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Ellipse;
    ed.doc.snap = 1;
    ed.pointer_down(Point::new(0, 0), (0.0, 0.0), false, false);
    ed.pointer_move(Point::new(80, 10), (80.0, 10.0));
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.circles.len(), 1, "expected ellipse instance, not a segment");
    assert!(scene.circles[0].rx > scene.circles[0].ry);
    assert!(scene.circles[0].stroke > 0.0);
    assert!(
        scene.lines.is_empty(),
        "ellipse draft must not fall back to a line"
    );
}
