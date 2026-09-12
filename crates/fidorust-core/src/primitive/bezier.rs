//! Cubic Bézier curve.

use serde::{Deserialize, Serialize};

use crate::geom::{bezier_point, dist_point_xy_segment_sq, Aabb, Point};
use crate::layers::LayerId;

use super::traits::{Geometry, HitTest};
use super::BEZIER_SEGMENTS_HIT;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bezier {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub layer: LayerId,
}

impl Geometry for Bezier {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        let mut bb = Aabb::from_points([self.p0, self.p3]);
        for i in 1..BEZIER_SEGMENTS_HIT {
            let t = i as f32 / BEZIER_SEGMENTS_HIT as f32;
            let (x, y) = bezier_point(self.p0, self.p1, self.p2, self.p3, t);
            bb.include(Point::new(x.round() as i32, y.round() as i32));
        }
        bb
    }
    fn control_points(&self) -> Vec<Point> {
        vec![self.p0, self.p1, self.p2, self.p3]
    }
    fn set_control_point(&mut self, index: usize, p: Point) {
        match index {
            0 => self.p0 = p,
            1 => self.p1 = p,
            2 => self.p2 = p,
            3 => self.p3 = p,
            _ => {}
        }
    }
    fn transform_points(&mut self, f: impl Fn(Point) -> Point) {
        self.p0 = f(self.p0);
        self.p1 = f(self.p1);
        self.p2 = f(self.p2);
        self.p3 = f(self.p3);
    }
}

impl HitTest for Bezier {
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        let mut prev = self.p0.as_f32();
        for i in 1..=BEZIER_SEGMENTS_HIT {
            let t = i as f32 / BEZIER_SEGMENTS_HIT as f32;
            let cur = bezier_point(self.p0, self.p1, self.p2, self.p3, t);
            let a = Point::new(prev.0.round() as i32, prev.1.round() as i32);
            let b = Point::new(cur.0.round() as i32, cur.1.round() as i32);
            if dist_point_xy_segment_sq(x, y, a, b) <= tol2 {
                return true;
            }
            prev = cur;
        }
        false
    }

    fn paint_order(&self) -> u8 {
        1
    }
}
