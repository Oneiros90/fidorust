//! Bundled IBM Plex Mono (SIL OFL) for schematic labels.

use std::cell::RefCell;
use std::collections::HashMap;

use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use ttf_parser::{Face, OutlineBuilder};

const FONT_DATA: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");

thread_local! {
    static GLYPHS: RefCell<HashMap<char, Vec<[f32; 2]>>> = RefCell::new(HashMap::new());
}

/// Triangle vertices in a unit cell: `x` in `[0, 1]` (advance), `y` in `[0, 1]`
/// (`0` = top of the em box). Descenders may exceed `1`.
pub fn glyph_triangles(ch: char) -> Vec<[f32; 2]> {
    if ch.is_whitespace() {
        return Vec::new();
    }
    GLYPHS.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(v) = cache.get(&ch) {
            return v.clone();
        }
        let tris = tessellate_char(ch);
        cache.insert(ch, tris.clone());
        tris
    })
}

struct Metrics {
    units: f32,
    ascent: f32,
    advance: f32,
}

fn face() -> Face<'static> {
    Face::parse(FONT_DATA, 0).expect("bundled IBM Plex Mono Regular")
}

fn metrics(face: &Face<'_>) -> Metrics {
    let units = f32::from(face.units_per_em());
    let ascent = f32::from(face.ascender());
    let gid = face.glyph_index('M').or_else(|| face.glyph_index('0'));
    let advance = gid
        .and_then(|g| face.glyph_hor_advance(g))
        .map(f32::from)
        .unwrap_or(units);
    Metrics {
        units,
        ascent,
        advance,
    }
}

struct LyonOutline {
    builder: lyon::path::Builder,
    started: bool,
}

impl OutlineBuilder for LyonOutline {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.started {
            self.builder.end(false);
            self.started = false;
        }
        self.builder.begin(point(x, y));
        self.started = true;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder.quadratic_bezier_to(point(x1, y1), point(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder
            .cubic_bezier_to(point(x1, y1), point(x2, y2), point(x, y));
    }

    fn close(&mut self) {
        if self.started {
            self.builder.close();
            self.started = false;
        }
    }
}

fn tessellate_char(ch: char) -> Vec<[f32; 2]> {
    let face = face();
    let m = metrics(&face);
    let gid = face
        .glyph_index(ch)
        .or_else(|| face.glyph_index('?'))
        .unwrap_or(ttf_parser::GlyphId(0));

    let mut outline = LyonOutline {
        builder: Path::builder(),
        started: false,
    };
    if face.outline_glyph(gid, &mut outline).is_none() {
        return Vec::new();
    }
    if outline.started {
        outline.builder.end(false);
    }
    let path = outline.builder.build();
    if path.iter().next().is_none() {
        return Vec::new();
    }

    let mut buffers: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let opts = FillOptions::tolerance((m.units * 0.012).max(0.5));
    let _ = tess.tessellate_path(
        &path,
        &opts,
        &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| {
            let p = v.position();
            [p.x, p.y]
        }),
    );

    let adv = m.advance.max(1.0);
    let mut out = Vec::with_capacity(buffers.indices.len());
    for tri in buffers.indices.chunks(3) {
        if tri.len() != 3 {
            continue;
        }
        for &i in tri {
            if let Some(&[x, y]) = buffers.vertices.get(i as usize) {
                out.push([x / adv, (m.ascent - y) / m.units]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_a_is_filled() {
        let tris = glyph_triangles('A');
        assert!(
            tris.len() >= 9,
            "expected tessellated triangles, got {}",
            tris.len()
        );
        assert_eq!(tris.len() % 3, 0);
    }

    #[test]
    fn space_has_no_geometry() {
        assert!(glyph_triangles(' ').is_empty());
    }
}
