//! Line-oriented FidoCAD 0.96 parser. Unknown / FCJ / FJC / CV / CP lines are skipped.

use crate::consts::{
    GRID_MAX, GRID_MIN, SNAP_MAX, SNAP_MIN, STROKE_HUNDREDTHS_MAX, STROKE_HUNDREDTHS_MIN,
};
use crate::document::{Document, ProjectSettings};
use crate::geom::Point;
use crate::layers::{LayerId, LayerInfo, LayerSet};
use crate::library::{ComponentDef, Library, LibraryKind, LibrarySet, PROJECT_STEM};
use crate::primitive::{
    Bezier, ComponentRef, Connection, Ellipse, Line, PadStyle, PcbPad, PcbTrack, Poly, Primitive,
    Rect, Text, DEFAULT_FONT, MAX_POLY_VERTICES,
};
use encoding_rs::WINDOWS_1252;
use thiserror::Error;

use super::tokens::{extract_string, font_from_token, TokenCursor};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("empty input")]
    Empty,
    #[error("invalid library header")]
    BadLibrary,
}

/// Decode bytes as UTF-8 if valid, otherwise CP-1252 (Windows FidoCAD).
pub fn decode_bytes(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    let (cow, _, _) = WINDOWS_1252.decode(bytes);
    cow.into_owned()
}

fn skip_line(line: &str) -> bool {
    let u = line.to_ascii_uppercase();
    u.starts_with("FCJ")
        || u.starts_with("FJC")
        || u.starts_with("CV ")
        || u.starts_with("CP ")
        || u.starts_with("CV\t")
        || u.starts_with("CP\t")
        || line.starts_with('[')
        || line.starts_with('{')
        || line.starts_with('*')
}

/// `LD <r> <g> <b> <visible> [a] <name…>` — `a` is optional (0–255, default 255).
pub fn parse_ld_line(line: &str) -> Option<LayerInfo> {
    let line = line.trim();
    if line.len() < 2 || !line[..2].eq_ignore_ascii_case("LD") {
        return None;
    }
    let rest = &line[2..];
    if !rest.is_empty() && !rest.as_bytes()[0].is_ascii_whitespace() {
        return None;
    }
    let t = TokenCursor::new(rest);
    if t.len() < 4 {
        return None;
    }
    let r = t.i32(0)?.clamp(0, 255) as u8;
    let g = t.i32(1)?.clamp(0, 255) as u8;
    let b = t.i32(2)?.clamp(0, 255) as u8;
    let show = t.i32(3)? != 0;
    let (a, name_skip) = match t.i32(4) {
        Some(v) if (0..=255).contains(&v) => (v as u8, 5),
        _ => (255, 4),
    };
    let name = extract_string(rest, name_skip);
    Some(LayerInfo {
        name,
        color: [r, g, b, a],
        show,
    })
}

/// `PS <gridX> <gridY> <snapX> <snapY> <showGrid> <snapEnable> <hideOrigin> <strokeHundredths> <filled>`
///
/// Trailing fields may be omitted (they keep [`ProjectSettings`] defaults).
pub fn parse_ps_line(line: &str) -> Option<ProjectSettings> {
    let line = line.trim();
    if line.len() < 2 || !line[..2].eq_ignore_ascii_case("PS") {
        return None;
    }
    let rest = &line[2..];
    if !rest.is_empty() && !rest.as_bytes()[0].is_ascii_whitespace() {
        return None;
    }
    let t = TokenCursor::new(rest);
    let mut s = ProjectSettings::default();
    if let Some(v) = t.i32(0) {
        s.grid = v.clamp(GRID_MIN, GRID_MAX);
    }
    if let Some(v) = t.i32(1) {
        s.grid_y = v.clamp(GRID_MIN, GRID_MAX);
    }
    if let Some(v) = t.i32(2) {
        s.snap = v.clamp(SNAP_MIN, SNAP_MAX);
    }
    if let Some(v) = t.i32(3) {
        s.snap_y = v.clamp(SNAP_MIN, SNAP_MAX);
    }
    if let Some(v) = t.i32(4) {
        s.show_grid = v != 0;
    }
    if let Some(v) = t.i32(5) {
        s.snap_enable = v != 0;
    }
    if let Some(v) = t.i32(6) {
        s.hide_component_origin = v != 0;
    }
    if let Some(v) = t.i32(7) {
        s.stroke_hundredths = v.clamp(STROKE_HUNDREDTHS_MIN, STROKE_HUNDREDTHS_MAX);
    }
    if let Some(v) = t.i32(8) {
        s.default_filled = v != 0;
    }
    Some(s)
}

