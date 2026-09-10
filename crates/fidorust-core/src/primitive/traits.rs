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
    fn body_hit(&self, pt: Point, tol2: f64) -> bool;
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
