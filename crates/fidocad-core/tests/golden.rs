use fidocad_core::parse::{builtin_libraries, parse_document, parse_primitive_line};
use fidocad_core::primitive::STYLE_MIRRORED;
use fidocad_core::serialize::{serialize_document, serialize_primitive};
use fidocad_core::{
    ComponentRef, Connection, DblClickAction, Document, Editor, LayerId, Line, PcbPad, PcbTrack,
    Point, Poly, Primitive, PropPatch, Rect, Text, Tool, Transform, COMPONENT_ORIGIN, DEFAULT_FONT,
};

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
        .filter(|p| matches!(p, Primitive::Component(ComponentRef { .. })))
        .count();
    assert_eq!(macros, 4);
    let texts = doc
        .primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Text(Text { .. })))
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
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 0),
        _ => panic!("expected line"),
    }
    let p = parse_primitive_line("LI 1 2 3 4 7").unwrap();
    match p {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 7),
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
    let out = serialize_document(&doc, None);
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
    use fidocad_core::COMPONENT_ORIGIN;

    let xf = |origin, rotations, mirrored| Transform {
        origin,
        rotations,
        mirrored,
    };

    // FidoCadJ MapCoordinates: local (110, 100) at MC 105 30 0 0 → (115, 30)
    assert_eq!(
        xf(Point::new(105, 30), 0, false).apply(Point::new(110, 100), COMPONENT_ORIGIN),
        Point::new(115, 30)
    );

    // MC 105 30 1 0 210 — diode tip at macro (110, 90)
    assert_eq!(
        xf(Point::new(105, 30), 1, false).apply(Point::new(110, 90), COMPONENT_ORIGIN),
        Point::new(115, 40)
    );

    // MC 115 80 3 0 210
    assert_eq!(
        xf(Point::new(115, 80), 3, false).apply(Point::new(110, 90), COMPONENT_ORIGIN),
        Point::new(105, 70)
    );

    // Mirrored macro 300 at (195, 100) 0 1
    assert_eq!(
        xf(Point::new(195, 100), 0, true).apply(Point::new(110, 100), COMPONENT_ORIGIN),
        Point::new(185, 100)
    );
}

#[test]
fn expand_terminal() {
    let libs = builtin_libraries();
    let doc = parse_document("[FIDOCAD]\nMC 10 10 0 0 000\n").unwrap();
    let flat = fidocad_core::library::expand_primitive(&doc.primitives[0], &libs);
    assert!(flat.len() >= 2);
}

#[test]
fn mc_optional_layer_token() {
    let doc = parse_document("[FIDOCAD]\nMC 10 20 0 0 080 2\n").unwrap();
    match &doc.primitives[0] {
        Primitive::Component(c) => {
            assert_eq!(c.name, "080");
            assert_eq!(c.layer.0, 2);
        }
        _ => panic!(),
    }
    let out = serialize_document(&doc, None);
    assert!(out.contains("MC 10 20 0 0 080 2"), "{out}");
}

#[test]
fn mirrored_text_aabb_extends_left_of_origin() {
    let p = parse_primitive_line("TY 1100 265 30 15 0 5 1 * Propic2 compatible PCB").unwrap();
    let bb = p.aabb();
    // style 5 includes mirrored (bit 4): glyphs sit left of pos, not right.
    assert!(bb.max.x <= 1100, "max.x={} should be ≤ origin", bb.max.x);
    assert!(
        bb.min.x < 1100 - 100,
        "min.x={} should cover the string leftward",
        bb.min.x
    );
    assert!(bb.min.y <= 265);
    assert!(bb.max.y >= 265 + 30);
}

#[test]
fn axis_aligned_text_aabb_extends_right_of_origin() {
    let p = parse_primitive_line("TY 100 200 20 10 0 0 0 * Hello").unwrap();
    let bb = p.aabb();
    assert_eq!(bb.min, Point::new(100, 200));
    assert_eq!(bb.max, Point::new(100 + 10 * 5, 200 + 20));
}

