use fidorust_core::parse::{builtin_libraries, parse_document};
use fidorust_core::{
    Editor, Ellipse, LayerId, Line, PcbPad, PcbTrack, Point, Primitive, Rect, Text, Tool,
    DEFAULT_FONT,
};
use fidorust_gpu::font::{glyph_covers, install_hit_hooks};
use fidorust_gpu::{tessellate_editor, tessellate_export, tessellate_primitives, Rgb};

#[test]
fn tessellate_alimentatore_has_strokes() {
    let libs = builtin_libraries();
    let mut ed = Editor::new(libs);
    ed.load_text(include_str!("../../fidorust-core/tests/Alimentatore.fcd"))
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
    doc.primitives
        .push(fidorust_core::Primitive::PcbPad(PcbPad {
            pos: fidorust_core::Point::new(380, 65),
            dx: 18,
            dy: 18,
            hole: 4,
            style: fidorust_core::primitive::PadStyle::RoundedRect,
            layer: fidorust_core::LayerId(0),
        }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let scene = tessellate_editor(&ed);
    assert!(scene.fills.len() >= 6);
    assert_eq!(scene.circles.len(), 0);
    assert_eq!(scene.pad_holes.len(), 1);
}

#[test]
fn tessellate_oval_pcb_pad_has_circular_hole() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives
        .push(fidorust_core::Primitive::PcbPad(PcbPad {
            pos: fidorust_core::Point::new(260, 125),
            dx: 40,
            dy: 30,
            hole: 25,
            style: fidorust_core::primitive::PadStyle::Oval,
            layer: fidorust_core::LayerId(0),
        }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.circles.len(), 1);
    assert!(scene.circles[0].inner > 0.01);
    assert!((scene.circles[0].rx - 20.0).abs() < 0.01);
    assert!((scene.circles[0].ry - 15.0).abs() < 0.01);
    assert_eq!(scene.fills.len(), 0);
    assert_eq!(scene.pad_holes.len(), 1);
    assert!((scene.pad_holes[0].r - 12.5).abs() < 0.01);
    let svg = fidorust_gpu::scene_to_thumb_svg(&scene, 40.0);
    assert_eq!(svg.matches("<polygon").count(), 0);
    assert!(
        svg.contains("fill-rule=\"evenodd\""),
        "oval pad thumbs must punch the hole with a native path: {svg}"
    );
}

#[test]
fn dense_oval_pad_thumb_stays_compact() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    for i in 0..256 {
        doc.primitives
            .push(fidorust_core::Primitive::PcbPad(PcbPad {
                pos: fidorust_core::Point::new(10 + (i % 16) * 20, 10 + (i / 16) * 20),
                dx: 12,
                dy: 12,
                hole: 6,
                style: fidorust_core::primitive::PadStyle::Oval,
                layer: fidorust_core::LayerId(0),
            }));
    }
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.circles.len(), 256);
    assert_eq!(scene.fills.len(), 0);
    let svg = fidorust_gpu::scene_to_thumb_svg(&scene, 40.0);
    assert_eq!(svg.matches("<polygon").count(), 0);
    assert_eq!(svg.matches("<path").count(), 256);
    assert!(
        svg.len() < 80_000,
        "PGA-like thumbs must not explode into tessellated SVG, got {} bytes",
        svg.len()
    );
}

#[test]
fn tessellate_pcb_track_is_filled_capsule() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives
        .push(fidorust_core::Primitive::PcbTrack(PcbTrack {
            a: fidorust_core::Point::new(80, 140),
            b: fidorust_core::Point::new(140, 140),
            width: 16,
            layer: fidorust_core::LayerId(0),
        }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
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
        doc.primitives.push(fidorust_core::Primitive::Line(Line {
            a: fidorust_core::Point::new(x, 0),
            b: fidorust_core::Point::new(x, 400),
            layer: fidorust_core::LayerId(0),
        }));
    }
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.lines.len(), 100);
}