fn apply_layers(doc: &mut Document, mut defined: Vec<LayerInfo>) {
    if !defined.is_empty() {
        for (i, layer) in defined.iter_mut().enumerate() {
            if layer.name.is_empty() {
                layer.name = format!("Layer {}", i + 1);
            }
        }
        doc.layers = LayerSet::from_vec(defined);
    } else {
        doc.layers = LayerSet::default();
        let max = doc
            .primitives
            .iter()
            .map(|p| p.layer().index())
            .max()
            .unwrap_or(0);
        doc.layers.ensure_len(max + 1);
    }
    for p in &mut doc.primitives {
        p.set_layer(doc.layers.clamp_id(p.layer()));
    }
}

/// `MC x y rot mir name [layer]` — a trailing integer assigns the instance to a
/// project layer. No trailing integer means classic FidoCAD (use definition layers).
fn mc_name_and_layer(rest: &str) -> (String, Option<LayerId>) {
    let rest = rest.trim();
    if rest.is_empty() {
        return (String::new(), None);
    }
    if let Some((name, last)) = rest.rsplit_once(char::is_whitespace) {
        if !last.is_empty() && last.bytes().all(|b| b.is_ascii_digit()) {
            if let Ok(n) = last.parse::<i32>() {
                return (name.trim_end().to_string(), Some(LayerId::from_i32(n)));
            }
        }
    }
    (rest.to_string(), None)
}

pub fn parse_primitive_line(line: &str) -> Option<Primitive> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let op = line.get(0..2)?.to_ascii_uppercase();
    let rest = &line[2..];
    let t = TokenCursor::new(rest);
    match op.as_str() {
        "LI" => {
            if t.len() < 4 {
                return None;
            }
            Some(Primitive::Line(Line {
                a: t.point(0)?,
                b: t.point(2)?,
                layer: t.layer_or_default(4),
            }))
        }
        "SA" => {
            if t.len() < 2 {
                return None;
            }
            Some(Primitive::Connection(Connection {
                pos: t.point(0)?,
                layer: t.layer_or_default(2),
            }))
        }
        "BE" => {
            if t.len() < 8 {
                return None;
            }
            Some(Primitive::Bezier(Bezier {
                p0: t.point(0)?,
                p1: t.point(2)?,
                p2: t.point(4)?,
                p3: t.point(6)?,
                layer: t.layer_or_default(8),
            }))
        }
        "RP" | "RV" => {
            if t.len() < 4 {
                return None;
            }
            Some(Primitive::Rect(Rect {
                a: t.point(0)?,
                b: t.point(2)?,
                filled: op == "RP",
                layer: t.layer_or_default(4),
            }))
        }
        "EP" | "EV" => {
            if t.len() < 4 {
                return None;
            }
            Some(Primitive::Ellipse(Ellipse {
                a: t.point(0)?,
                b: t.point(2)?,
                filled: op == "EP",
                layer: t.layer_or_default(4),
            }))
        }
        "PP" | "PV" => {
            if t.len() < 4 {
                return None;
            }
            let nums = t.nums();
            if nums.len() < 4 {
                return None;
            }
            let (coords, layer) = if nums.len() % 2 == 1 {
                let last = *nums.last().unwrap();
                (&nums[..nums.len() - 1], LayerId::from_i32(last))
            } else {
                (nums.as_slice(), LayerId(0))
            };
            let mut pts = Vec::new();
            for chunk in coords.chunks(2) {
                if chunk.len() == 2 {
                    pts.push(Point::new(chunk[0], chunk[1]));
                }
            }
            if pts.len() < 2 {
                return None;
            }
            pts.truncate(MAX_POLY_VERTICES);
            Some(Primitive::Poly(Poly {
                pts,
                filled: op == "PP",
                layer,
            }))
        }
        "PL" => {
            if t.len() < 5 {
                return None;
            }
            Some(Primitive::PcbTrack(PcbTrack {
                a: t.point(0)?,
                b: t.point(2)?,
                width: t.i32(4)?.max(1),
                layer: t.layer_or_default(5),
            }))
        }
        "PA" => {
            if t.len() < 6 {
                return None;
            }
            Some(Primitive::PcbPad(PcbPad {
                pos: t.point(0)?,
                dx: t.i32(2)?,
                dy: t.i32(3)?,
                hole: t.i32(4)?,
                style: PadStyle::from_i32(t.i32(5)?),
                layer: t.layer_or_default(6),
            }))
        }
        "MC" => {
            if t.len() < 5 {
                return None;
            }
            let rest = t.rest_string(4);
            if rest.is_empty() {
                return None;
            }
            let (name, layer_tok) = mc_name_and_layer(&rest);
            if name.is_empty() {
                return None;
            }
            let standard = name.starts_with('~') || !name.contains('.');
            let (layer, use_component_layers) = match layer_tok {
                Some(layer) => (layer, false),
                None => (LayerId(0), true),
            };
            Some(Primitive::Component(ComponentRef {
                pos: t.point(0)?,
                rotations: (t.i32(2)? as u8) % 4,
                mirrored: t.i32(3)? != 0,
                name: name.trim_start_matches('~').to_string(),
                standard,
                layer,
                use_component_layers,
            }))
        }
        "TE" => {
            if t.len() < 2 {
                return None;
            }
            Some(Primitive::Text(Text {
                pos: t.point(0)?,
                sy: 5,
                sx: 3,
                angle: 0,
                style: 0,
                layer: LayerId(0),
                font: DEFAULT_FONT.into(),
                text: t.rest_string(2),
                simple: true,
            }))
        }
        "TX" => {
            if t.len() < 7 {
                return None;
            }
            Some(Primitive::Text(Text {
                pos: t.point(0)?,
                sy: t.i32(2)?,
                sx: t.i32(3)?,
                angle: t.i32(4)?,
                style: t.get(5)?.parse().unwrap_or(0),
                layer: LayerId(0),
                font: font_from_token(t.get(6)?),
                text: t.rest_string(7),
                simple: false,
            }))
        }
        "TY" => {
            if t.len() < 8 {
                return None;
            }
            Some(Primitive::Text(Text {
                pos: t.point(0)?,
                sy: t.i32(2)?,
                sx: t.i32(3)?,
                angle: t.i32(4)?,
                style: t.get(5)?.parse().unwrap_or(0),
                layer: LayerId::from_i32(t.i32(6)?),
                font: font_from_token(t.get(7)?),
                text: t.rest_string(8),
                simple: false,
            }))
        }
        _ => None,
    }
}