#[test]
fn star_token_is_courier_prime_and_roundtrips() {
    let p = parse_primitive_line("TY 0 0 4 3 0 0 0 * Hello").unwrap();
    match &p {
        Primitive::Text(t) => {
            assert_eq!(t.font, DEFAULT_FONT);
            assert_eq!(t.font, "Courier Prime");
        }
        other => panic!("expected text, got {other:?}"),
    }
    let out = serialize_primitive(&p);
    assert!(
        out.contains(" * Hello"),
        "default font must serialize as *: {out}"
    );
}

#[test]
fn text_angle_90_aabb_goes_up() {
    let p = parse_primitive_line("TY 301 203 4 2 90 1 0 * Azzurro").unwrap();
    let bb = p.aabb();
    assert!(
        bb.max.y <= 203,
        "90° CCW must not extend below origin, max.y={}",
        bb.max.y
    );
    assert!(
        bb.min.y < 203 - 10,
        "baseline should go up, min.y={}",
        bb.min.y
    );
    assert!(bb.max.x > 301);
}

#[test]
fn text_angle_90_hit_is_above_origin() {
    let p = parse_primitive_line("TY 301 203 4 2 90 1 0 * Azzurro").unwrap();
    assert!(
        p.body_hit(Point::new(301, 198), 0.0),
        "point above origin should hit"
    );
    assert!(
        !p.body_hit(Point::new(301, 208), 0.0),
        "point below origin should miss"
    );
}

#[test]
fn rotate_selected_increments_text_angle() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(sample_text(Point::new(10, 20), "AB"));
    ed.set_selected(vec![0]);
    ed.rotate_selected();
    match &ed.doc().primitives[0] {
        Primitive::Text(t) => assert_eq!(t.angle, 90),
        _ => panic!("expected text"),
    }
    ed.rotate_selected();
    match &ed.doc().primitives[0] {
        Primitive::Text(t) => assert_eq!(t.angle, 180),
        _ => panic!("expected text"),
    }
}

#[test]
fn apply_transform_rotates_text_angle_with_map_coordinates() {
    let mut p = parse_primitive_line("TY 110 100 4 2 180 1 1 * +").unwrap();
    p.apply_transform(Transform {
        origin: COMPONENT_ORIGIN,
        rotations: 1,
        mirrored: false,
    });
    match &p {
        Primitive::Text(t) => {
            // Clockwise MapCoordinates step: (10,0) → (0,10), angle 180 − 90.
            assert_eq!(t.pos, Point::new(100, 110));
            assert_eq!(t.angle, 90);
        }
        other => panic!("expected text, got {other:?}"),
    }

    let mut q = parse_primitive_line("TY 110 100 4 2 180 1 1 * +").unwrap();
    q.apply_transform(Transform {
        origin: COMPONENT_ORIGIN,
        rotations: 3,
        mirrored: false,
    });
    match &q {
        Primitive::Text(t) => {
            // CCW step used by rotate_at: (10,0) → (0,−10), angle 180 + 90.
            assert_eq!(t.pos, Point::new(100, 90));
            assert_eq!(t.angle, 270);
        }
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn apply_transform_mirrors_text_style_like_specchia() {
    let mut p = parse_primitive_line("TY 110 100 4 2 0 1 1 * +").unwrap();
    p.apply_transform(Transform {
        origin: COMPONENT_ORIGIN,
        rotations: 0,
        mirrored: true,
    });
    match &p {
        Primitive::Text(t) => {
            assert_eq!(t.pos, Point::new(90, 100));
            assert_eq!(t.angle, 0);
            assert_ne!(t.style & STYLE_MIRRORED, 0);
        }
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn mirror_selected_toggles_text_mirrored_like_the_property() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(sample_text(Point::new(10, 20), "AB"));
    ed.set_selected(vec![0]);
    match &ed.doc().primitives[0] {
        Primitive::Text(t) => assert_eq!(t.style & STYLE_MIRRORED, 0),
        _ => panic!("expected text"),
    }
    ed.mirror_selected();
    match &ed.doc().primitives[0] {
        Primitive::Text(t) => {
            assert_ne!(
                t.style & STYLE_MIRRORED,
                0,
                "Specchia must set the same mirrored style bit as the text property"
            );
        }
        _ => panic!("expected text"),
    }
    ed.mirror_selected();
    match &ed.doc().primitives[0] {
        Primitive::Text(t) => assert_eq!(t.style & STYLE_MIRRORED, 0),
        _ => panic!("expected text"),
    }
}

#[test]
fn pcb_pad_and_track() {
    let p = parse_primitive_line("PA 100 100 18 18 8 0 1").unwrap();
    match p {
        Primitive::PcbPad(PcbPad { hole, layer, .. }) => {
            assert_eq!(hole, 8);
            assert_eq!(layer.0, 1);
        }
        _ => panic!(),
    }
    let t = parse_primitive_line("PL 0 0 50 0 10 1").unwrap();
    match t {
        Primitive::PcbTrack(PcbTrack { width, layer, .. }) => {
            assert_eq!(width, 10);
            assert_eq!(layer.0, 1);
        }
        _ => panic!(),
    }
}

#[test]
fn empty_document_ok() {
    let d = Document::default();
    let s = serialize_document(&d, None);
    assert!(s.starts_with("[FIDOCAD]"));
}

#[test]
fn reject_zero_length_line() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Line);
    ed.doc_mut().snap = 1;
    let p = Point::new(10, 10);
    ed.pointer_down(p, (0.0, 0.0), false, false);
    ed.pointer_move(p, (0.0, 0.0));
    ed.pointer_up(p);
    assert!(
        ed.doc_mut().primitives.is_empty(),
        "original FidoCAD ignores a second point that coincides with the first"
    );
}

#[test]
fn accept_nonzero_line() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Line);
    ed.doc_mut().snap = 1;
    ed.pointer_down(Point::new(10, 10), (0.0, 0.0), false, false);
    ed.pointer_move(Point::new(30, 10), (20.0, 0.0));
    ed.pointer_up(Point::new(30, 10));
    assert_eq!(ed.doc_mut().primitives.len(), 1);
}

