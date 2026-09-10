//! Axis-aligned ellipse.

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{map_ab, set_ab, Geometry, HitTest};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    pub a: Point,
    pub b: Point,
    pub filled: bool,
    pub layer: LayerId,
}

impl Geometry for Ellipse {
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

impl HitTest for Ellipse {
    fn body_hit(&self, pt: Point, _tol2: f64) -> bool {
        let cx = (self.a.x + self.b.x) as f64 / 2.0;
        let cy = (self.a.y + self.b.y) as f64 / 2.0;
        let rx = ((self.a.x - self.b.x).abs() as f64 / 2.0).max(1.0);
        let ry = ((self.a.y - self.b.y).abs() as f64 / 2.0).max(1.0);
        let nx = (pt.x as f64 - cx) / rx;
        let ny = (pt.y as f64 - cy) / ry;
        let d = nx * nx + ny * ny;
        if self.filled {
            d <= 1.05
        } else {
            (d - 1.0).abs() < 0.15
        }
    }
}
