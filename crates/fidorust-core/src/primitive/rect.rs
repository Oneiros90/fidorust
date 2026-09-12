//! Axis-aligned rectangle.

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{map_ab, set_ab, Geometry, HitTest};
use crate::properties::{apply_filled, read_filled, PropField, PropFieldValue, PropSource};

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
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        let minx = self.a.x.min(self.b.x) as f64;
        let maxx = self.a.x.max(self.b.x) as f64;
        let miny = self.a.y.min(self.b.y) as f64;
        let maxy = self.a.y.max(self.b.y) as f64;
        let tol = tol2.sqrt();
        if self.filled {
            x >= minx - tol && x <= maxx + tol && y >= miny - tol && y <= maxy + tol
        } else {
            let on_h = (y - miny).abs() <= tol || (y - maxy).abs() <= tol;
            let on_v = (x - minx).abs() <= tol || (x - maxx).abs() <= tol;
            (on_h && x >= minx - tol && x <= maxx + tol)
                || (on_v && y >= miny - tol && y <= maxy + tol)
        }
    }

    fn paint_order(&self) -> u8 {
        if self.filled {
            0
        } else {
            1
        }
    }
}

impl PropSource for Rect {
    fn fields() -> &'static [PropField] {
        &[PropField::Filled]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        read_filled(self.filled, self.layer, field)
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        apply_filled(&mut self.filled, &mut self.layer, field, value)
    }
}