fn find_header(text: &str) -> Option<(DocKind, String, usize)> {
    find_header_from(text, 0).map(|(kind, title, body_off, _)| (kind, title, body_off))
}

fn find_header_from(text: &str, from: usize) -> Option<(DocKind, String, usize, usize)> {
    let bytes = text.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            let rest = &text[i + 1..];
            let rest_trim = rest.trim_start();
            let kind = if rest_trim.starts_with("FIDOCAD") {
                Some(DocKind::Document)
            } else if rest_trim.starts_with("FIDOLIB") {
                Some(DocKind::Library)
            } else if rest_trim.starts_with("MACROCAD") {
                Some(DocKind::MacroCad)
            } else {
                None
            };
            if let Some(kind) = kind {
                let after_kw = match kind {
                    DocKind::Document => rest_trim.strip_prefix("FIDOCAD")?,
                    DocKind::Library => rest_trim.strip_prefix("FIDOLIB")?,
                    DocKind::MacroCad => rest_trim.strip_prefix("MACROCAD")?,
                };
                let after_kw = after_kw.trim_start();
                let end = after_kw.find(']')?;
                let title = after_kw[..end].trim().to_string();
                let rest_start = i + 1;
                let trim_pad = rest.len() - rest_trim.len();
                let rest_trim_start = rest_start + trim_pad;
                let kw_and_space = rest_trim.len() - after_kw.len();
                let body_off = rest_trim_start + kw_and_space + end + 1;
                return Some((kind, title, body_off.min(text.len()), i));
            }
        }
        i += 1;
    }
    None
}

fn is_section_header(line: &str) -> bool {
    let t = line.trim();
    if !t.starts_with('[') {
        return false;
    }
    let inner = t.trim_start_matches('[').trim_start().to_ascii_uppercase();
    inner.starts_with("FIDOLIB") || inner.starts_with("FIDOCAD") || inner.starts_with("MACROCAD")
}

#[derive(Clone, Copy)]
enum DocKind {
    Document,
    Library,
    MacroCad,
}

pub fn parse_document(text: &str) -> Result<Document, ParseError> {
    parse_document_inner(text).map(|(doc, _)| doc)
}

pub fn parse_document_with_project_library(
    text: &str,
) -> Result<(Document, Option<Library>), ParseError> {
    parse_document_inner(text)
}

