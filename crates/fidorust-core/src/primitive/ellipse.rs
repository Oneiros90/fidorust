//! Axis-aligned ellipse.

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{map_ab, set_ab, Geometry, HitTest};
use crate::properties::{apply_filled, read_filled, PropField, PropFieldValue, PropSource};

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
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        let cx = (self.a.x + self.b.x) as f64 / 2.0;
        let cy = (self.a.y + self.b.y) as f64 / 2.0;
        let rx = ((self.a.x - self.b.x).abs() as f64 / 2.0).max(1.0);
        let ry = ((self.a.y - self.b.y).abs() as f64 / 2.0).max(1.0);
        let nx = (x - cx) / rx;
        let ny = (y - cy) / ry;
        let d = nx * nx + ny * ny;
        let tn = (tol2.sqrt() / rx.min(ry)).max(0.0);
        if self.filled {
            d <= (1.0 + tn) * (1.0 + tn)
        } else {
            let inner = (1.0 - tn).max(0.0);
            d >= inner * inner && d <= (1.0 + tn) * (1.0 + tn)
        }
    }

    fn paint_order(&self) -> u8 {
        2
    }
}

impl PropSource for Ellipse {
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
