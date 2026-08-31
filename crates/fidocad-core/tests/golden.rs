use fidocad_core::parse::{builtin_libraries, parse_document, parse_primitive_line};
use fidocad_core::serialize::{serialize_document, serialize_primitive};
use fidocad_core::{Document, Editor, Point, Primitive, SaveOptions, Tool};

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
    let (a, b) = ed.marquee_rect().expect("marquee should be live while dragging");
    assert_eq!(a, Point::new(0, 0));
    assert_eq!(b, Point::new(40, 25));
    ed.pointer_up(Point::new(40, 25));
    assert!(ed.marquee_rect().is_none());
}
