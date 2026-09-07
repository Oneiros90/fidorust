//! PCB pad.

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::pad::PadStyle;
use super::traits::{set_pos, Geometry, HitTest};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcbPad {
    pub pos: Point,
    pub dx: i32,
    pub dy: i32,
    pub hole: i32,
    pub style: PadStyle,
    pub layer: LayerId,
}

impl Geometry for PcbPad {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        Aabb {
            min: Point::new(self.pos.x - self.dx / 2, self.pos.y - self.dy / 2),
            max: Point::new(self.pos.x + self.dx / 2, self.pos.y + self.dy / 2),
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

impl HitTest for PcbPad {
    fn body_hit(&self, pt: Point, _tol2: f64) -> bool {
        let hx = self.dx / 2;
        let hy = self.dy / 2;
        (pt.x - self.pos.x).abs() <= hx && (pt.y - self.pos.y).abs() <= hy
    }
}