#[test]
fn tessellate_ellipse_is_stroked_not_annulus() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives
        .push(fidorust_core::Primitive::Ellipse(Ellipse {
            a: fidorust_core::Point::new(0, 0),
            b: fidorust_core::Point::new(40, 20),
            filled: false,
            layer: fidorust_core::LayerId(0),
        }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.circles.len(), 1);
    assert!(scene.circles[0].stroke > 0.0);
    assert_eq!(scene.circles[0].inner, 0.0);
}

#[test]
fn draft_ellipse_previews_as_ellipse_not_line() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Ellipse);
    ed.doc_mut().snap = 1;
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
    let scene = fidorust_gpu::tessellate_primitives(&prims, &fidorust_core::LayerSet::default());
    let svg = fidorust_gpu::scene_to_thumb_svg(&scene, 40.0);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<line") || svg.contains("<ellipse") || svg.contains("<polygon"));
}

#[test]
fn macro_cursor_svg_has_hotspot() {
    let libs = builtin_libraries();
    let (_, def) = libs.lookup("080").expect("resistor");
    let prims = fidorust_gpu::tessellate_primitives(
        &fidorust_core::library::expand_component(
            def,
            fidorust_core::geom::Transform {
                origin: fidorust_core::COMPONENT_ORIGIN,
                rotations: 0,
                mirrored: false,
            },
            &libs,
            0,
        ),
        &fidorust_core::LayerSet::default(),
    );
    let cur = fidorust_gpu::scene_to_cursor_svg(&prims, fidorust_core::COMPONENT_ORIGIN);
    assert!(cur.w > 1.0 && cur.h > 1.0);
    assert!(
        cur.svg.contains("<line") || cur.svg.contains("<ellipse") || cur.svg.contains("<polygon")
    );
}

