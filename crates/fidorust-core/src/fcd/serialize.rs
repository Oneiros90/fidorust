//! Serialize a document back to FidoCAD 0.96 text (CRLF, omit layer 0 except TY).

use crate::document::{Document, PrimitiveDefaults, Sheet};
use crate::layers::LayerInfo;
use crate::library::{
    explode_user_components_in_document, fold_user_components_in_document, Library, LibrarySet,
};
use crate::primitive::{
    Bezier, ComponentRef, Connection, Ellipse, Line, PcbBoard, PcbPad, PcbTrack, Poly, Primitive,
    Rect, Text,
};

use super::tokens::{font_token, plus_spaces, push_layer};

pub fn serialize_primitive(p: &Primitive) -> String {
    let mut s = String::new();
    match p {
        Primitive::Line(Line { a, b, layer }) => {
            s.push_str(&format!("LI {} {} {} {}", a.x, a.y, b.x, b.y));
            push_layer(&mut s, *layer);
        }
        Primitive::Rect(Rect {
            a,
            b,
            filled,
            layer,
        }) => {
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
        Primitive::PcbBoard(PcbBoard {
            a,
            b,
            filled,
            layer,
        }) => {
            s.push_str(&format!(
                "{} {} {} {} {}",
                if *filled { "BP" } else { "BV" },
                a.x,
                a.y,
                b.x,
                b.y
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::Ellipse(Ellipse {
            a,
            b,
            filled,
            layer,
        }) => {
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
        Primitive::Poly(Poly { pts, filled, layer }) => {
            s.push_str(if *filled { "PP" } else { "PV" });
            for pt in pts {
                s.push_str(&format!(" {} {}", pt.x, pt.y));
            }
            push_layer(&mut s, *layer);
        }
        Primitive::Bezier(Bezier {
            p0,
            p1,
            p2,
            p3,
            layer,
        }) => {
            s.push_str(&format!(
                "BE {} {} {} {} {} {} {} {}",
                p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::Connection(Connection { pos, layer }) => {
            s.push_str(&format!("SA {} {}", pos.x, pos.y));
            push_layer(&mut s, *layer);
        }
        Primitive::PcbTrack(PcbTrack { a, b, width, layer }) => {
            s.push_str(&format!("PL {} {} {} {} {}", a.x, a.y, b.x, b.y, width));
            push_layer(&mut s, *layer);
        }
        Primitive::PcbPad(PcbPad {
            pos,
            dx,
            dy,
            hole,
            style,
            layer,
        }) => {
            s.push_str(&format!(
                "PA {} {} {} {} {} {}",
                pos.x, pos.y, dx, dy, hole, *style as i32
            ));
            push_layer(&mut s, *layer);
        }
        Primitive::Text(Text {
            pos,
            sy,
            sx,
            angle,
            style,
            layer,
            font,
            text,
            simple,
        }) => {
            if *simple {
                s.push_str(&format!("TE {} {} {}\r\n", pos.x, pos.y, text));
            } else {
                // TY always writes the layer, including 0.
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
        Primitive::Component(ComponentRef {
            pos,
            rotations,
            mirrored,
            name,
            layer,
            use_component_layers,
            ..
        }) => {
            s.push_str(&format!(
                "MC {} {} {} {} {}",
                pos.x,
                pos.y,
                rotations,
                if *mirrored { 1 } else { 0 },
                name
            ));
            if *use_component_layers {
                s.push_str("\r\n");
            } else {
                s.push(' ');
                s.push_str(&layer.0.to_string());
                s.push_str("\r\n");
            }
        }
    }
    s
}

pub fn serialize_project_settings_for(doc: &Document, sheet: &Sheet) -> String {
    let vis = |on: bool| if on { 1 } else { 0 };
    format!(
        "PS {} {} {} {} {} {} {}\r\n",
        sheet.grid,
        sheet.grid_y,
        sheet.snap,
        sheet.snap_y,
        vis(sheet.show_grid),
        vis(sheet.snap_enable),
        vis(doc.hide_component_origin),
    )
}

pub fn serialize_primitive_defaults(defaults: &PrimitiveDefaults) -> String {
    let vis = |on: bool| if on { 1 } else { 0 };
    let mut out = String::new();
    out.push_str(&format!("PD LI {}\r\n", defaults.line_stroke_hundredths));
    out.push_str(&format!("PD BE {}\r\n", defaults.bezier_stroke_hundredths));
    out.push_str(&format!(
        "PD RV {} {}\r\n",
        defaults.rect.stroke_hundredths,
        vis(defaults.rect.filled)
    ));
    out.push_str(&format!(
        "PD BV {} {}\r\n",
        defaults.board.stroke_hundredths,
        vis(defaults.board.filled)
    ));
    out.push_str(&format!(
        "PD EV {} {}\r\n",
        defaults.ellipse.stroke_hundredths,
        vis(defaults.ellipse.filled)
    ));
    out.push_str(&format!(
        "PD PV {} {}\r\n",
        defaults.poly.stroke_hundredths,
        vis(defaults.poly.filled)
    ));
    out.push_str(&format!(
        "PD TY {} {} {} {}\r\n",
        defaults.text.sy,
        defaults.text.sx,
        defaults.text.style,
        font_token(&defaults.text.font)
    ));
    out.push_str(&format!("PD PL {}\r\n", defaults.track_width));
    out.push_str(&format!(
        "PD PA {} {} {} {}\r\n",
        defaults.pad.dx, defaults.pad.dy, defaults.pad.hole, defaults.pad.style as i32
    ));
    out
}

pub fn serialize_project_settings(doc: &Document) -> String {
    serialize_project_settings_for(doc, doc)
}

pub fn serialize_layer(info: &LayerInfo) -> String {
    let vis = if info.show { 1 } else { 0 };
    if info.color[3] == 255 {
        format!(
            "LD {} {} {} {} {}\r\n",
            info.color[0], info.color[1], info.color[2], vis, info.name
        )
    } else {
        format!(
            "LD {} {} {} {} {} {}\r\n",
            info.color[0], info.color[1], info.color[2], vis, info.color[3], info.name
        )
    }
}

pub fn serialize_document(doc: &Document, libs: Option<&LibrarySet>) -> String {
    serialize_document_body(doc, libs)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SaveLibraryPolicy {
    #[default]
    Keep,
    FoldIntoProject,
    ExplodeUser,
}

impl std::str::FromStr for SaveLibraryPolicy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "fold" | "project" => Self::FoldIntoProject,
            "explode" | "split" => Self::ExplodeUser,
            _ => Self::Keep,
        })
    }
}

pub fn serialize_document_with_policy(
    doc: &Document,
    libs: &LibrarySet,
    policy: SaveLibraryPolicy,
) -> String {
    match policy {
        SaveLibraryPolicy::Keep => serialize_document(doc, Some(libs)),
        SaveLibraryPolicy::FoldIntoProject => {
            let mut doc = doc.clone();
            let mut libs = libs.clone();
            fold_user_components_in_document(&mut doc, &mut libs);
            serialize_document(&doc, Some(&libs))
        }
        SaveLibraryPolicy::ExplodeUser => {
            let mut doc = doc.clone();
            let mut libs = libs.clone();
            explode_user_components_in_document(&mut doc, &mut libs);
            serialize_document(&doc, Some(&libs))
        }
    }
}

fn serialize_document_body(doc: &Document, libs: Option<&LibrarySet>) -> String {
    let mut out = String::new();
    if doc.title.is_empty() {
        out.push_str("[FIDOCAD]\r\n");
    } else {
        out.push_str("[FIDOCAD ");
        out.push_str(&doc.title);
        out.push_str("]\r\n");
    }
    for layer in doc.layers.iter() {
        out.push_str(&serialize_layer(layer));
    }
    for group in &doc.common_node_groups {
        let names: Vec<&str> = group
            .iter()
            .map(String::as_str)
            .filter(|k| !k.is_empty())
            .collect();
        if names.is_empty() {
            continue;
        }
        out.push_str("CN ");
        out.push_str(&names.join(" | "));
        out.push_str("\r\n");
    }
    out.push_str(&serialize_pad_links(doc));
    out.push_str(&serialize_primitive_defaults(&doc.defaults));
    let multi = doc.sheets.len() > 1;
    for sheet in &doc.sheets {
        if multi {
            out.push_str("[FIDOSHEET");
            if !sheet.name.is_empty() {
                out.push(' ');
                out.push_str(&sheet.name);
            }
            out.push_str("]\r\n");
        }
        out.push_str(&serialize_project_settings_for(doc, sheet));
        for p in &sheet.primitives {
            out.push_str(&serialize_primitive(p));
        }
    }
    out.push_str(&serialize_pcb_ext(doc));
    if let Some(libs) = libs {
        if let Some(project) = libs.project() {
            if !project.components.is_empty() {
                out.push_str(&serialize_library(project));
            }
        }
    }
    out
}

fn serialize_pad_links(doc: &Document) -> String {
    let mut out = String::new();
    for link in &doc.pad_links {
        out.push_str(&format!(
            "LK {} {} {} {} {} {} {}\r\n",
            plus_spaces(&link.pad_sheet),
            link.x,
            link.y,
            link.layer,
            plus_spaces(&link.dot_sheet),
            link.dx,
            link.dy
        ));
    }
    out
}

fn serialize_pcb_ext(doc: &Document) -> String {
    if doc.pcb_ext_lines.is_empty() {
        return String::new();
    }
    let mut out = String::from("[FIDOEXT pcb]\r\n");
    for line in &doc.pcb_ext_lines {
        out.push_str(line);
        out.push_str("\r\n");
    }
    out
}

pub fn serialize_library(lib: &Library) -> String {
    let mut out = String::new();
    let header = if lib.kind == crate::library::LibraryKind::Project {
        lib.file_stem.as_str()
    } else if !lib.name.is_empty() {
        lib.name.as_str()
    } else {
        lib.file_stem.as_str()
    };
    out.push_str("[FIDOLIB ");
    out.push_str(header);
    out.push_str("]\r\n");
    let mut last_cat: Option<&str> = None;
    for c in &lib.components {
        let cat = if c.category.is_empty() {
            None
        } else {
            Some(c.category.as_str())
        };
        if cat != last_cat {
            if let Some(cat) = cat {
                out.push('{');
                out.push_str(cat);
                out.push_str("}\r\n");
            }
            last_cat = cat;
        }
        out.push('[');
        out.push_str(&c.key);
        out.push(' ');
        out.push_str(&c.name);
        out.push_str("]\r\n");
        for p in &c.primitives {
            out.push_str(&serialize_primitive(p));
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