#[test]
fn marquee_rect_while_dragging() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(0, 0), (0.0, 0.0), false, false);
    ed.pointer_move(Point::new(40, 25), (40.0, 25.0));
    assert_eq!(ed.marquee_screen_rect(), Some((0.0, 0.0, 40.0, 25.0)));
    ed.pointer_up(Point::new(40, 25));
    assert!(ed.marquee_screen_rect().is_none());
}

#[test]
fn right_click_rotates_pending_component() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Component);
    ed.set_pending_component(Some("080".into()));
    ed.set_pending_follow(true);
    assert!(ed.right_click(Point::new(20, 20)));
    assert_eq!(ed.pending_rotations(), 1);
    assert!(ed.right_click(Point::new(20, 20)));
    assert_eq!(ed.pending_rotations(), 2);
}

#[test]
fn right_click_on_selected_component_does_not_rotate() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Component);
    ed.set_pending_component(Some("080".into()));
    assert!(!ed.right_click(Point::new(20, 20)));
    assert_eq!(ed.pending_rotations(), 0);
}

#[test]
fn click_places_pending_component_without_follow() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Component);
    ed.set_pending_component(Some("080".into()));
    ed.pointer_down(Point::new(20, 20), (20.0, 20.0), false, false);
    assert_eq!(ed.doc().primitives.len(), 1);
    assert_eq!(ed.tool(), Tool::Component);
    assert_eq!(ed.pending_component(), Some("080"));
    assert!(ed.selected().is_empty());
}

#[test]
fn drop_places_component_and_returns_to_select() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Component);
    ed.set_pending_component(Some("080".into()));
    ed.place_dropped_component(Point::new(20, 20));
    assert_eq!(ed.doc().primitives.len(), 1);
    assert_eq!(ed.tool(), Tool::Select);
    assert!(ed.pending_component().is_none());
    assert_eq!(ed.selected(), &[0]);
}

#[test]
fn right_click_rotates_while_moving_selection() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Select);
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.selected_mut().push(0);
    ed.pointer_down(Point::new(5, 0), (5.0, 0.0), false, false);
    ed.pointer_move(Point::new(8, 0), (8.0, 0.0));
    assert!(ed.right_click(Point::new(8, 0)));
    match &ed.doc_mut().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_ne!((*a, *b), (Point::new(3, 0), Point::new(13, 0)));
        }
        _ => panic!("expected line"),
    }
}