fn parse_document_inner(text: &str) -> Result<(Document, Option<Library>), ParseError> {
    if text.trim().is_empty() {
        return Err(ParseError::Empty);
    }
    let mut doc = Document::default();
    let (body, trailing) = if let Some((kind, title, off, _)) = find_header_from(text, 0) {
        if matches!(kind, DocKind::Library) {
            return Err(ParseError::BadLibrary);
        }
        doc.title = title;
        if let Some((k, _, _, lib_start)) = find_header_from(text, off) {
            if matches!(k, DocKind::Library) {
                (&text[off..lib_start], Some(&text[lib_start..]))
            } else {
                (&text[off..], None)
            }
        } else {
            (&text[off..], None)
        }
    } else {
        (text, None)
    };
    let mut warnings = 0u32;
    let mut defined = Vec::new();
    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if is_section_header(line) {
            break;
        }
        if skip_line(line) {
            continue;
        }
        if let Some(info) = parse_ld_line(line) {
            defined.push(info);
            continue;
        }
        if let Some(settings) = parse_ps_line(line) {
            doc.apply_project_settings(settings);
            continue;
        }
        match parse_primitive_line(line) {
            Some(p) => doc.primitives.push(p),
            None => {
                if !line.starts_with('[') && !line.starts_with('{') {
                    warnings += 1;
                }
            }
        }
    }
    apply_layers(&mut doc, defined);
    doc.warnings = warnings;
    let project = trailing.and_then(|t| {
        let mut lib = parse_library(t).ok()?;
        lib.file_stem = PROJECT_STEM.into();
        lib.kind = LibraryKind::Project;
        lib.standard = false;
        Some(lib)
    });
    Ok((doc, project))
}

pub fn parse_library(text: &str) -> Result<Library, ParseError> {
    let Some((kind, title, off)) = find_header(text) else {
        return Err(ParseError::BadLibrary);
    };
    if !matches!(kind, DocKind::Library) {
        return Err(ParseError::BadLibrary);
    }
    let mut lib = Library {
        name: title,
        ..Library::default()
    };
    let mut category = String::new();
    let mut current: Option<ComponentDef> = None;

    let flush = |lib: &mut Library, current: &mut Option<ComponentDef>| {
        if let Some(m) = current.take() {
            if !m.key.is_empty() {
                lib.components.push(m);
            }
        }
    };

    for raw in text[off..].lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if is_section_header(line) {
            break;
        }
        if line.starts_with('{') && line.ends_with('}') {
            flush(&mut lib, &mut current);
            category = line.trim_matches(|c| c == '{' || c == '}').to_string();
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            flush(&mut lib, &mut current);
            let inner = line.trim_matches(|c| c == '[' || c == ']').trim();
            let (key, name) = match inner.split_once(char::is_whitespace) {
                Some((k, n)) => (k.to_string(), n.trim().to_string()),
                None => (inner.to_string(), inner.to_string()),
            };
            current = Some(ComponentDef {
                key,
                name,
                description: String::new(),
                category: category.clone(),
                primitives: Vec::new(),
            });
            continue;
        }
        if line.len() >= 2 && line[..2].eq_ignore_ascii_case("DS") {
            let rest = line[2..].trim_start();
            if let Some(m) = current.as_mut() {
                if m.description.is_empty() {
                    m.description = rest.to_string();
                } else {
                    m.description.push('\n');
                    m.description.push_str(rest);
                }
            }
            continue;
        }
        if let Some(prim) = parse_primitive_line(line) {
            if let Some(m) = current.as_mut() {
                m.primitives.push(prim);
            }
        }
    }
    flush(&mut lib, &mut current);
    Ok(lib)
}

pub fn parse_library_set(named: &[(&str, &str)]) -> LibrarySet {
    let mut set = LibrarySet::new();
    for (stem, text) in named {
        if let Ok(mut lib) = parse_library(text) {
            lib.file_stem = (*stem).to_string();
            lib.standard = matches!(*stem, "stdlib" | "PCB");
            lib.kind = match *stem {
                PROJECT_STEM => LibraryKind::Project,
                crate::library::LOCAL_STEM => LibraryKind::Local,
                _ => LibraryKind::Builtin,
            };
            set.add(lib);
        }
    }
    set
}

pub fn builtin_libraries() -> LibrarySet {
    let stdlib = decode_bytes(include_bytes!("../../libraries/stdlib.fcl"));
    let pcb = decode_bytes(include_bytes!("../../libraries/PCB.fcl"));
    let lib1 = decode_bytes(include_bytes!("../../libraries/lib1.fcl"));
    let mut set = parse_library_set(&[("stdlib", &stdlib), ("PCB", &pcb), ("lib1", &lib1)]);
    set.ensure_user_libraries();
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_allows_spaces_before_keyword() {
        let doc = parse_document("[  FIDOCAD  titled]\nLI 0 0 1 1\n").unwrap();
        assert_eq!(doc.title, "titled");
        assert_eq!(doc.primitives.len(), 1);
    }
}
