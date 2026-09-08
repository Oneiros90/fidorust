//! Unexpanded component instance (FidoCAD `MC` line).

use serde::{Deserialize, Serialize};

use crate::consts::COMPONENT_HIT_R2;
use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{set_pos, Geometry, HitTest};

/// Unexpanded component instance. Body is expanded at draw/hit time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentRef {
    pub pos: Point,
    pub rotations: u8,
    pub mirrored: bool,
    pub name: String,
    pub standard: bool,
}

impl Geometry for ComponentRef {
    fn layer(&self) -> LayerId {
        LayerId(0)
    }
    fn set_layer(&mut self, _layer: LayerId) {}
    fn aabb(&self) -> Aabb {
        Aabb {
            min: Point::new(self.pos.x - 10, self.pos.y - 10),
            max: Point::new(self.pos.x + 10, self.pos.y + 10),
        }
    }
    fn control_points(&self) -> Vec<Point> {
        vec![self.pos]
    }
    fn set_control_point(&mut self, index: usize, p: Point) {
        set_pos(&mut self.pos, index, p);
    }
    fn transform_points(&mut self, f: impl Fn(Point) -> Point) {
        self.pos = f(self.pos);
    }
}

impl HitTest for ComponentRef {
    fn body_hit(&self, pt: Point, _tol2: f64) -> bool {
        self.pos.dist_sq(pt) as f64 <= COMPONENT_HIT_R2
    }
}
