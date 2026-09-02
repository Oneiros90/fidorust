use fidocad_core::parse::{builtin_libraries, parse_document, parse_primitive_line};
use fidocad_core::serialize::{serialize_document, serialize_primitive};
use fidocad_core::{Document, Editor, LayerId, Point, Primitive, SaveOptions, Tool};

const WEBSITE_SAMPLE: &str = r#"[FIDOCAD]
MC 65 35 0 0 410
MC 65 55 0 0 420
LI 65 35 65 55
LI 80 45 95 45
LI 65 45 50 45
MC 80 65 0 0 040
SA 80 45
LI 95 43 95 47
LI 95 47 102 47
LI 95 43 102 43
LI 102 43 105 45
LI 105 45 102 47
LI 40 43 40 47
LI 40 47 47 47
LI 40 43 47 43
LI 47 43 50 45
LI 50 45 47 47
SA 65 45
TE 40 35 IN
TE 95 35 OUT
MC 80 25 3 0 010
"#;

#[test]
fn parse_website_sample() {
    let doc = parse_document(WEBSITE_SAMPLE).unwrap();
    assert!(doc.primitives.len() >= 20);
    let macros = doc
        .primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Macro { .. }))
        .count();
    assert_eq!(macros, 4);
    let texts = doc
        .primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Text { .. }))
        .count();
    assert_eq!(texts, 2);
}

#[test]
fn skip_fcj_and_splines() {
    let src = "[FIDOCAD]\nLI 0 0 10 10\nFCJ 1 0 8 4 0 0\nCV 0 0 0 10 10 0\nFJC A 0.5\nSA 5 5\n";
    let doc = parse_document(src).unwrap();
    assert_eq!(doc.primitives.len(), 2);
}

#[test]
fn layer_omitted_means_zero() {
    let p = parse_primitive_line("LI 1 2 3 4").unwrap();
    match p {
        Primitive::Line { layer, .. } => assert_eq!(layer.0, 0),
        _ => panic!("expected line"),
    }
    let p = parse_primitive_line("LI 1 2 3 4 7").unwrap();
    match p {
        Primitive::Line { layer, .. } => assert_eq!(layer.0, 7),
        _ => panic!(),
    }
}

#[test]
fn roundtrip_line_layer() {
    let p = parse_primitive_line("LI 10 20 30 40 2").unwrap();
    let s = serialize_primitive(&p);
    assert_eq!(s, "LI 10 20 30 40 2\r\n");
    let p0 = parse_primitive_line("LI 10 20 30 40").unwrap();
    assert_eq!(serialize_primitive(&p0), "LI 10 20 30 40\r\n");
}

#[test]
fn alimentatore_golden() {
    let src = include_str!("Alimentatore.fcd");
    let doc = parse_document(src).unwrap();
    assert!(doc.title.contains("Alimentatore"));
    assert!(doc.primitives.len() > 50);
    let out = serialize_document(&doc, SaveOptions::default(), None);
    let doc2 = parse_document(&out).unwrap();
    assert_eq!(doc.primitives.len(), doc2.primitives.len());
}

#[test]
fn builtin_stdlib_has_resistor() {
    let libs = builtin_libraries();
    let found = libs.lookup("080").expect("resistor 080");
    assert!(found.1.name.to_lowercase().contains("resist"));
    assert!(!found.1.primitives.is_empty());
}

#[test]
fn macro_transform_matches_fidocad() {
    use fidocad_core::geom::{Point, Transform};
    use fidocad_core::MACRO_ORIGIN;

    let xf = |origin, rotations, mirrored| Transform {
        origin,
        rotations,
        mirrored,
    };

    // FidoCadJ MapCoordinates: local (110, 100) at MC 105 30 0 0 → (115, 30)
    assert_eq!(
        xf(Point::new(105, 30), 0, false).apply(Point::new(110, 100), MACRO_ORIGIN),
        Point::new(115, 30)
    );

    // MC 105 30 1 0 210 — diode tip at macro (110, 90)
    assert_eq!(
        xf(Point::new(105, 30), 1, false).apply(Point::new(110, 90), MACRO_ORIGIN),
        Point::new(115, 40)
    );

    // MC 115 80 3 0 210
    assert_eq!(
        xf(Point::new(115, 80), 3, false).apply(Point::new(110, 90), MACRO_ORIGIN),
        Point::new(105, 70)
    );

    // Mirrored macro 300 at (195, 100) 0 1
    assert_eq!(
        xf(Point::new(195, 100), 0, true).apply(Point::new(110, 100), MACRO_ORIGIN),
        Point::new(185, 100)
    );
}

#[test]
fn expand_terminal() {
    let libs = builtin_libraries();
    let doc = parse_document("[FIDOCAD]\nMC 10 10 0 0 000\n").unwrap();
    let flat = fidocad_core::library::flatten(&doc.primitives, &libs);
    assert!(flat.len() >= 2);
}

#[test]
fn pcb_pad_and_track() {
    let p = parse_primitive_line("PA 100 100 18 18 8 0 1").unwrap();
    match p {
        Primitive::PcbPad { hole, layer, .. } => {
            assert_eq!(hole, 8);
            assert_eq!(layer.0, 1);
        }
        _ => panic!(),
    }
    let t = parse_primitive_line("PL 0 0 50 0 10 1").unwrap();
    match t {
        Primitive::PcbTrack { width, layer, .. } => {
            assert_eq!(width, 10);
            assert_eq!(layer.0, 1);
        }
        _ => panic!(),
    }
}

