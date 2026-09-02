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
fn tessellate_rounded_pcb_pad_uses_fills() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidocad_core::Primitive::PcbPad {
        pos: fidocad_core::Point::new(380, 65),
        dx: 18,
        dy: 18,
        hole: 4,
        style: fidocad_core::primitive::PadStyle::RoundedRect,
        layer: fidocad_core::LayerId(0),
    });
    let mut ed = Editor::new(builtin_libraries());
    ed.doc = doc;
    let scene = tessellate_editor(&ed);
    assert!(scene.fills.len() >= 6);
    assert_eq!(scene.circles.len(), 0);
    assert_eq!(scene.pad_holes.len(), 1);
}

#[test]
fn tessellate_oval_pcb_pad_has_circular_hole() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidocad_core::Primitive::PcbPad {
        pos: fidocad_core::Point::new(260, 125),
        dx: 40,
        dy: 30,
        hole: 25,
        style: fidocad_core::primitive::PadStyle::Oval,
        layer: fidocad_core::LayerId(0),
    });
    let mut ed = Editor::new(builtin_libraries());
    ed.doc = doc;
    let scene = tessellate_editor(&ed);
    assert!(scene.fills.len() >= 6);
    assert_eq!(scene.circles.len(), 0);
    assert_eq!(scene.pad_holes.len(), 1);
    assert!((scene.pad_holes[0].r - 12.5).abs() < 0.01);
}

#[test]
fn tessellate_pcb_track_is_filled_capsule() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidocad_core::Primitive::PcbTrack {
        a: fidocad_core::Point::new(80, 140),
        b: fidocad_core::Point::new(140, 140),
        width: 16,
        layer: fidocad_core::LayerId(0),
    });
    let mut ed = Editor::new(builtin_libraries());
    ed.doc = doc;
    let scene = tessellate_editor(&ed);
    assert!(
        scene.fills.len() >= 3,
        "expected filled capsule, got {} fills",
        scene.fills.len()
    );
    assert!(
        scene.lines.is_empty(),
        "pcb track must not use the line shader"
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
    assert_eq!(
        scene.circles.len(),
        1,
        "expected ellipse instance, not a segment"
    );
    assert!(scene.circles[0].rx > scene.circles[0].ry);
    assert!(scene.circles[0].stroke > 0.0);
    assert!(
        scene.lines.is_empty(),
        "ellipse draft must not fall back to a line"
    );
}

#[test]
fn macro_thumb_svg_has_geometry() {
    let libs = builtin_libraries();
    let (_, def) = libs.lookup("080").expect("resistor");
    let prims = fidocad_core::library::expand_macro(
        def,
        fidocad_core::geom::Transform {
            origin: fidocad_core::MACRO_ORIGIN,
            rotations: 0,
            mirrored: false,
        },
        &libs,
        0,
    );
    let scene =
        fidocad_gpu::tessellate_primitives(&prims, &fidocad_core::LayerSet::default(), false);
    let svg = fidocad_gpu::scene_to_thumb_svg(&scene, 40.0);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<line") || svg.contains("<ellipse") || svg.contains("<polygon"));
}

#[test]
fn macro_cursor_svg_has_hotspot() {
    let libs = builtin_libraries();
    let (_, def) = libs.lookup("080").expect("resistor");
    let prims = fidocad_gpu::tessellate_primitives(
        &fidocad_core::library::expand_macro(
            def,
            fidocad_core::geom::Transform {
                origin: fidocad_core::MACRO_ORIGIN,
                rotations: 0,
                mirrored: false,
            },
            &libs,
            0,
        ),
        &fidocad_core::LayerSet::default(),
        false,
    );
    let cur = fidocad_gpu::scene_to_cursor_svg(&prims, fidocad_core::MACRO_ORIGIN);
    assert!(cur.w > 1.0 && cur.h > 1.0);
    assert!(cur.svg.contains("<line") || cur.svg.contains("<ellipse") || cur.svg.contains("<polygon"));
}

#[test]
fn pending_macro_ghost_appears_at_hover() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Macro;
    ed.pending_macro = Some("080".into());
    ed.hover = Some(fidocad_core::Point::new(40, 40));
    let scene = tessellate_editor(&ed);
    assert!(
        !scene.lines.is_empty() || !scene.circles.is_empty() || !scene.fills.is_empty(),
        "expected ghost geometry for pending macro"
    );
}

#[test]
fn tessellate_text_uses_filled_glyphs() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidocad_core::Primitive::Text {
        pos: fidocad_core::Point::new(0, 0),
        sy: 10,
        sx: 6,
        angle: 0,
        style: 0,
        layer: fidocad_core::LayerId(0),
        font: "Courier New".into(),
        text: "Vcc".into(),
        simple: false,
    });
    let mut ed = Editor::new(builtin_libraries());
    ed.doc = doc;
    let scene = tessellate_editor(&ed);
    assert!(
        scene.fills.len() >= 27,
        "expected filled glyph triangles, got {}",
        scene.fills.len()
    );
    assert_eq!(scene.fills.len() % 3, 0);
}

#[test]
fn editing_text_hides_glyphs() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidocad_core::Primitive::Text {
        pos: Point::new(0, 0),
        sy: 10,
        sx: 6,
        angle: 0,
        style: 0,
        layer: fidocad_core::LayerId(0),
        font: "Courier New".into(),
        text: "Vcc".into(),
        simple: false,
    });
    let mut ed = Editor::new(builtin_libraries());
    ed.doc = doc;
    let before = tessellate_editor(&ed).fills.len();
    assert!(before > 0);
    ed.editing_text = Some(0);
    assert!(tessellate_editor(&ed).fills.is_empty());
}
