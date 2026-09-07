//! Line segment.

use serde::{Deserialize, Serialize};

use crate::geom::{dist_point_segment_sq, Aabb, Point};
use crate::layers::LayerId;

use super::traits::{map_ab, set_ab, Geometry, HitTest};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub a: Point,
    pub b: Point,
    pub layer: LayerId,
}

impl Geometry for Line {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        Aabb::from_points([self.a, self.b])
    }
    fn control_points(&self) -> Vec<Point> {
        vec![self.a, self.b]
    }
    fn set_control_point(&mut self, index: usize, p: Point) {
        set_ab(&mut self.a, &mut self.b, index, p);
    }
    fn transform_points(&mut self, f: impl Fn(Point) -> Point) {
        map_ab(&mut self.a, &mut self.b, f);
    }
}

impl HitTest for Line {
    fn body_hit(&self, pt: Point, tol2: f64) -> bool {
        dist_point_segment_sq(pt, self.a, self.b) <= tol2
    }
}