#[test]
fn invert_selection_toggles_indices() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Connection(Connection {
        pos: Point::new(0, 0),
        layer: LayerId(0),
    }));
    ed.doc_mut().insert(Primitive::Connection(Connection {
        pos: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.set_selected(vec![0]);
    ed.invert_selection();
    assert_eq!(ed.selected(), [1].as_slice());
}

fn sample_text(pos: Point, text: &str) -> Primitive {
    Primitive::Text(Text {
        pos,
        sy: 4,
        sx: 3,
        angle: 0,
        style: 0,
        layer: LayerId(0),
        font: DEFAULT_FONT.into(),
        text: text.into(),
        simple: false,
    })
}

#[test]
fn text_hit_matches_glyph_box() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(sample_text(Point::new(10, 20), "AB"));
    // "AB" is 2×3 LU wide and 4 LU tall, origin top-left at (10, 20).
    assert!(ed.begin_text_edit_at(Point::new(11, 21)).is_some());
    ed.cancel_text_edit();
    assert!(ed.begin_text_edit_at(Point::new(13, 14)).is_none());
    assert!(ed.begin_text_edit_at(Point::new(22, 22)).is_none());
}

#[test]
fn text_edit_commit_replaces_content() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(sample_text(Point::new(0, 0), "IN"));
    let session = ed.begin_text_edit_at(Point::new(1, 1)).expect("hit text");
    assert_eq!(session.text, "IN");
    assert_eq!(ed.editing_text(), Some(0));
    ed.commit_text_edit("OUT".into());
    assert!(ed.editing_text().is_none());
    assert!(ed.can_undo());
    match &ed.doc_mut().primitives[0] {
        Primitive::Text(Text { text, .. }) => assert_eq!(text, "OUT"),
        _ => panic!("expected text"),
    }
}

#[test]
fn dblclick_finishes_poly_instead_of_text_edit() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Poly);
    ed.doc_mut().snap = 1;
    ed.pointer_down(Point::new(0, 0), (0.0, 0.0), false, false);
    ed.pointer_up(Point::new(0, 0));
    ed.pointer_down(Point::new(10, 0), (10.0, 0.0), false, false);
    ed.pointer_up(Point::new(10, 0));
    assert!(ed.begin_text_edit_at(Point::new(5, 0)).is_none());
    assert!(ed
        .doc()
        .primitives
        .iter()
        .any(|p| matches!(p, Primitive::Poly(Poly { .. }))));
}

#[test]
fn dblclick_opens_properties_on_non_text() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    assert_eq!(
        ed.handle_dblclick(Point::new(5, 0)),
        DblClickAction::OpenProperties
    );
    assert_eq!(ed.selected(), &[0]);
}

#[test]
fn dblclick_empty_space_does_not_open_properties() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    *ed.selected_mut() = vec![0];
    assert_eq!(ed.handle_dblclick(Point::new(80, 80)), DblClickAction::None);
}

#[test]
fn dblclick_keeps_multi_selection_for_properties() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 5),
        b: Point::new(10, 5),
        layer: LayerId(0),
    }));
    *ed.selected_mut() = vec![0, 1];
    assert_eq!(
        ed.handle_dblclick(Point::new(5, 0)),
        DblClickAction::OpenProperties
    );
    assert_eq!(ed.selected(), &[0, 1]);
}

#[test]
fn snap_xy_independent_and_disable() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 10;
    ed.doc_mut().snap_y = 5;
    assert_eq!(ed.snap_pt(Point::new(14, 8)), Point::new(10, 10));
    ed.set_snap_enable(false);
    assert_eq!(ed.snap_pt(Point::new(14, 8)), Point::new(14, 8));
}

#[test]
fn file_without_ld_uses_four_fidocad_layers() {
    let doc = parse_document("[FIDOCAD]\nLI 0 0 10 10\nLI 0 0 10 10 1\n").unwrap();
    assert_eq!(doc.layers.len(), 4);
    assert_eq!(doc.layers.get(0).unwrap().name, "Schema");
    assert_eq!(doc.layers.get(1).unwrap().name, "PCB lato rame");
    assert_eq!(doc.layers.get(1).unwrap().color, [0, 0, 192, 255]);
}

