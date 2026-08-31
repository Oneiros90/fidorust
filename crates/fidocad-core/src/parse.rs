//! Line-oriented FidoCAD 0.96 parser. Unknown / FCJ / FJC / CV / CP lines are skipped.

use crate::document::Document;
use crate::geom::Point;
use crate::layers::LayerId;
use crate::library::{Library, LibrarySet, MacroDef};
use crate::primitive::{PadStyle, Primitive};
use encoding_rs::{UTF_8, WINDOWS_1252};
use thiserror::Error;

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

pub fn decode_str(text: &str) -> String {
    // Already a String/str from the caller; keep as-is.
    text.to_string()
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

fn tokens(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}

fn parse_i32(s: &str) -> Option<i32> {
    s.parse().ok()
}

fn layer_from(toks: &[&str], idx: usize) -> LayerId {
    toks.get(idx)
        .and_then(|s| parse_i32(s))
        .map(LayerId::from_i32)
        .unwrap_or(LayerId(0))
}

/// Text after `skip` whitespace-separated tokens (original ExtractString).
fn extract_string(line: &str, skip: usize) -> String {
    let bytes = line.as_bytes();
    let mut i = 0;
    let max = bytes.len();
    while i < max && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    for _ in 0..skip {
        while i < max && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        while i < max && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
    }
    if i >= max {
        return String::new();
    }
    let rest = &line[i..];
    rest.trim_end_matches(|c: char| (c as u32) < 32)
        .to_string()
}

fn font_from_token(tok: &str) -> String {
    if tok == "*" {
        "Courier New".into()
    } else {
        tok.replace("++", " ")
    }
}

pub fn parse_primitive_line(line: &str) -> Option<Primitive> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let op = line.get(0..2)?.to_ascii_uppercase();
    match op.as_str() {
        "LI" => {
            let t = tokens(&line[2..]);
            if t.len() < 4 {
                return None;
            }
            Some(Primitive::Line {
                a: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                b: Point::new(parse_i32(t[2])?, parse_i32(t[3])?),
                layer: layer_from(&t, 4),
            })
        }
        "SA" => {
            let t = tokens(&line[2..]);
            if t.len() < 2 {
                return None;
            }
            Some(Primitive::Connection {
                pos: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                layer: layer_from(&t, 2),
            })
        }
        "BE" => {
            let t = tokens(&line[2..]);
            if t.len() < 8 {
                return None;
            }
            Some(Primitive::Bezier {
                p0: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                p1: Point::new(parse_i32(t[2])?, parse_i32(t[3])?),
                p2: Point::new(parse_i32(t[4])?, parse_i32(t[5])?),
                p3: Point::new(parse_i32(t[6])?, parse_i32(t[7])?),
                layer: layer_from(&t, 8),
            })
        }
        "RP" | "RV" => {
            let t = tokens(&line[2..]);
            if t.len() < 4 {
                return None;
            }
            Some(Primitive::Rect {
                a: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                b: Point::new(parse_i32(t[2])?, parse_i32(t[3])?),
                filled: op == "RP",
                layer: layer_from(&t, 4),
            })
        }
        "EP" | "EV" => {
            let t = tokens(&line[2..]);
            if t.len() < 4 {
                return None;
            }
            Some(Primitive::Ellipse {
                a: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                b: Point::new(parse_i32(t[2])?, parse_i32(t[3])?),
                filled: op == "EP",
                layer: layer_from(&t, 4),
            })
        }
        "PP" | "PV" => {
            let t = tokens(&line[2..]);
            if t.len() < 4 {
                return None;
            }
            let nums: Vec<i32> = t.iter().filter_map(|s| parse_i32(s)).collect();
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
            pts.truncate(10);
            Some(Primitive::Poly {
                pts,
                filled: op == "PP",
                layer,
            })
        }
        "PL" => {
            let t = tokens(&line[2..]);
            if t.len() < 5 {
                return None;
            }
            Some(Primitive::PcbTrack {
                a: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                b: Point::new(parse_i32(t[2])?, parse_i32(t[3])?),
                width: parse_i32(t[4])?.max(1),
                layer: layer_from(&t, 5),
            })
        }
        "PA" => {
            let t = tokens(&line[2..]);
            if t.len() < 6 {
                return None;
            }
            Some(Primitive::PcbPad {
                pos: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                dx: parse_i32(t[2])?,
                dy: parse_i32(t[3])?,
                hole: parse_i32(t[4])?,
                style: PadStyle::from_i32(parse_i32(t[5])?),
                layer: layer_from(&t, 6),
            })
        }
        "MC" => {
            let t = tokens(&line[2..]);
            if t.len() < 5 {
                return None;
            }
            let name = extract_string(&line[2..], 4);
            if name.is_empty() {
                return None;
            }
            let standard = name.starts_with('~') || !name.contains('.');
            Some(Primitive::Macro {
                pos: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                rotations: (parse_i32(t[2])? as u8) % 4,
                mirrored: parse_i32(t[3])? != 0,
                name: name.trim_start_matches('~').to_string(),
                standard,
            })
        }
        "TE" => {
            let t = tokens(&line[2..]);
            if t.len() < 2 {
                return None;
            }
            Some(Primitive::Text {
                pos: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                sy: 5,
                sx: 3,
                angle: 0,
                style: 0,
                layer: LayerId(0),
                font: "Courier New".into(),
                text: extract_string(&line[2..], 2),
                simple: true,
            })
        }
        "TX" => {
            let t = tokens(&line[2..]);
            if t.len() < 7 {
                return None;
            }
            Some(Primitive::Text {
                pos: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                sy: parse_i32(t[2])?,
                sx: parse_i32(t[3])?,
                angle: parse_i32(t[4])?,
                style: t[5].parse().unwrap_or(0),
                layer: LayerId(0),
                font: font_from_token(t[6]),
                text: extract_string(&line[2..], 7),
                simple: false,
            })
        }
        "TY" => {
            let t = tokens(&line[2..]);
            if t.len() < 8 {
                return None;
            }
            Some(Primitive::Text {
                pos: Point::new(parse_i32(t[0])?, parse_i32(t[1])?),
                sy: parse_i32(t[2])?,
                sx: parse_i32(t[3])?,
                angle: parse_i32(t[4])?,
                style: t[5].parse().unwrap_or(0),
                layer: LayerId::from_i32(parse_i32(t[6])?),
                font: font_from_token(t[7]),
                text: extract_string(&line[2..], 8),
                simple: false,
            })
        }
        _ => None,
    }
}

