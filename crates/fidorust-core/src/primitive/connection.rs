//! Electrical junction / connection dot.

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{set_pos, Geometry, HitTest};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub pos: Point,
    pub layer: LayerId,
}

impl Geometry for Connection {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        Aabb {
            min: Point::new(self.pos.x - 2, self.pos.y - 2),
            max: Point::new(self.pos.x + 2, self.pos.y + 2),
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

impl HitTest for Connection {
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        let r = crate::consts::CONNECTION_RADIUS + tol2.sqrt();
        self.pos.dist_sq_xy(x, y) <= r * r
    }

    fn opaque_at(&self, x: f64, y: f64, _stroke_w: f64) -> bool {
        self.body_hit(x, y, 0.0)
    }

    fn paint_order(&self) -> u8 {
        2
    }
}