#[test]
fn export_svg_includes_ellipses_and_smart_holes() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives
        .push(fidorust_core::Primitive::Ellipse(Ellipse {
            a: fidorust_core::Point::new(0, 0),
            b: fidorust_core::Point::new(40, 20),
            filled: false,
            layer: fidorust_core::LayerId(0),
        }));
    doc.primitives
        .push(fidorust_core::Primitive::PcbPad(PcbPad {
            pos: fidorust_core::Point::new(80, 40),
            dx: 20,
            dy: 20,
            hole: 8,
            style: fidorust_core::primitive::PadStyle::Oval,
            layer: fidorust_core::LayerId(0),
        }));
    doc.primitives
        .push(fidorust_core::Primitive::PcbTrack(PcbTrack {
            a: fidorust_core::Point::new(60, 40),
            b: fidorust_core::Point::new(100, 40),
            width: 10,
            layer: fidorust_core::LayerId(0),
        }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let svg = fidorust_gpu::export_svg(
        &ed.doc().primitives,
        &ed.doc().layers,
        ed.libs(),
        4.0,
        ed.doc().stroke_width(),
    );
    assert!(svg.contains("<ellipse"), "round figures must be exported");
    assert!(
        svg.contains("fill=\"none\""),
        "stroked ellipses must not default-fill"
    );
    assert!(
        svg.contains("pad-hole") && svg.contains("mask"),
        "smart holes need a mask plus pad-hole markers"
    );
    assert!(
        !svg.contains(r#"<rect width="100%" height="100%" fill="white"/>"#),
        "export must not paint a white page background"
    );
    assert!(
        !svg.contains("viewBox=\"0 0 800"),
        "must not use the canvas viewport"
    );
    let polygons = svg.matches("<polygon").count();
    assert_eq!(
        polygons, 0,
        "pads/tracks must stay native shapes, not tessellated triangles, got {polygons} polygons"
    );
    assert!(
        svg.contains(r#"stroke-width="10""#) || svg.contains(r#"stroke-width="10.00""#),
        "pcb tracks must be stroked lines, not filled capsules: {svg}"
    );
}

#[test]
fn export_svg_keeps_beziers_and_text_native() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives
        .push(fidorust_core::Primitive::Bezier(fidorust_core::Bezier {
            p0: fidorust_core::Point::new(0, 0),
            p1: fidorust_core::Point::new(10, 20),
            p2: fidorust_core::Point::new(20, 20),
            p3: fidorust_core::Point::new(30, 0),
            layer: fidorust_core::LayerId(0),
        }));
    doc.primitives.push(fidorust_core::Primitive::Text(Text {
        pos: fidorust_core::Point::new(0, 40),
        sy: 10,
        sx: 6,
        angle: 0,
        style: 0,
        layer: fidorust_core::LayerId(0),
        font: DEFAULT_FONT.into(),
        text: "Vcc".into(),
        simple: false,
    }));
    doc.primitives
        .push(fidorust_core::Primitive::PcbPad(PcbPad {
            pos: fidorust_core::Point::new(50, 50),
            dx: 20,
            dy: 12,
            hole: 6,
            style: fidorust_core::primitive::PadStyle::RoundedRect,
            layer: fidorust_core::LayerId(0),
        }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let svg = fidorust_gpu::export_svg(
        &ed.doc().primitives,
        &ed.doc().layers,
        ed.libs(),
        2.0,
        ed.doc().stroke_width(),
    );
    assert!(svg.contains("<path d=\"M "), "beziers must be cubic paths");
    assert!(svg.contains(" C "), "beziers must keep control points");
    assert!(
        svg.contains("<text"),
        "labels must stay as text, not glyph triangles"
    );
    assert!(
        svg.contains(r#"font-family="&quot;Courier Prime&quot;,monospace""#),
        "default labels must quote Courier Prime so CSS does not split the name: {svg}"
    );
    assert!(svg.contains(">Vcc</text>"));
    assert!(
        svg.contains("rx="),
        "rounded pads must use native rounded rects: {svg}"
    );
    assert_eq!(svg.matches("<polygon").count(), 0);
    assert!(
        svg.matches("<line").count() < 4,
        "a cubic must not explode into draw segments: {svg}"
    );
}

#[test]
fn pending_macro_ghost_appears_at_hover() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Component);
    ed.set_pending_component(Some("080".into()));
    ed.set_pending_follow(true);
    ed.set_hover(Some(fidorust_core::Point::new(40, 40)));
    let scene = tessellate_editor(&ed);
    assert!(
        !scene.lines.is_empty() || !scene.circles.is_empty() || !scene.fills.is_empty(),
        "expected ghost geometry for pending macro"
    );
}

#[test]
fn pending_component_without_follow_has_no_ghost() {
    let mut ed = Editor::new(builtin_libraries());
    ed.set_tool(Tool::Component);
    ed.set_pending_component(Some("080".into()));
    ed.set_hover(Some(fidorust_core::Point::new(40, 40)));
    let scene = tessellate_editor(&ed);
    assert!(
        scene.lines.is_empty() && scene.circles.is_empty() && scene.fills.is_empty(),
        "selected-but-not-dragged component must not follow the cursor"
    );
}

#[test]
fn tessellate_text_uses_filled_glyphs() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidorust_core::Primitive::Text(Text {
        pos: fidorust_core::Point::new(0, 0),
        sy: 10,
        sx: 6,
        angle: 0,
        style: 0,
        layer: fidorust_core::LayerId(0),
        font: DEFAULT_FONT.into(),
        text: "Vcc".into(),
        simple: false,
    }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let scene = tessellate_editor(&ed);
    assert!(
        scene.fills.len() >= 27,
        "expected filled glyph triangles, got {}",
        scene.fills.len()
    );
    assert_eq!(scene.fills.len() % 3, 0);
}

#[test]
fn tessellate_unknown_font_falls_back_to_courier_prime() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidorust_core::Primitive::Text(Text {
        pos: fidorust_core::Point::new(0, 0),
        sy: 10,
        sx: 6,
        angle: 0,
        style: 0,
        layer: fidorust_core::LayerId(0),
        font: String::new(),
        text: "Vcc".into(),
        simple: false,
    }));
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let scene = tessellate_editor(&ed);
    assert!(
        scene.fills.len() >= 27,
        "empty font must still tessellate Courier Prime, got {}",
        scene.fills.len()
    );
}

#[test]
fn editing_text_hides_glyphs() {
    let mut doc = parse_document("[FIDOCAD]\n").unwrap();
    doc.primitives.push(fidorust_core::Primitive::Text(Text {
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
    let mut ed = Editor::new(builtin_libraries());
    ed.set_doc(doc);
    let before = tessellate_editor(&ed).fills.len();
    assert!(before > 0);
    ed.set_editing_text(Some(0));
    assert!(tessellate_editor(&ed).fills.is_empty());
}

#[test]
fn tessellate_layer_alpha() {
    let doc = parse_document("[FIDOCAD]\nLD 0 80 200 1 128 Copper\nLI 0 0 10 10\n").unwrap();
    let scene = tessellate_primitives(&doc.primitives, &doc.layers);
    assert_eq!(scene.lines.len(), 1);
    assert!((scene.lines[0].a - 128.0 / 255.0).abs() < 0.001);
}

#[test]
fn selection_handles_store_world_centers() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(fidorust_core::Primitive::Line(Line {
        a: Point::new(10, 20),
        b: Point::new(40, 20),
        layer: fidorust_core::LayerId(0),
    }));
    ed.set_selected(vec![0]);
    let scene = tessellate_editor(&ed);
    assert_eq!(scene.handles.len(), 2);
    let pts: Vec<(i32, i32)> = scene
        .handles
        .iter()
        .map(|h| (h.x.round() as i32, h.y.round() as i32))
        .collect();
    assert!(pts.contains(&(10, 20)));
    assert!(pts.contains(&(40, 20)));
}

#[test]
fn duplicate_drag_ghost_adds_preview_geometry() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.set_tool(Tool::Select);
    ed.pointer_down(Point::new(5, 0), (0.0, 0.0), false, false);
    let before = tessellate_editor(&ed);
    ed.set_move_duplicate(true);
    ed.pointer_move(Point::new(25, 0), (80.0, 0.0));
    let during = tessellate_editor(&ed);
    assert!(
        during.lines.len() > before.lines.len(),
        "expected ghost geometry during duplicate drag, before={} during={}",
        before.lines.len(),
        during.lines.len()
    );
}

#[test]
fn duplicate_selection_ghost_adds_preview_geometry() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(0, 0),
        b: Point::new(10, 0),
        layer: LayerId(0),
    }));
    ed.set_selected(vec![0]);
    ed.set_hover(Some(Point::new(40, 20)));
    let before = tessellate_editor(&ed);
    ed.duplicate_selection();
    let during = tessellate_editor(&ed);
    assert!(
        during.lines.len() > before.lines.len(),
        "expected ghost geometry after Duplicate, before={} during={}",
        before.lines.len(),
        during.lines.len()
    );
}

