//! PCB copper track.

use serde::{Deserialize, Serialize};

use crate::geom::{dist_point_xy_segment_sq, Aabb, Point};
use crate::layers::LayerId;

use super::traits::{map_ab, set_ab, Geometry, HitTest};
use crate::properties::{apply_layer, read_layer, PropField, PropFieldValue, PropSource};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcbTrack {
    pub a: Point,
    pub b: Point,
    pub width: i32,
    pub layer: LayerId,
}

impl Geometry for PcbTrack {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        let pad = (self.width / 2).max(1);
        Aabb::from_points([self.a, self.b]).expand(pad)
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

impl HitTest for PcbTrack {
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        let half = (self.width as f64 / 2.0).max(0.0);
        let r = half + tol2.sqrt();
        dist_point_xy_segment_sq(x, y, self.a, self.b) <= r * r
    }

    fn opaque_at(&self, x: f64, y: f64, _stroke_w: f64) -> bool {
        self.body_hit(x, y, 0.0)
    }
}

impl PropSource for PcbTrack {
    fn fields() -> &'static [PropField] {
        &[PropField::Thickness]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        match field {
            PropField::Thickness => Some(PropFieldValue::Int { value: self.width }),
            _ => read_layer(self.layer, field),
        }
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        match (field, value) {
            (PropField::Thickness, PropFieldValue::Int { value: v }) => {
                self.width = (*v).clamp(1, 100);
                true
            }
            _ => apply_layer(&mut self.layer, field, value),
        }
    }
}