#[test]
fn file_without_ld_pads_generic_layers() {
    let doc = parse_document("[FIDOCAD]\nLI 0 0 10 10 6\n").unwrap();
    assert_eq!(doc.layers.len(), 7);
    assert_eq!(doc.layers.get(6).unwrap().name, "Layer 7");
    match &doc.primitives[0] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 6),
        _ => panic!(),
    }
}

#[test]
fn ld_lines_replace_defaults() {
    let src = "[FIDOCAD]\nLD 10 20 30 0 Bottom copper\nLD 255 0 0 1 Top\nLI 0 0 10 10 1\n";
    let doc = parse_document(src).unwrap();
    assert_eq!(doc.layers.len(), 2);
    assert_eq!(doc.layers.get(0).unwrap().name, "Bottom copper");
    assert_eq!(doc.layers.get(0).unwrap().color, [10, 20, 30, 255]);
    assert!(!doc.layers.get(0).unwrap().show);
    assert_eq!(doc.layers.get(1).unwrap().name, "Top");
    assert!(doc.layers.get(1).unwrap().show);
    match &doc.primitives[0] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 1),
        _ => panic!(),
    }
}

#[test]
fn ld_out_of_range_primitive_clamps_to_zero() {
    let src = "[FIDOCAD]\nLD 0 0 0 1 Only\nLI 0 0 10 10 3\n";
    let doc = parse_document(src).unwrap();
    assert_eq!(doc.layers.len(), 1);
    match &doc.primitives[0] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 0),
        _ => panic!(),
    }
}

#[test]
fn serialize_writes_ld_and_roundtrips() {
    let src = "[FIDOCAD Title]\nLD 1 2 3 0 Hidden sheet\nLD 0 80 200 1 Rame\nLI 1 2 3 4 1\n";
    let doc = parse_document(src).unwrap();
    let out = serialize_document(&doc, None);
    assert!(out.starts_with("[FIDOCAD Title]\r\nLD 1 2 3 0 Hidden sheet\r\nLD 0 80 200 1 Rame\r\nPS 5 5 5 5 1 1 1 25 0\r\n"));
    let doc2 = parse_document(&out).unwrap();
    assert_eq!(doc.layers.len(), doc2.layers.len());
    assert_eq!(
        doc.layers.get(0).unwrap().name,
        doc2.layers.get(0).unwrap().name
    );
    assert_eq!(
        doc.layers.get(0).unwrap().show,
        doc2.layers.get(0).unwrap().show
    );
    assert_eq!(doc.primitives.len(), doc2.primitives.len());
}

#[test]
fn ld_optional_alpha_roundtrips() {
    let src = "[FIDOCAD]\nLD 10 20 30 1 128 Overlay\nLI 0 0 10 10\n";
    let doc = parse_document(src).unwrap();
    assert_eq!(doc.layers.get(0).unwrap().color, [10, 20, 30, 128]);
    let out = serialize_document(&doc, None);
    assert!(out.contains("LD 10 20 30 1 128 Overlay"));
    let doc2 = parse_document(&out).unwrap();
    assert_eq!(doc2.layers.get(0).unwrap().color, [10, 20, 30, 128]);
}

#[test]
fn new_document_has_four_fallback_layers() {
    let d = Document::default();
    assert_eq!(d.layers.len(), 4);
    let s = serialize_document(&d, None);
    assert!(s.contains("LD 0 0 0 1 Schema"));
    assert!(s.contains("LD 0 0 192 1 PCB lato rame"));
    assert!(s.contains("PS 5 5 5 5 1 1 1 25 0"));
}

#[test]
fn ps_line_roundtrips() {
    let src = "[FIDOCAD]\nLD 0 0 0 1 Schema\nPS 10 8 4 2 0 0 0 50 1\nLI 0 0 10 10\n";
    let doc = parse_document(src).unwrap();
    assert_eq!(doc.grid, 10);
    assert_eq!(doc.grid_y, 8);
    assert_eq!(doc.snap, 4);
    assert_eq!(doc.snap_y, 2);
    assert!(!doc.show_grid);
    assert!(!doc.snap_enable);
    assert!(!doc.hide_component_origin);
    assert_eq!(doc.stroke_hundredths, 50);
    assert!(doc.default_filled);
    let out = serialize_document(&doc, None);
    assert!(out.contains("PS 10 8 4 2 0 0 0 50 1"));
    let doc2 = parse_document(&out).unwrap();
    assert_eq!(doc.project_settings(), doc2.project_settings());
}

