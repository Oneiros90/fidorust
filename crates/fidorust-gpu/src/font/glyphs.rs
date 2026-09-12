//! Glyph outline tessellation and unit-cell coverage for hit-testing.

use std::cell::RefCell;
use std::collections::HashMap;

use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use ttf_parser::{Face, OutlineBuilder};

use super::registry::{cache_key, tessellate_for_key};

type GlyphCache = HashMap<(String, char), Vec<[f32; 2]>>;

thread_local! {
    static GLYPHS: RefCell<GlyphCache> = RefCell::new(HashMap::new());
}

/// Triangle vertices in a unit cell: `x` in `[0, 1]` (advance), `y` in `[0, 1]`
/// (`0` = top of the em box). Descenders may exceed `1`.
/// Unknown or empty `font` uses bundled Courier Prime.
pub fn glyph_triangles(font: &str, ch: char) -> Vec<[f32; 2]> {
    if ch.is_whitespace() {
        return Vec::new();
    }
    let key = cache_key(font);
    GLYPHS.with(|cache| {
        {
            let cache = cache.borrow();
            if let Some(v) = cache.get(&(key.clone(), ch)) {
                return v.clone();
            }
        }
        let tris = tessellate_for_key(&key, ch);
        cache.borrow_mut().insert((key, ch), tris.clone());
        tris
    })
}

fn point_in_tri(px: f32, py: f32, a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let v0x = c[0] - a[0];
    let v0y = c[1] - a[1];
    let v1x = b[0] - a[0];
    let v1y = b[1] - a[1];
    let v2x = px - a[0];
    let v2y = py - a[1];
    let dot00 = v0x * v0x + v0y * v0y;
    let dot01 = v0x * v1x + v0y * v1y;
    let dot02 = v0x * v2x + v0y * v2y;
    let dot11 = v1x * v1x + v1y * v1y;
    let dot12 = v1x * v2x + v1y * v2y;
    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < 1e-20 {
        return false;
    }
    let inv = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv;
    let v = (dot00 * dot12 - dot01 * dot02) * inv;
    u >= 0.0 && v >= 0.0 && u + v <= 1.0
}

/// Unit-cell coverage matching [`glyph_triangles`] (`u` along advance, `v` down the em box).
pub fn glyph_covers(font: &str, ch: char, u: f32, v: f32) -> bool {
    glyph_triangles(font, ch)
        .as_chunks::<3>()
        .0
        .iter()
        .any(|t| point_in_tri(u, v, t[0], t[1], t[2]))
}

/// Wire glyph coverage into core hit-testing (safe to call more than once).
///
/// GPU injects this hook at runtime so `fidorust-core` can hit-test text without
/// depending on tessellation. The inversion (gpu → core) is intentional.
pub fn install_hit_hooks() {
    fidorust_core::primitive::set_glyph_ink(glyph_covers);
}

struct Metrics {
    units: f32,
    ascent: f32,
    advance: f32,
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

pub(super) fn tessellate_char(data: &[u8], face_index: u32, ch: char) -> Vec<[f32; 2]> {
    let Ok(face) = Face::parse(data, face_index) else {
        return Vec::new();
    };
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
pub(super) fn clear_glyphs() {
    GLYPHS.with(|g| g.borrow_mut().clear());
}
