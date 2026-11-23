use fidorust_core::parse::builtin_libraries;
use fidorust_core::serialize::serialize_document;
use fidorust_core::{Editor, LayerId, PadAccess, PadLink, PcbPad, Point, Primitive};

fn link(pad_sheet: &str, dot_sheet: &str, x: i32, y: i32) -> PadLink {
    PadLink {
        pad_sheet: pad_sheet.into(),
        x,
        y,
        layer: 0,
        dot_sheet: dot_sheet.into(),
        dx: x + 5,
        dy: y + 5,
    }
}

#[test]
fn fcd_roundtrip_keeps_links_and_opaque_lines() {
    let src = "\
[FIDOCAD]
[FIDOSHEET Foglio 1]
LI 0 0 10 10
[FIDOSHEET Board]
PA 12 34 10 10 4 0 0
LK Foglio++1 12 34 0 Board 80 20
[FIDOEXT pcb]
NOTE keep this
";
    let doc = fidorust_core::parse_document(src).expect("parse");
    assert_eq!(doc.warnings, 0, "ext lines must not warn");
    assert_eq!(doc.pad_links.len(), 1);
    assert_eq!(doc.pad_links[0].pad_sheet, "Foglio 1");
    assert_eq!(doc.pad_links[0].dot_sheet, "Board");
    assert_eq!(doc.pad_links[0].pad_pos(), Point::new(12, 34));
    assert_eq!(doc.pad_links[0].dot_pos(), Point::new(80, 20));
    assert_eq!(doc.pcb_ext_lines, vec!["NOTE keep this".to_string()]);

    let out = serialize_document(&doc, None);
    assert!(
        out.contains("LK Foglio++1 12 34 0 Board 80 20\r\n"),
        "{out}"
    );
    assert!(!out.contains('{'), "pad links must not be JSON: {out}");
    let lib_at = out.find("[FIDOLIB").unwrap_or(out.len());
    let lk_at = out.find("LK Foglio++1").expect("lk");
    assert!(lk_at < lib_at, "LK stays before a library");
    assert!(out.contains("[FIDOEXT pcb]\r\n"), "{out}");
    assert!(out.contains("NOTE keep this\r\n"), "{out}");
    let again = fidorust_core::parse_document(&out).expect("reparse");
    assert_eq!(again.pad_links, doc.pad_links);
    assert_eq!(again.pcb_ext_lines, doc.pcb_ext_lines);
}

#[test]
fn empty_document_omits_fidoext_block() {
    let doc = fidorust_core::parse_document("[FIDOCAD]\nLI 0 0 1 1\n").unwrap();
    let out = serialize_document(&doc, None);
    assert!(!out.contains("FIDOEXT"), "{out}");
    assert!(!out.contains("LK "), "{out}");
    assert!(doc.pad_links.is_empty());
}

#[test]
fn json_lk_inside_fidoext_still_loads() {
    let src = "\
[FIDOCAD]
LI 0 0 1 1
[FIDOEXT pcb]
LK {\"padSheet\":\"Foglio 1\",\"x\":12,\"y\":34,\"layer\":0,\"dotSheet\":\"Board\",\"dx\":80,\"dy\":20}
";
    let doc = fidorust_core::parse_document(src).expect("parse");
    assert_eq!(doc.pad_links.len(), 1);
    assert_eq!(doc.pad_links[0].pad_sheet, "Foglio 1");
    let out = serialize_document(&doc, None);
    assert!(
        out.contains("LK Foglio++1 12 34 0 Board 80 20\r\n"),
        "{out}"
    );
    assert!(!out.contains("[FIDOEXT pcb]"), "{out}");
}

#[test]
fn rename_and_delete_sheet_update_links_and_undo() {
    let mut ed = Editor::new(builtin_libraries());
    let second = ed.add_sheet();
    let names = ed.sheet_names();
    ed.doc_mut().pad_links = vec![link(&names[0], &names[1], 1, 2)];
    assert!(ed.rename_sheet(second, "Board"));
    assert_eq!(ed.doc().pad_links[0].dot_sheet, "Board");
    assert_eq!(ed.doc().pad_links[0].pad_sheet, names[0]);
    ed.undo();
    assert_eq!(ed.doc().pad_links[0].dot_sheet, names[1]);
    assert_eq!(ed.sheet_names()[second], names[1]);

    assert!(ed.delete_sheet(second));
    assert!(ed.doc().pad_links.is_empty());
    ed.undo();
    assert_eq!(ed.doc().pad_links.len(), 1);
    assert_eq!(ed.doc().pad_links[0].dot_sheet, names[1]);
}

#[test]
fn link_edit_and_pad_move_share_one_undo_step() {
    let mut ed = Editor::new(builtin_libraries());
    let name = ed.sheet_names()[0].clone();
    ed.doc_mut().insert(Primitive::PcbPad(PcbPad {
        pos: Point::new(10, 20),
        dx: 12,
        dy: 12,
        hole: 4,
        style: fidorust_core::PadStyle::Oval,
        layer: LayerId(0),
        access: PadAccess::Both,
    }));
    ed.doc_mut().pad_links = vec![link(&name, &name, 10, 20)];
    ed.snapshot_sheet_undo();
    match &mut ed.doc_mut().primitives[0] {
        Primitive::PcbPad(pad) => pad.pos = Point::new(30, 40),
        _ => panic!("pad"),
    }
    ed.doc_mut().pad_links[0].set_pad_pos(Point::new(30, 40));
    ed.undo();
    match &ed.doc().primitives[0] {
        Primitive::PcbPad(pad) => assert_eq!(pad.pos, Point::new(10, 20)),
        _ => panic!("pad"),
    }
    assert_eq!(ed.doc().pad_links[0].pad_pos(), Point::new(10, 20));
    ed.redo();
    assert_eq!(ed.doc().pad_links[0].pad_pos(), Point::new(30, 40));
}

#[test]
fn import_remaps_sheet_names_inside_links() {
    let mut ed = Editor::new(builtin_libraries());
    assert!(ed.rename_sheet(0, "Alpha"));
    let src = "\
[FIDOCAD]
[FIDOSHEET Alpha]
LI 0 0 1 1
[FIDOSHEET Beta]
LI 2 2 3 3
LK Alpha 1 2 3 Beta 4 5
";
    ed.import_fcd_sheets(src).unwrap();
    let names = ed.sheet_names();
    assert!(names.iter().any(|n| n == "Alpha 2"), "{names:?}");
    assert!(names.iter().any(|n| n == "Beta"), "{names:?}");
    let imported = ed
        .doc()
        .pad_links
        .iter()
        .find(|l| l.x == 1)
        .expect("imported link");
    assert_eq!(imported.pad_sheet, "Alpha 2");
    assert_eq!(imported.dot_sheet, "Beta");
    assert_eq!(imported.layer, 3);
}