#[test]
fn missing_ps_keeps_defaults() {
    let doc = parse_document("[FIDOCAD]\nLI 0 0 10 10\n").unwrap();
    assert_eq!(
        doc.project_settings(),
        fidocad_core::ProjectSettings::default()
    );
}

#[test]
fn ps_trailing_fields_optional_and_last_wins() {
    let src = "[FIDOCAD]\nPS 10 10 5 5 1 1 1 25 0\nPS 8 8\nLI 0 0 1 1\n";
    let doc = parse_document(src).unwrap();
    assert_eq!(doc.grid, 8);
    assert_eq!(doc.grid_y, 8);
    assert_eq!(doc.snap, 5);
    assert!(doc.show_grid);
    assert_eq!(doc.stroke_hundredths, 25);
    assert!(!doc.default_filled);
}

#[test]
fn delete_layer_remaps_and_can_move_objects() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(1),
    }));
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 5),
        b: Point::new(10, 5),
        layer: LayerId(2),
    }));
    assert!(ed.delete_layer(1, Some(0)));
    assert_eq!(ed.doc_mut().layers.len(), 3);
    match &ed.doc_mut().primitives[0] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 0),
        _ => panic!(),
    }
    match &ed.doc_mut().primitives[1] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 1),
        _ => panic!(),
    }
}

#[test]
fn delete_layer_drops_objects() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(1),
    }));
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 5),
        b: Point::new(10, 5),
        layer: LayerId(0),
    }));
    assert!(ed.delete_layer(1, None));
    assert_eq!(ed.doc_mut().primitives.len(), 1);
    match &ed.doc_mut().primitives[0] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 0),
        _ => panic!(),
    }
}

#[test]
fn reorder_layer_remaps_primitive_ids() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 5),
        b: Point::new(10, 5),
        layer: LayerId(2),
    }));
    let name0 = ed.doc_mut().layers.get(0).unwrap().name.clone();
    let name2 = ed.doc_mut().layers.get(2).unwrap().name.clone();
    assert!(ed.reorder_layer(2, 0));
    assert_eq!(ed.doc_mut().layers.get(0).unwrap().name, name2);
    assert_eq!(ed.doc_mut().layers.get(1).unwrap().name, name0);
    match &ed.doc_mut().primitives[0] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 1),
        _ => panic!(),
    }
    match &ed.doc_mut().primitives[1] {
        Primitive::Line(Line { layer, .. }) => assert_eq!(layer.0, 0),
        _ => panic!(),
    }
}

#[test]
fn cannot_delete_last_layer() {
    let mut ed = Editor::new(builtin_libraries());
    while ed.doc_mut().layers.len() > 1 {
        assert!(ed.delete_layer(0, None));
    }
    assert!(!ed.delete_layer(0, None));
    assert_eq!(ed.doc_mut().layers.len(), 1);
}

#[test]
fn layer_color_drag_is_one_undo() {
    let mut ed = Editor::new(builtin_libraries());
    let original = ed.doc().layers.get(0).unwrap().color;
    assert!(ed.set_layer_color(0, [1, 2, 3, 128]));
    assert!(ed.set_layer_color(0, [4, 5, 6, 64]));
    assert_eq!(ed.doc().layers.get(0).unwrap().color, [4, 5, 6, 64]);
    ed.undo();
    assert_eq!(ed.doc().layers.get(0).unwrap().color, original);
    assert!(!ed.can_undo());
}

#[test]
fn empty_selection_move_is_undoable() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.pointer_move(Point::new(15, 0), (60.0, 0.0));
    ed.pointer_up(Point::new(15, 0));
    assert!(ed.can_undo());
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(10, 0), Point::new(20, 0)));
        }
        _ => panic!("expected line"),
    }
    ed.undo();
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(0, 0), Point::new(10, 0)));
        }
        _ => panic!("expected line"),
    }
    assert!(!ed.can_undo());
    assert!(ed.can_redo());
}