#[test]
fn ruler_overlay_visible_only_when_tool_active() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.set_tool(Tool::Ruler);
    ed.pointer_down(Point::new(0, 0), (0.0, 0.0), false, false);
    ed.pointer_move(Point::new(100, 0), (100.0, 0.0));
    ed.pointer_up(Point::new(100, 0));
    let shown = tessellate_editor(&ed);
    assert!(
        shown.lines.len() >= 3,
        "expected segment + end ticks, got {}",
        shown.lines.len()
    );
    assert!(
        !shown.fills.is_empty(),
        "expected glyph fills for the length label"
    );
    let accent = Rgb::ACCENT_LIGHT.rgba(1.0);
    assert!((shown.lines[0].r - accent[0]).abs() < 0.01);
    assert!((shown.lines[0].g - accent[1]).abs() < 0.01);
    assert!((shown.lines[0].b - accent[2]).abs() < 0.01);

    ed.set_tool(Tool::Select);
    let hidden = tessellate_editor(&ed);
    assert!(hidden.lines.is_empty());
    assert!(hidden.fills.is_empty());

    let exported = tessellate_export(&ed, &ed.doc().layers);
    assert!(exported.lines.is_empty());
    assert!(exported.fills.is_empty());
}

fn sample_o(layer: LayerId) -> Primitive {
    Primitive::Text(Text {
        pos: Point::new(0, 0),
        sy: 20,
        sx: 20,
        angle: 0,
        style: 0,
        layer,
        font: DEFAULT_FONT.into(),
        text: "O".into(),
        simple: false,
    })
}

