mod common;

use fidocad_core::parse::{builtin_libraries, parse_document, parse_library};
use fidocad_core::properties::{apply_selection_props, selection_props_form, PropPatch};
use fidocad_core::serialize::{serialize_clipboard, serialize_document};
use fidocad_core::{
    Bezier, ComponentRef, Connection, Editor, Ellipse, LayerId, Line, PcbPad, PcbTrack, Point,
    Poly, Primitive, Rect, Text, Tool,
};

fn roundtrip(src: &str) -> String {
    let doc = parse_document(src).expect("parse");
    serialize_document(&doc, None)
}

#[test]
fn snapshot_alimentatore_roundtrip() {
    let src = include_str!("Alimentatore.fcd");
    common::assert_snapshot("alimentatore.fcd", &roundtrip(src));
}

#[test]
fn snapshot_sample_roundtrip() {
    let src = include_str!("../../../apps/ui/src/lib/sample.fcd");
    common::assert_snapshot("sample.fcd", &roundtrip(src));
}

#[test]
fn snapshot_library_tree() {
    let libs = builtin_libraries();
    let json = serde_json::to_string_pretty(&libs.tree()).unwrap();
    common::assert_snapshot("library_tree.json", &json);
}

#[test]
fn snapshot_stdlib_expanded_macros() {
    let text = include_str!("../libraries/stdlib.fcl");
    let lib = parse_library(text).unwrap();
    let libs = builtin_libraries();
    let mut out = String::new();
    for m in &lib.components {
        out.push_str(&format!("# {}\n", m.key));
        for p in &m.primitives {
            for q in fidocad_core::library::expand_primitive(p, &libs) {
                out.push_str(&fidocad_core::serialize::serialize_primitive(&q));
            }
        }
    }
    common::assert_snapshot("stdlib_expanded.fcd", &out);
}

#[test]
fn snapshot_pcb_expanded_macros() {
    let text = fidocad_core::parse::decode_bytes(include_bytes!("../libraries/PCB.fcl"));
    let lib = parse_library(&text).unwrap();
    let libs = builtin_libraries();
    let mut out = String::new();
    for m in &lib.components {
        out.push_str(&format!("# {}\n", m.key));
        for p in &m.primitives {
            for q in fidocad_core::library::expand_primitive(p, &libs) {
                out.push_str(&fidocad_core::serialize::serialize_primitive(&q));
            }
        }
    }
    common::assert_snapshot("pcb_expanded.fcd", &out);
}

#[test]
fn snapshot_lib1_expanded_macros() {
    let text = include_str!("../libraries/lib1.fcl");
    let lib = parse_library(text).unwrap();
    let libs = builtin_libraries();
    let mut out = String::new();
    for m in &lib.components {
        out.push_str(&format!("# {}\n", m.key));
        for p in &m.primitives {
            for q in fidocad_core::library::expand_primitive(p, &libs) {
                out.push_str(&fidocad_core::serialize::serialize_primitive(&q));
            }
        }
    }
    common::assert_snapshot("lib1_expanded.fcd", &out);
}

#[test]
fn snapshot_selection_props_mixed() {
    let line = Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 10),
        layer: LayerId(1),
    });
    let rect = Primitive::Rect(Rect {
        a: Point::new(0, 0),
        b: Point::new(10, 10),
        filled: true,
        layer: LayerId(1),
    });
    let text = Primitive::Text(Text {
        pos: Point::new(0, 0),
        sy: 4,
        sx: 3,
        angle: 0,
        style: 0,
        layer: LayerId(0),
        font: "Courier New".into(),
        text: "AB".into(),
        simple: false,
    });
    let pad = Primitive::PcbPad(PcbPad {
        pos: Point::new(0, 0),
        dx: 18,
        dy: 18,
        hole: 8,
        style: fidocad_core::PadStyle::Oval,
        layer: LayerId(2),
    });
    let homogeneous = selection_props_form(&[&rect, &rect]);
    let mixed = selection_props_form(&[&line, &rect]);
    let texts = selection_props_form(&[&text]);
    let pads = selection_props_form(&[&pad]);
    let json = serde_json::to_string_pretty(&serde_json::json!({
        "homogeneous_rects": homogeneous,
        "mixed_line_rect": mixed,
        "text": texts,
        "pad": pads,
    }))
    .unwrap();
    common::assert_snapshot("selection_props.json", &json);
}