fn find_header(text: &str) -> Option<(DocKind, String, usize)> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            let rest = &text[i + 1..];
            let rest_trim = rest.trim_start();
            let kind = if rest_trim.starts_with("FIDOCAD") {
                Some(DocKind::Document)
            } else if rest_trim.starts_with("FIDOLIB") {
                Some(DocKind::Library)
            } else if rest_trim.starts_with("MACROCAD") {
                Some(DocKind::Macro)
            } else {
                None
            };
            if let Some(kind) = kind {
                let after_kw = match kind {
                    DocKind::Document => rest_trim.strip_prefix("FIDOCAD")?,
                    DocKind::Library => rest_trim.strip_prefix("FIDOLIB")?,
                    DocKind::Macro => rest_trim.strip_prefix("MACROCAD")?,
                };
                let after_kw = after_kw.trim_start();
                let end = after_kw.find(']')?;
                let title = after_kw[..end].trim().to_string();
                let abs = (rest_trim.as_ptr() as usize).saturating_sub(text.as_ptr() as usize);
                let body_off = abs + rest_trim.len() - after_kw.len() + end + 1;
                return Some((kind, title, body_off.min(text.len())));
            }
        }
        i += 1;
    }
    None
}

#[derive(Clone, Copy)]
enum DocKind {
    Document,
    Library,
    Macro,
}

pub fn parse_document(text: &str) -> Result<Document, ParseError> {
    if text.trim().is_empty() {
        return Err(ParseError::Empty);
    }
    let mut doc = Document::default();
    let body = if let Some((kind, title, off)) = find_header(text) {
        if matches!(kind, DocKind::Library) {
            return Err(ParseError::BadLibrary);
        }
        doc.title = title;
        &text[off..]
    } else {
        // Header optional: parse all primitive lines (forum paste).
        text
    };
    let mut warnings = 0u32;
    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if skip_line(line) {
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
    doc.warnings = warnings;
    Ok(doc)
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
        file_stem: String::new(),
        standard: false,
        macros: Vec::new(),
    };
    let mut category = String::new();
    let mut current: Option<MacroDef> = None;

    let flush = |lib: &mut Library, current: &mut Option<MacroDef>| {
        if let Some(m) = current.take() {
            if !m.key.is_empty() {
                lib.macros.push(m);
            }
        }
    };

    for raw in text[off..].lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
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
            current = Some(MacroDef {
                key,
                name,
                category: category.clone(),
                primitives: Vec::new(),
            });
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
            set.add(lib);
        }
    }
    set
}

pub fn builtin_libraries() -> LibrarySet {
    let stdlib = decode_bytes(include_bytes!("../libraries/stdlib.fcl"));
    let pcb = decode_bytes(include_bytes!("../libraries/PCB.fcl"));
    let lib1 = decode_bytes(include_bytes!("../libraries/lib1.fcl"));
    parse_library_set(&[("stdlib", &stdlib), ("PCB", &pcb), ("lib1", &lib1)])
}

#[allow(dead_code)]
fn _keep_utf8() {
    let _ = UTF_8;
}
