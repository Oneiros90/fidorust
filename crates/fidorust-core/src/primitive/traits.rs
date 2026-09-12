//! Shared geometry / hit-test dispatch for graphic primitives.

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

pub trait Geometry {
    fn layer(&self) -> LayerId;
    fn set_layer(&mut self, layer: LayerId);
    fn aabb(&self) -> Aabb;
    fn control_points(&self) -> Vec<Point>;
    fn set_control_point(&mut self, index: usize, p: Point);
    fn transform_points(&mut self, f: impl Fn(Point) -> Point);
}

pub trait HitTest {
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool;

    /// Drawn ink at `(x, y)` — fill, stroke, or copper; not click-slop or holes.
    fn opaque_at(&self, x: f64, y: f64, stroke_w: f64) -> bool {
        let r = (stroke_w * 0.5).max(0.0);
        self.body_hit(x, y, r * r)
    }

    /// Draw batch within a layer. Must match GPU `draw_layer`: fills (0), then
    /// line instances (1), then circle instances (2). Oval pads and junctions
    /// are circles, so they sit on top of tracks / fills on the same layer.
    fn paint_order(&self) -> u8 {
        0
    }
}

pub(crate) fn set_ab(a: &mut Point, b: &mut Point, index: usize, p: Point) {
    match index {
        0 => *a = p,
        1 => *b = p,
        _ => {}
    }
}

pub(crate) fn map_ab(a: &mut Point, b: &mut Point, f: impl Fn(Point) -> Point) {
    *a = f(*a);
    *b = f(*b);
}

pub(crate) fn set_pos(pos: &mut Point, index: usize, p: Point) {
    if index == 0 {
        *pos = p;
    }
}