#[test]
fn snapshot_primitive_json_shapes() {
    let prims = [
        Primitive::Line(Line {
            a: Point::new(1, 2),
            b: Point::new(3, 4),
            layer: LayerId(1),
        }),
        Primitive::Rect(Rect {
            a: Point::new(0, 0),
            b: Point::new(5, 5),
            filled: true,
            layer: LayerId(0),
        }),
        Primitive::Ellipse(Ellipse {
            a: Point::new(0, 0),
            b: Point::new(8, 4),
            filled: false,
            layer: LayerId(2),
        }),
        Primitive::Poly(Poly {
            pts: vec![Point::new(0, 0), Point::new(1, 0), Point::new(1, 1)],
            filled: false,
            layer: LayerId(0),
        }),
        Primitive::Bezier(Bezier {
            p0: Point::new(0, 0),
            p1: Point::new(1, 2),
            p2: Point::new(3, 2),
            p3: Point::new(4, 0),
            layer: LayerId(0),
        }),
        Primitive::Text(Text {
            pos: Point::new(0, 0),
            sy: 4,
            sx: 3,
            angle: 90,
            style: 3,
            layer: LayerId(1),
            font: "Courier New".into(),
            text: "Hi".into(),
            simple: false,
        }),
        Primitive::Connection(Connection {
            pos: Point::new(7, 8),
            layer: LayerId(0),
        }),
        Primitive::PcbTrack(PcbTrack {
            a: Point::new(0, 0),
            b: Point::new(10, 0),
            width: 4,
            layer: LayerId(1),
        }),
        Primitive::PcbPad(PcbPad {
            pos: Point::new(5, 5),
            dx: 18,
            dy: 12,
            hole: 8,
            style: fidocad_core::PadStyle::RoundedRect,
            layer: LayerId(1),
        }),
        Primitive::Component(ComponentRef {
            pos: Point::new(10, 20),
            rotations: 1,
            mirrored: true,
            name: "080".into(),
            standard: true,
            layer: LayerId(0),
        }),
    ];
    common::assert_snapshot(
        "primitive_json.json",
        &serde_json::to_string_pretty(&prims).unwrap(),
    );
}