#[test]
fn empty_document_ok() {
    let d = Document::default();
    let s = serialize_document(&d, SaveOptions::default(), None);
    assert!(s.starts_with("[FIDOCAD]"));
}

#[test]
fn reject_zero_length_line() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Line;
    ed.doc.snap = 1;
    let p = Point::new(10, 10);
    ed.pointer_down(p, (0.0, 0.0), false, false);
    ed.pointer_move(p, (0.0, 0.0));
    ed.pointer_up(p);
    assert!(
        ed.doc.primitives.is_empty(),
        "original FidoCAD ignores a second point that coincides with the first"
    );
}

#[test]
fn accept_nonzero_line() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Line;
    ed.doc.snap = 1;
    ed.pointer_down(Point::new(10, 10), (0.0, 0.0), false, false);
    ed.pointer_move(Point::new(30, 10), (20.0, 0.0));
    ed.pointer_up(Point::new(30, 10));
    assert_eq!(ed.doc.primitives.len(), 1);
}

#[test]
fn marquee_rect_while_dragging() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Select;
    ed.pointer_down(Point::new(0, 0), (0.0, 0.0), false, false);
    ed.pointer_move(Point::new(40, 25), (40.0, 25.0));
    let (a, b) = ed
        .marquee_rect()
        .expect("marquee should be live while dragging");
    assert_eq!(a, Point::new(0, 0));
    assert_eq!(b, Point::new(40, 25));
    ed.pointer_up(Point::new(40, 25));
    assert!(ed.marquee_rect().is_none());
}

#[test]
fn right_click_rotates_pending_macro() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Macro;
    ed.pending_macro = Some("080".into());
    assert!(ed.right_click(Point::new(20, 20)));
    assert_eq!(ed.pending_rotations, 1);
    assert!(ed.right_click(Point::new(20, 20)));
    assert_eq!(ed.pending_rotations, 2);
}

#[test]
fn right_click_rotates_while_moving_selection() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Select;
    ed.doc.snap = 1;
    ed.doc.insert(Primitive::Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    });
    ed.selected.push(0);
    ed.pointer_down(Point::new(5, 0), (5.0, 0.0), false, false);
    ed.pointer_move(Point::new(8, 0), (8.0, 0.0));
    assert!(ed.right_click(Point::new(8, 0)));
    match &ed.doc.primitives[0] {
        Primitive::Line { a, b, .. } => {
            assert_ne!((*a, *b), (Point::new(3, 0), Point::new(13, 0)));
        }
        _ => panic!("expected line"),
    }
}

#[test]
fn invert_selection_toggles_indices() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc.insert(Primitive::Connection {
        pos: Point::new(0, 0),
        layer: LayerId(0),
    });
    ed.doc.insert(Primitive::Connection {
        pos: Point::new(10, 0),
        layer: LayerId(0),
    });
    ed.selected = vec![0];
    ed.invert_selection();
    assert_eq!(ed.selected, vec![1]);
}

fn sample_text(pos: Point, text: &str) -> Primitive {
    Primitive::Text {
        pos,
        sy: 4,
        sx: 3,
        angle: 0,
        style: 0,
        layer: LayerId(0),
        font: "Courier New".into(),
        text: text.into(),
        simple: false,
    }
}

#[test]
fn text_hit_matches_glyph_box() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc.snap = 1;
    ed.doc.insert(sample_text(Point::new(10, 20), "AB"));
    // "AB" is 2×3 LU wide and 4 LU tall, origin top-left at (10, 20).
    assert!(ed.begin_text_edit_at(Point::new(11, 21)).is_some());
    ed.cancel_text_edit();
    assert!(ed.begin_text_edit_at(Point::new(13, 14)).is_none());
    assert!(ed.begin_text_edit_at(Point::new(22, 22)).is_none());
}

#[test]
fn text_edit_commit_replaces_content() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc.snap = 1;
    ed.doc.insert(sample_text(Point::new(0, 0), "IN"));
    let session = ed.begin_text_edit_at(Point::new(1, 1)).expect("hit text");
    assert_eq!(session.text, "IN");
    assert_eq!(ed.editing_text, Some(0));
    ed.commit_text_edit("OUT".into());
    assert!(ed.editing_text.is_none());
    match &ed.doc.primitives[0] {
        Primitive::Text { text, .. } => assert_eq!(text, "OUT"),
        _ => panic!("expected text"),
    }
}

#[test]
fn dblclick_finishes_poly_instead_of_text_edit() {
    let mut ed = Editor::new(builtin_libraries());
    ed.tool = Tool::Poly;
    ed.doc.snap = 1;
    ed.pointer_down(Point::new(0, 0), (0.0, 0.0), false, false);
    ed.pointer_up(Point::new(0, 0));
    ed.pointer_down(Point::new(10, 0), (10.0, 0.0), false, false);
    ed.pointer_up(Point::new(10, 0));
    assert!(ed.begin_text_edit_at(Point::new(5, 0)).is_none());
    assert!(ed
        .doc
        .primitives
        .iter()
        .any(|p| matches!(p, Primitive::Poly { .. })));
}

#[test]
fn snap_xy_independent_and_disable() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc.snap = 10;
    ed.doc.snap_y = 5;
    assert_eq!(ed.snap_pt(Point::new(14, 8)), Point::new(10, 10));
    ed.snap_enable = false;
    assert_eq!(ed.snap_pt(Point::new(14, 8)), Point::new(14, 8));
}
