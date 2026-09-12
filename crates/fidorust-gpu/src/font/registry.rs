//! Registered TTF faces and the bundled Courier Prime fallback.

use std::cell::RefCell;
use std::collections::HashMap;

use fidorust_core::primitive::DEFAULT_FONT;
use ttf_parser::{Face, Style};

pub(crate) const FONT_DATA: &[u8] = include_bytes!("../../fonts/CourierPrime-Regular.ttf");

struct RegisteredFont {
    display: String,
    data: Vec<u8>,
    face_index: u32,
    score: i32,
}

thread_local! {
    static FONTS: RefCell<HashMap<String, RegisteredFont>> = RefCell::new(HashMap::new());
}

pub(super) fn cache_key(font: &str) -> String {
    let t = font.trim();
    if t.is_empty() {
        DEFAULT_FONT.to_ascii_lowercase()
    } else {
        t.to_ascii_lowercase()
    }
}

fn regularity_score(face: &Face<'_>) -> i32 {
    let mut score = 0;
    if !face.is_italic() {
        score += 100;
    }
    if face.style() == Style::Normal {
        score += 50;
    }
    let w = face.weight().to_number() as i32;
    score -= (w - 400).abs();
    score
}

fn advance(face: &Face<'_>, ch: char) -> Option<u16> {
    face.glyph_index(ch).and_then(|g| face.glyph_hor_advance(g))
}

/// True if the face is tagged monospace or typical letters share the same advance.
pub fn face_is_mono(face: &Face<'_>) -> bool {
    if face.is_monospaced() {
        return true;
    }
    let samples = ['i', 'M', 'W', '.', '0'];
    let mut widths = samples.iter().filter_map(|&ch| advance(face, ch));
    let Some(first) = widths.next() else {
        return false;
    };
    first > 0 && widths.all(|w| w == first)
}

fn collection_len(data: &[u8]) -> u32 {
    ttf_parser::fonts_in_collection(data).unwrap_or(1)
}

pub(crate) fn best_mono_face(data: &[u8]) -> Option<(u32, i32)> {
    let n = collection_len(data);
    let mut best: Option<(u32, i32)> = None;
    for index in 0..n {
        let Ok(face) = Face::parse(data, index) else {
            continue;
        };
        if !face_is_mono(&face) {
            continue;
        }
        let score = regularity_score(&face);
        match best {
            Some((_, s)) if s >= score => {}
            _ => best = Some((index, score)),
        }
    }
    best
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn family_name(face: &Face<'_>) -> Option<String> {
    let mut fallback = None;
    for name in face.names() {
        if !name.is_unicode() {
            continue;
        }
        let Some(s) = name.to_string() else {
            continue;
        };
        if s.is_empty() {
            continue;
        }
        if name.name_id == ttf_parser::name_id::TYPOGRAPHIC_FAMILY {
            return Some(s);
        }
        if name.name_id == ttf_parser::name_id::FAMILY {
            fallback = Some(s);
        }
    }
    fallback
}

/// Register a TTF/OTF/TTC face. Keeps monospace families; Regular is preferred.
pub fn register_font(name: &str, data: &[u8]) -> bool {
    let name = name.trim();
    if name.is_empty() || data.is_empty() {
        return false;
    }
    let Some((face_index, score)) = best_mono_face(data) else {
        return false;
    };
    let key = name.to_ascii_lowercase();
    FONTS.with(|fonts| {
        let mut fonts = fonts.borrow_mut();
        if let Some(existing) = fonts.get(&key) {
            if existing.score >= score {
                return true;
            }
        }
        fonts.insert(
            key,
            RegisteredFont {
                display: name.to_string(),
                data: data.to_vec(),
                face_index,
                score,
            },
        );
        true
    })
}

/// Bundled Courier Prime TTF (SIL OFL).
pub fn bundled_font_data() -> &'static [u8] {
    FONT_DATA
}

/// TTF/OTF bytes for `name`. Courier Prime (including empty/`*`) is always
/// the bundled face; other families only if [`register_font`] succeeded.
pub fn font_file_bytes(name: &str) -> Vec<u8> {
    let key = cache_key(name);
    if key == DEFAULT_FONT.to_ascii_lowercase() {
        return FONT_DATA.to_vec();
    }
    FONTS.with(|fonts| {
        fonts
            .borrow()
            .get(&key)
            .map(|e| e.data.clone())
            .unwrap_or_default()
    })
}

/// Courier Prime first, then registered system families (no duplicates).
pub fn registered_families() -> Vec<String> {
    let mut names = vec![DEFAULT_FONT.to_string()];
    FONTS.with(|fonts| {
        let mut extra: Vec<String> = fonts
            .borrow()
            .values()
            .map(|f| f.display.clone())
            .filter(|n| !n.eq_ignore_ascii_case(DEFAULT_FONT))
            .collect();
        extra.sort_by_key(|a| a.to_ascii_lowercase());
        names.extend(extra);
    });
    names
}

pub(super) fn tessellate_for_key(key: &str, ch: char) -> Vec<[f32; 2]> {
    FONTS.with(|fonts| {
        let fonts = fonts.borrow();
        match fonts.get(key) {
            Some(e) => super::glyphs::tessellate_char(&e.data, e.face_index, ch),
            None => super::glyphs::tessellate_char(FONT_DATA, 0, ch),
        }
    })
}

#[cfg(test)]
pub(super) fn clear_fonts() {
    FONTS.with(|f| f.borrow_mut().clear());
}
