//! Token helpers for FidoCAD line-oriented opcodes.

use crate::geom::Point;
use crate::layers::LayerId;
use crate::primitive::DEFAULT_FONT;

pub struct TokenCursor<'a> {
    toks: Vec<&'a str>,
    line: &'a str,
}

impl<'a> TokenCursor<'a> {
    pub fn new(line: &'a str) -> Self {
        Self {
            toks: line.split_whitespace().collect(),
            line,
        }
    }

    pub fn len(&self) -> usize {
        self.toks.len()
    }

    pub fn get(&self, i: usize) -> Option<&'a str> {
        self.toks.get(i).copied()
    }

    pub fn i32(&self, i: usize) -> Option<i32> {
        self.get(i)?.parse().ok()
    }

    pub fn point(&self, i: usize) -> Option<Point> {
        Some(Point::new(self.i32(i)?, self.i32(i + 1)?))
    }

    pub fn layer_or_default(&self, i: usize) -> LayerId {
        self.i32(i).map(LayerId::from_i32).unwrap_or(LayerId(0))
    }

    pub fn rest_string(&self, skip: usize) -> String {
        extract_string(self.line, skip)
    }

    pub fn nums(&self) -> Vec<i32> {
        self.toks.iter().filter_map(|s| s.parse().ok()).collect()
    }
}

/// Text after `skip` whitespace-separated tokens (original ExtractString).
pub fn extract_string(line: &str, skip: usize) -> String {
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
    rest.trim_end_matches(|c: char| (c as u32) < 32).to_string()
}

pub fn font_from_token(tok: &str) -> String {
    if tok == "*" {
        DEFAULT_FONT.into()
    } else {
        tok.replace("++", " ")
    }
}

pub fn font_token(font: &str) -> String {
    if font.eq_ignore_ascii_case(DEFAULT_FONT) {
        "*".into()
    } else {
        font.replace(' ', "++")
    }
}

pub fn push_layer(out: &mut String, layer: LayerId) {
    if layer.0 != 0 {
        out.push(' ');
        out.push_str(&layer.0.to_string());
    }
    out.push_str("\r\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_is_courier_prime() {
        assert_eq!(font_from_token("*"), "Courier Prime");
        assert_eq!(font_token("Courier Prime"), "*");
        assert_eq!(font_token("courier prime"), "*");
    }

    #[test]
    fn spaces_roundtrip_via_plus() {
        assert_eq!(font_from_token("Lucida++Console"), "Lucida Console");
        assert_eq!(font_token("Lucida Console"), "Lucida++Console");
    }
}
