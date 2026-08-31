//! Serialize a document back to FidoCAD 0.96 text (CRLF, omit layer 0).

use crate::document::{Document, SaveOptions};
use crate::layers::LayerId;
use crate::library::LibrarySet;
use crate::primitive::Primitive;

fn push_layer(out: &mut String, layer: LayerId) {
    if layer.0 != 0 {
        out.push(' ');
        out.push_str(&layer.0.to_string());
    }
    out.push_str("\r\n");
}

fn font_token(font: &str) -> String {
    if font.eq_ignore_ascii_case("Courier New") {
        "*".into()
    } else {
        font.replace(' ', "++")
    }
}

pub fn serialize_primitive(p: &Primitive) -> String {
    let mut s = String::new();
    match p {
        Primitive::Line { a, b, layer } => {
            s.push_str(&format!("LI {} {} {} {}", a.x, a.y, b.x, b.y));
            push_layer(&mut s, *layer);
        }
        Primitive::Rect { a, b, filled, layer } => {
            s.push_str(&format!(
                "{} {} {} {} {}",
                if *filled { "RP" } else { "RV" },
                a.x,
                a.y,
                b.x,
                b.y
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::Ellipse { a, b, filled, layer } => {
            s.push_str(&format!(
                "{} {} {} {} {}",
                if *filled { "EP" } else { "EV" },
                a.x,
                a.y,
                b.x,
                b.y
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::Poly { pts, filled, layer } => {
            s.push_str(if *filled { "PP" } else { "PV" });
            for pt in pts {
                s.push_str(&format!(" {} {}", pt.x, pt.y));
            }
            push_layer(&mut s, *layer);
        }
        Primitive::Bezier {
            p0,
            p1,
            p2,
            p3,
            layer,
        } => {
            s.push_str(&format!(
                "BE {} {} {} {} {} {} {} {}",
                p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::Connection { pos, layer } => {
            s.push_str(&format!("SA {} {}", pos.x, pos.y));
            push_layer(&mut s, *layer);
        }
        Primitive::PcbTrack {
            a,
            b,
            width,
            layer,
        } => {
            s.push_str(&format!(
                "PL {} {} {} {} {}",
                a.x, a.y, b.x, b.y, width
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::PcbPad {
            pos,
            dx,
            dy,
            hole,
            style,
            layer,
        } => {
            s.push_str(&format!(
                "PA {} {} {} {} {} {}",
                pos.x, pos.y, dx, dy, hole, *style as i32
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::Text {
            pos,
            sy,
            sx,
            angle,
            style,
            layer,
            font,
            text,
            simple,
        } => {
            if *simple {
                s.push_str(&format!("TE {} {} {}\r\n", pos.x, pos.y, text));
            } else {
                s.push_str(&format!(
                    "TY {} {} {} {} {} {} {} {} {}\r\n",
                    pos.x,
                    pos.y,
                    sy,
                    sx,
                    angle,
                    style,
                    layer.0,
                    font_token(font),
                    text
                ));
            }
        }
        Primitive::Macro {
            pos,
            rotations,
            mirrored,
            name,
            ..
        } => {
            s.push_str(&format!(
                "MC {} {} {} {} {}\r\n",
                pos.x,
                pos.y,
                rotations,
                if *mirrored { 1 } else { 0 },
                name
            ));
        }
    }
    s
}

fn is_standard_macro(name: &str, libs: Option<&LibrarySet>) -> bool {
    if let Some(libs) = libs {
        return libs.is_standard(name);
    }
    !name.contains('.')
}

fn expand_for_save(p: &Primitive, libs: Option<&LibrarySet>, split: bool) -> Vec<Primitive> {
    match p {
        Primitive::Macro { name, standard, .. } => {
            let should_split = split && !is_standard_macro(name, libs) && !standard;
            if should_split {
                if let Some(libs) = libs {
                    return p_expand(p, libs);
                }
            }
            vec![p.clone()]
        }
        _ => vec![p.clone()],
    }
}

fn p_expand(p: &Primitive, libs: &LibrarySet) -> Vec<Primitive> {
    crate::library::expand_primitive(p, libs)
}

pub fn serialize_document(doc: &Document, opts: SaveOptions, libs: Option<&LibrarySet>) -> String {
    let mut out = String::new();
    if doc.title.is_empty() {
        out.push_str("[FIDOCAD]\r\n");
    } else {
        out.push_str("[FIDOCAD ");
        out.push_str(&doc.title);
        out.push_str("]\r\n");
    }
    for p in &doc.primitives {
        for q in expand_for_save(p, libs, opts.split_nonstandard_macros) {
            out.push_str(&serialize_primitive(&q));
        }
    }
    out
}

pub fn serialize_clipboard(prims: &[Primitive]) -> String {
    let mut out = String::from("[FIDOCAD]\r\n");
    for p in prims {
        out.push_str(&serialize_primitive(p));
    }
    out
}