#[test]
fn snapshot_editor_script() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.set_snap_enable(true);
    ed.set_tool(Tool::Line);
    ed.pointer_down(Point::new(0, 0), (0.0, 0.0), false, false);
    ed.pointer_move(Point::new(10, 0), (10.0, 0.0));
    ed.pointer_up(Point::new(10, 0));
    ed.set_tool(Tool::Rect);
    ed.set_filled(true);
    ed.pointer_down(Point::new(0, 10), (0.0, 10.0), false, false);
    ed.pointer_move(Point::new(8, 18), (8.0, 18.0));
    ed.pointer_up(Point::new(8, 18));
    ed.set_tool(Tool::Ellipse);
    ed.set_filled(false);
    ed.pointer_down(Point::new(20, 0), (20.0, 0.0), false, false);
    ed.pointer_move(Point::new(30, 10), (30.0, 10.0));
    ed.pointer_up(Point::new(30, 10));
    ed.set_tool(Tool::Connection);
    ed.pointer_down(Point::new(5, 5), (5.0, 5.0), false, false);
    ed.pointer_up(Point::new(5, 5));
    ed.set_tool(Tool::PcbTrack);
    ed.pointer_down(Point::new(0, 30), (0.0, 30.0), false, false);
    ed.pointer_move(Point::new(20, 30), (20.0, 30.0));
    ed.pointer_up(Point::new(20, 30));
    ed.set_tool(Tool::PcbPad);
    ed.pointer_down(Point::new(40, 40), (40.0, 40.0), false, false);
    ed.pointer_up(Point::new(40, 40));
    ed.set_tool(Tool::Text);
    ed.set_pending_text("HI".into());
    ed.pointer_down(Point::new(50, 0), (50.0, 0.0), false, false);
    ed.pointer_up(Point::new(50, 0));
    ed.set_tool(Tool::Poly);
    ed.pointer_down(Point::new(60, 0), (60.0, 0.0), false, false);
    ed.pointer_up(Point::new(60, 0));
    ed.pointer_down(Point::new(70, 0), (70.0, 0.0), false, false);
    ed.pointer_up(Point::new(70, 0));
    ed.pointer_down(Point::new(70, 10), (70.0, 10.0), false, false);
    ed.pointer_up(Point::new(70, 10));
    ed.finish_poly();
    ed.set_tool(Tool::Bezier);
    ed.pointer_down(Point::new(80, 0), (80.0, 0.0), false, false);
    ed.pointer_up(Point::new(80, 0));
    ed.pointer_down(Point::new(85, 5), (85.0, 5.0), false, false);
    ed.pointer_up(Point::new(85, 5));
    ed.pointer_down(Point::new(90, 5), (90.0, 5.0), false, false);
    ed.pointer_up(Point::new(90, 5));
    ed.pointer_down(Point::new(95, 0), (95.0, 0.0), false, false);
    ed.pointer_up(Point::new(95, 0));
    ed.set_pending_component(Some("080".into()));
    ed.adopt_component_tool();
    ed.insert_pending_component_at(Point::new(100, 20));
    ed.select_all();
    let clip = serialize_clipboard(
        &ed.selected()
            .iter()
            .filter_map(|&i| ed.doc().primitives.get(i).cloned())
            .collect::<Vec<_>>(),
    );
    ed.rotate_selected();
    ed.mirror_selected();
    ed.split_selected_components();
    ed.undo();
    ed.redo();
    ed.set_selected(vec![0]);
    ed.set_selected_layer(LayerId(1));
    let _ = ed.add_layer();
    ed.reorder_layer(0, 1);
    ed.set_layer_name(0, "Renamed".into());
    ed.set_layer_show(0, false);
    ed.set_layer_color(1, [9, 8, 7, 255]);
    let patch = PropPatch {
        layer: Some(2),
        ..Default::default()
    };
    let mut targets: Vec<Primitive> = ed
        .selected()
        .iter()
        .filter_map(|&i| ed.doc().primitives.get(i).cloned())
        .collect();
    apply_selection_props(&mut targets, &patch);
    let out = serialize_document(ed.doc(), Some(ed.libs()));
    let mut dump = String::new();
    dump.push_str(&out);
    dump.push_str("\n---clipboard---\n");
    dump.push_str(&clip);
    dump.push_str(&format!(
        "\n---meta---\nundo={} redo={} sel={:?} layer={}\n",
        ed.can_undo(),
        ed.can_redo(),
        ed.selected(),
        ed.layer().0
    ));
    common::assert_snapshot("editor_script.fcd", &dump);
}

#[test]
fn snapshot_opcode_roundtrip_matrix() {
    let src = "\
[FIDOCAD Matrix]
LI 1 2 3 4
LI 1 2 3 4 2
SA 5 6
SA 5 6 1
BE 0 0 1 2 3 2 4 0
BE 0 0 1 2 3 2 4 0 3
RP 0 0 10 10
RV 1 1 8 8 1
EP 0 0 10 6
EV 0 0 10 6 2
PP 0 0 4 0 4 4 0 4
PV 0 0 4 0 4 4 1
PL 0 0 20 0 4
PL 0 0 20 0 4 2
PA 10 10 18 18 8 0
PA 10 10 18 12 8 2 1
MC 10 20 1 1 080
TE 0 0 HELLO
TX 0 0 5 3 0 0 * WORLD
TY 0 0 5 3 90 3 1 Courier++New HI
";
    common::assert_snapshot("opcode_matrix.fcd", &roundtrip(src));
}
