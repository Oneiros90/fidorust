//! Hit-testing against flattened primitives.

use crate::consts::HANDLE_TOL_SCALE;
use crate::geom::Point;
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
        if h.dist_sq(pt) as f64 <= tol2 * HANDLE_TOL_SCALE {
            return Some(i);
        }
    }
    if p.body_hit(pt, tol2) {
        Some(usize::MAX) // body, not a handle
    } else {
        None
    }
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
            if !layers.visible(q.layer()) && !p.is_macro() {
                continue;
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
        if let Primitive::Macro(m) = p {
            if m.pos.dist_sq(pt) as f64 <= tol2 * HANDLE_TOL_SCALE {
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
        let bb = crate::library::expanded_aabb(p, libs);
        if bb.min.x >= minx && bb.max.x <= maxx && bb.min.y >= miny && bb.max.y <= maxy {
            out.push(i);
        }
    }
    out
}
