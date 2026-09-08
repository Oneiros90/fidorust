//! Hit-testing against flattened primitives.

use crate::consts::HANDLE_RADIUS_PX;
use crate::geom::Point;
use crate::layers::LayerSet;
use crate::library::LibrarySet;
use crate::primitive::Primitive;

pub const HIT_TOLERANCE: f64 = 4.0;

pub struct Hit {
    pub index: usize,
    pub handle: Option<usize>,
}

/// World-space radius² matching a [`crate::consts::HANDLE_RADIUS_PX`] circle at `zoom`.
fn handle_r2_world(zoom: f32) -> f64 {
    let z = zoom.max(0.1) as f64;
    let r = HANDLE_RADIUS_PX as f64 / z;
    r * r
}

fn hit_prim(p: &Primitive, pt: Point, tol2: f64, handle_r2: f64) -> Option<usize> {
    let handles = p.control_points();
    for (i, h) in handles.iter().enumerate() {
        if h.dist_sq(pt) as f64 <= handle_r2 {
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
    let handle_r2 = handle_r2_world(zoom);
    // Top-most first.
    for (index, p) in prims.iter().enumerate().rev() {
        let expanded = crate::library::expand_primitive(p, libs);
        for q in &expanded {
            if !layers.visible(q.layer()) && !p.is_component() {
                continue;
            }
            if let Some(h) = hit_prim(q, pt, tol2, handle_r2) {
                let handle = if expanded.len() == 1 && h != usize::MAX {
                    Some(h)
                } else {
                    None
                };
                return Some(Hit { index, handle });
            }
        }
        if let Primitive::Component(m) = p {
            if m.pos.dist_sq(pt) as f64 <= handle_r2 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::LayerId;
    use crate::primitive::{Line, Primitive};

    fn line_prims() -> Vec<Primitive> {
        vec![Primitive::Line(Line {
            a: Point::new(0, 0),
            b: Point::new(100, 0),
            layer: LayerId(0),
        })]
    }

    #[test]
    fn handle_hit_is_screen_space() {
        let prims = line_prims();
        let libs = LibrarySet::default();
        let layers = LayerSet::default();
        // 8 LU above the origin handle: 4 px at zoom 0.5 (inside 6 px), 32 px at zoom 4.
        let pt = Point::new(0, 8);
        let lo = hit_test(&prims, &libs, &layers, pt, 0.5);
        assert_eq!(lo.and_then(|h| h.handle), Some(0));
        let hi = hit_test(&prims, &libs, &layers, pt, 4.0);
        assert!(hi.is_none());
    }

    #[test]
    fn handle_hit_at_center_survives_high_zoom() {
        let prims = line_prims();
        let libs = LibrarySet::default();
        let layers = LayerSet::default();
        let hit = hit_test(&prims, &libs, &layers, Point::new(0, 0), 40.0);
        assert_eq!(hit.and_then(|h| h.handle), Some(0));
    }
}