#[test]
fn select_click_without_move_is_not_undoable() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.set_tool(Tool::Select);
    ed.selected_mut().push(0);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.pointer_up(Point::new(5, 0));
    assert!(!ed.can_undo());
}

#[test]
fn apply_props_and_grid_are_undoable() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Rect(Rect {
        a: Point::new(0, 0),
        b: Point::new(10, 10),
        filled: false,
        layer: LayerId(0),
    }));
    ed.selected_mut().push(0);
    let applied = ed
        .apply_selection_props_patch(&PropPatch {
            filled: Some(true),
            ..Default::default()
        })
        .unwrap();
    assert!(applied);
    assert!(ed.can_undo());
    match &ed.doc().primitives[0] {
        Primitive::Rect(Rect { filled, .. }) => assert!(filled),
        _ => panic!("expected rect"),
    }
    ed.undo();
    match &ed.doc().primitives[0] {
        Primitive::Rect(Rect { filled, .. }) => assert!(!filled),
        _ => panic!("expected rect"),
    }

    ed.set_grid(10, 8);
    assert!(ed.can_undo());
    assert_eq!(ed.doc().grid, 10);
    assert_eq!(ed.doc().grid_y, 8);
    ed.set_grid(10, 8);
    ed.undo();
    assert_eq!(ed.doc().grid, 5);
    assert_eq!(ed.doc().grid_y, 5);
    assert!(!ed.can_undo());

    ed.set_pcb_mode(true);
    assert!(ed.can_undo());
    assert!(ed.doc().pcb_mode);
    ed.set_pcb_mode(true);
    ed.undo();
    assert!(!ed.doc().pcb_mode);

    let custom = fidocad_core::ProjectSettings {
        grid: 12,
        stroke_hundredths: 80,
        default_filled: true,
        show_grid: false,
        ..Default::default()
    };
    ed.apply_project_settings(custom);
    assert!(ed.can_undo());
    assert_eq!(ed.doc().grid, 12);
    assert_eq!(ed.doc().stroke_hundredths, 80);
    assert!(ed.doc().default_filled);
    assert!(!ed.doc().show_grid);
    ed.apply_project_settings(custom);
    ed.undo();
    assert_eq!(
        ed.doc().project_settings(),
        fidocad_core::ProjectSettings::default()
    );
}

#[test]
fn load_text_clears_history() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.set_grid(10, 10);
    assert!(ed.can_undo());
    ed.load_text("[FIDOCAD]\nLI 1 2 3 4\n").unwrap();
    assert!(!ed.can_undo());
    assert!(!ed.can_redo());
    assert_eq!(ed.doc().primitives.len(), 1);
}

fn insert_h_line(ed: &mut Editor, a: Point, b: Point) {
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a,
        b,
        layer: LayerId(0),
    }));
}

#[test]
fn duplicate_selection_places_bbox_min_at_cursor() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_selected(vec![0]);
    ed.set_hover(Some(Point::new(50, 20)));
    ed.duplicate_selection();
    assert_eq!(ed.doc().primitives.len(), 2);
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(0, 0), Point::new(10, 0)));
        }
        _ => panic!("expected original line"),
    }
    match &ed.doc().primitives[1] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(50, 20), Point::new(60, 20)));
        }
        _ => panic!("expected cloned line"),
    }
    assert_eq!(ed.selected(), &[1]);
}

#[test]
fn duplicate_drag_keeps_original_and_inserts_clone() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.set_move_duplicate(true);
    ed.pointer_move(Point::new(15, 0), (60.0, 0.0));
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(0, 0), Point::new(10, 0)));
        }
        _ => panic!("expected original line"),
    }
    assert!(ed.duplicate_drag());
    assert!(!ed.duplicate_drag_preview().is_empty());
    ed.pointer_up(Point::new(15, 0));
    assert_eq!(ed.doc().primitives.len(), 2);
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(0, 0), Point::new(10, 0)));
        }
        _ => panic!("expected original line"),
    }
    match &ed.doc().primitives[1] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(10, 0), Point::new(20, 0)));
        }
        _ => panic!("expected cloned line"),
    }
    assert_eq!(ed.selected(), &[1]);
    assert!(ed.can_undo());
}

