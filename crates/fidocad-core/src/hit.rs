//! Hit-testing against flattened primitives.

use crate::geom::{dist_point_segment_sq, bezier_point, Point};
use crate::layers::LayerSet;
use crate::library::LibrarySet;
use crate::primitive::Primitive;

pub const HIT_TOLERANCE: f64 = 4.0;

pub struct Hit {
    pub index: usize,
    pub handle: Option<usize>,
}

fn hit_prim(p: &Primitive, pt: Point, tol2: f64) -> Option<usize> {
    let handles = p.control_points();
    for (i, h) in handles.iter().enumerate() {
        if h.dist_sq(pt) as f64 <= tol2 * 4.0 {
            return Some(i);
        }
    }
    let body = match p {
        Primitive::Line { a, b, .. } | Primitive::PcbTrack { a, b, .. } => {
            dist_point_segment_sq(pt, *a, *b) <= tol2
        }
        Primitive::Rect { a, b, filled, .. } => {
            let minx = a.x.min(b.x);
            let maxx = a.x.max(b.x);
            let miny = a.y.min(b.y);
            let maxy = a.y.max(b.y);
            if *filled {
                pt.x >= minx && pt.x <= maxx && pt.y >= miny && pt.y <= maxy
            } else {
                let on_h = (pt.y - miny).abs() <= 3 || (pt.y - maxy).abs() <= 3;
                let on_v = (pt.x - minx).abs() <= 3 || (pt.x - maxx).abs() <= 3;
                (on_h && pt.x >= minx && pt.x <= maxx) || (on_v && pt.y >= miny && pt.y <= maxy)
            }
        }
        Primitive::Ellipse { a, b, filled, .. } => {
            let cx = (a.x + b.x) as f64 / 2.0;
            let cy = (a.y + b.y) as f64 / 2.0;
            let rx = ((a.x - b.x).abs() as f64 / 2.0).max(1.0);
            let ry = ((a.y - b.y).abs() as f64 / 2.0).max(1.0);
            let nx = (pt.x as f64 - cx) / rx;
            let ny = (pt.y as f64 - cy) / ry;
            let d = nx * nx + ny * ny;
            if *filled {
                d <= 1.05
            } else {
                (d - 1.0).abs() < 0.15
            }
        }
        Primitive::Poly { pts, filled, .. } => {
            if pts.len() < 2 {
                return None;
            }
            if *filled {
                point_in_poly(pt, pts)
            } else {
                pts.windows(2)
                    .any(|w| dist_point_segment_sq(pt, w[0], w[1]) <= tol2)
                    || dist_point_segment_sq(pt, *pts.last().unwrap(), pts[0]) <= tol2
            }
        }
        Primitive::Bezier { p0, p1, p2, p3, .. } => {
            let mut hit = false;
            let mut prev = p0.as_f32();
            for i in 1..=16 {
                let t = i as f32 / 16.0;
                let cur = bezier_point(*p0, *p1, *p2, *p3, t);
                let a = Point::new(prev.0 as i32, prev.1 as i32);
                let b = Point::new(cur.0 as i32, cur.1 as i32);
                if dist_point_segment_sq(pt, a, b) <= tol2 {
                    hit = true;
                    break;
                }
                prev = cur;
            }
            hit
        }
        Primitive::Connection { pos, .. } => pos.dist_sq(pt) as f64 <= 16.0,
        Primitive::PcbPad { pos, dx, dy, .. } => {
            let hx = dx / 2;
            let hy = dy / 2;
            (pt.x - pos.x).abs() <= hx && (pt.y - pos.y).abs() <= hy
        }
        Primitive::Text {
            pos,
            sy,
            sx,
            angle,
            style,
            text,
            ..
        } => text_body_hit(*pos, *sx, *sy, *angle, *style, text, pt),
        Primitive::Macro { pos, .. } => pos.dist_sq(pt) as f64 <= 64.0,
    };
    if body {
        Some(usize::MAX) // body, not a handle
    } else {
        None
    }
}

/// Glyph box used by the GPU tessellator: origin at `pos` (top-left of the em
/// box), `sx` per character, `sy` tall, then rotated by `angle` and optionally
/// mirrored (`style & 4`).
fn text_body_hit(
    pos: Point,
    sx: i32,
    sy: i32,
    angle: i32,
    style: u32,
    text: &str,
    pt: Point,
) -> bool {
    let n = text.chars().count().max(1) as f64;
    let w = sx.max(1) as f64 * n;
    let h = sy.max(1) as f64;
    let dx = (pt.x - pos.x) as f64;
    let dy = (pt.y - pos.y) as f64;
    let rad = (angle as f64).to_radians();
    let (sin, cos) = (rad.sin(), rad.cos());
    let mut lx = dx * cos + dy * sin;
    let ly = -dx * sin + dy * cos;
    if style & 4 != 0 {
        lx = -lx;
    }
    const PAD: f64 = 2.0;
    lx >= -PAD && lx <= w + PAD && ly >= -PAD && ly <= h + PAD
}

fn point_in_poly(pt: Point, pts: &[Point]) -> bool {
    let mut inside = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let pi = pts[i];
        let pj = pts[j];
        if ((pi.y > pt.y) != (pj.y > pt.y))
            && (pt.x as f64)
                < (pj.x - pi.x) as f64 * (pt.y - pi.y) as f64 / (pj.y - pi.y) as f64 + pi.x as f64
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn hit_test(
    prims: &[Primitive],
    libs: &LibrarySet,
    layers: &LayerSet,
    pt: Point,
    zoom: f32,
) -> Option<Hit> {
    let tol = (HIT_TOLERANCE / zoom.max(0.1) as f64).max(1.5);
    let tol2 = tol * tol;
    // Top-most first.
    for (index, p) in prims.iter().enumerate().rev() {
        let expanded = crate::library::expand_primitive(p, libs);
        for q in &expanded {
            if !layers.visible(q.layer()) && !matches!(p, Primitive::Macro { .. }) {
                continue;
            }
            if let Primitive::Macro { .. } = p {
                // Macros: hit if any child hits, or origin handle.
            }
            if let Some(h) = hit_prim(q, pt, tol2) {
                let handle = if expanded.len() == 1 && h != usize::MAX {
                    Some(h)
                } else {
                    None
                };
                return Some(Hit { index, handle });
            }
        }
        if let Primitive::Macro { pos, .. } = p {
            if pos.dist_sq(pt) as f64 <= tol2 * 4.0 {
                return Some(Hit {
                    index,
                    handle: Some(0),
                });
            }
        }
    }
    None
}

pub fn marquee_select(prims: &[Primitive], libs: &LibrarySet, a: Point, b: Point) -> Vec<usize> {
    let minx = a.x.min(b.x);
    let maxx = a.x.max(b.x);
    let miny = a.y.min(b.y);
    let maxy = a.y.max(b.y);
    let mut out = Vec::new();
    for (i, p) in prims.iter().enumerate() {
        let bb = {
            let mut boxx = crate::geom::Aabb::empty();
            for q in crate::library::expand_primitive(p, libs) {
                boxx.include_aabb(&q.aabb());
            }
            boxx
        };
        if bb.min.x >= minx && bb.max.x <= maxx && bb.min.y >= miny && bb.max.y <= maxy {
            out.push(i);
        }
    }
    out
}
