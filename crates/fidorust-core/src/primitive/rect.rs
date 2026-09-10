//! Axis-aligned rectangle.

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{map_ab, set_ab, Geometry, HitTest};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub a: Point,
    pub b: Point,
    pub filled: bool,
    pub layer: LayerId,
}

impl Geometry for Rect {
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

impl HitTest for Rect {
    fn body_hit(&self, pt: Point, _tol2: f64) -> bool {
        let minx = self.a.x.min(self.b.x);
        let maxx = self.a.x.max(self.b.x);
        let miny = self.a.y.min(self.b.y);
        let maxy = self.a.y.max(self.b.y);
        if self.filled {
            pt.x >= minx && pt.x <= maxx && pt.y >= miny && pt.y <= maxy
        } else {
            let on_h = (pt.y - miny).abs() <= 3 || (pt.y - maxy).abs() <= 3;
            let on_v = (pt.x - minx).abs() <= 3 || (pt.x - maxx).abs() <= 3;
            (on_h && pt.x >= minx && pt.x <= maxx) || (on_v && pt.y >= miny && pt.y <= maxy)
        }
    }
}