fn uv_to_world(u: f32, v: f32) -> (f64, f64) {
    (u as f64 * 20.0, v as f64 * 20.0)
}

#[test]
fn text_glyph_hole_clicks_through_to_fill() {
    install_hit_hooks();
    let mut hole = None;
    let mut ink = None;
    for i in 0..21 {
        for j in 0..21 {
            let u = i as f32 / 20.0;
            let v = j as f32 / 20.0;
            if glyph_covers(DEFAULT_FONT, 'O', u, v) {
                ink = Some((u, v));
            } else if (0.25..0.75).contains(&u) && (0.25..0.75).contains(&v) {
                hole = Some((u, v));
            }
        }
    }
    let hole = hole.expect("O should have a counter");
    let ink = ink.expect("O should have ink");

    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().snap = 1;
    ed.doc_mut().insert(Primitive::Rect(Rect {
        a: Point::new(0, 0),
        b: Point::new(20, 20),
        filled: true,
        layer: LayerId(0),
    }));
    ed.doc_mut().insert(sample_o(LayerId(1)));

    let (hx, hy) = uv_to_world(hole.0, hole.1);
    ed.pointer_down_at(
        hx,
        hy,
        Point::new(hx as i32, hy as i32),
        (0.0, 0.0),
        false,
        false,
    );
    assert_eq!(
        ed.selected(),
        &[0],
        "hole of O should pick the fill underneath"
    );

    ed.set_selected(Vec::new());
    let (ix, iy) = uv_to_world(ink.0, ink.1);
    ed.pointer_down_at(
        ix,
        iy,
        Point::new(ix as i32, iy as i32),
        (0.0, 0.0),
        false,
        false,
    );
    assert_eq!(ed.selected(), &[1], "ink of O should pick the text");
}

#[test]
fn hover_lightens_layer_color() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Rect(Rect {
        a: Point::new(0, 0),
        b: Point::new(20, 20),
        filled: true,
        layer: LayerId(0),
    }));
    let base = tessellate_editor(&ed);
    assert!(!base.fills.is_empty());
    let expected = Rgb::mix_white(
        Rgb::from_rgba_u8(ed.doc().layers.color(LayerId(0))),
        Rgb::HOVER_LIGHTEN,
    );
    ed.pointer_move(Point::new(10, 10), (40.0, 40.0));
    let hovered = tessellate_editor(&ed);
    let h = &hovered.fills[0];
    assert!((h.r - expected[0]).abs() < 1e-5);
    assert!((h.g - expected[1]).abs() < 1e-5);
    assert!((h.b - expected[2]).abs() < 1e-5);

    ed.set_selected(vec![0]);
    let sel = tessellate_editor(&ed);
    let orange = Rgb::SELECTION.rgba(1.0);
    assert!((sel.fills[0].r - orange[0]).abs() < 1e-5);
}

#[test]
fn unresolved_component_is_invisible() {
    let mut ed = Editor::new(builtin_libraries());
    ed.load_text("[FIDOCAD]\nMC 50 50 0 0 Missing.CS11\n")
        .unwrap();
    let scene = tessellate_editor(&ed);
    assert!(
        scene.lines.is_empty() && scene.fills.is_empty() && scene.circles.is_empty(),
        "unresolved components must not draw, got lines={} fills={} circles={}",
        scene.lines.len(),
        scene.fills.len(),
        scene.circles.len()
    );
}