#[test]
fn duplicate_drag_toggle_restores_then_moves() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.pointer_move(Point::new(15, 0), (60.0, 0.0));
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(10, 0), Point::new(20, 0)));
        }
        _ => panic!("expected moved line"),
    }
    ed.set_move_duplicate(true);
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(0, 0), Point::new(10, 0)));
        }
        _ => panic!("expected restored line"),
    }
    assert!(ed.duplicate_drag());
    ed.set_move_duplicate(false);
    match &ed.doc().primitives[0] {
        Primitive::Line(Line { a, b, .. }) => {
            assert_eq!((*a, *b), (Point::new(10, 0), Point::new(20, 0)));
        }
        _ => panic!("expected line under cursor"),
    }
    assert!(!ed.duplicate_drag());
}

#[test]
fn duplicate_drag_zero_delta_does_not_clone() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.set_move_duplicate(true);
    ed.pointer_up(Point::new(5, 0));
    assert_eq!(ed.doc().primitives.len(), 1);
    assert!(!ed.can_undo());
}

fn line_ends(p: &Primitive) -> (Point, Point) {
    match p {
        Primitive::Line(Line { a, b, .. }) => (*a, *b),
        _ => panic!("expected line"),
    }
}

#[test]
fn stamp_drag_copy_leaves_original_selected() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.pointer_move(Point::new(15, 0), (60.0, 0.0));
    assert!(ed.stamp_drag_copy());
    assert_eq!(ed.doc().primitives.len(), 2);
    assert_eq!(ed.selected(), &[0]);
    assert_eq!(
        line_ends(&ed.doc().primitives[0]),
        (Point::new(10, 0), Point::new(20, 0))
    );
    assert_eq!(
        line_ends(&ed.doc().primitives[1]),
        (Point::new(10, 0), Point::new(20, 0))
    );
    ed.pointer_move(Point::new(25, 0), (100.0, 0.0));
    assert_eq!(
        line_ends(&ed.doc().primitives[0]),
        (Point::new(20, 0), Point::new(30, 0))
    );
    assert_eq!(
        line_ends(&ed.doc().primitives[1]),
        (Point::new(10, 0), Point::new(20, 0))
    );
    ed.pointer_up(Point::new(25, 0));
    assert_eq!(ed.doc().primitives.len(), 2);
    assert_eq!(ed.selected(), &[0]);
}

#[test]
fn stamp_drag_copy_can_drop_several() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.pointer_move(Point::new(15, 0), (60.0, 0.0));
    assert!(ed.stamp_drag_copy());
    ed.pointer_move(Point::new(25, 0), (100.0, 0.0));
    assert!(ed.stamp_drag_copy());
    ed.pointer_up(Point::new(25, 0));
    assert_eq!(ed.doc().primitives.len(), 3);
    assert_eq!(ed.selected(), &[0]);
    assert_eq!(
        line_ends(&ed.doc().primitives[0]),
        (Point::new(20, 0), Point::new(30, 0))
    );
    assert_eq!(
        line_ends(&ed.doc().primitives[1]),
        (Point::new(10, 0), Point::new(20, 0))
    );
    assert_eq!(
        line_ends(&ed.doc().primitives[2]),
        (Point::new(20, 0), Point::new(30, 0))
    );
}

#[test]
fn stamp_drag_copy_in_duplicate_mode_uses_ghost_offset() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (20.0, 0.0), false, false);
    ed.set_move_duplicate(true);
    ed.pointer_move(Point::new(15, 0), (60.0, 0.0));
    assert!(ed.stamp_drag_copy());
    assert_eq!(ed.doc().primitives.len(), 2);
    assert_eq!(ed.selected(), &[0]);
    assert_eq!(
        line_ends(&ed.doc().primitives[0]),
        (Point::new(0, 0), Point::new(10, 0))
    );
    assert_eq!(
        line_ends(&ed.doc().primitives[1]),
        (Point::new(10, 0), Point::new(20, 0))
    );
}

#[test]
fn stamp_drag_copy_without_move_drag_is_noop() {
    let mut ed = Editor::new(builtin_libraries());
    insert_h_line(&mut ed, Point::new(0, 0), Point::new(10, 0));
    ed.set_selected(vec![0]);
    assert!(!ed.stamp_drag_copy());
    assert_eq!(ed.doc().primitives.len(), 1);
}
