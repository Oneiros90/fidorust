//! PCB pad.

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::pad::PadStyle;
use super::traits::{set_pos, Geometry, HitTest};
use crate::properties::{apply_layer, read_layer, PropField, PropFieldValue, PropSource};
use std::str::FromStr;

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
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        self.in_copper(x, y, tol2.sqrt())
    }

    fn opaque_at(&self, x: f64, y: f64, _stroke_w: f64) -> bool {
        self.in_copper(x, y, 0.0) && !self.in_hole(x, y)
    }

    fn paint_order(&self) -> u8 {
        match self.style {
            PadStyle::Oval => 2,
            PadStyle::Rectangular | PadStyle::RoundedRect => 0,
        }
    }
}

impl PcbPad {
    fn in_copper(&self, x: f64, y: f64, tol: f64) -> bool {
        let dx = x - self.pos.x as f64;
        let dy = y - self.pos.y as f64;
        let hx = self.dx as f64 / 2.0 + tol;
        let hy = self.dy as f64 / 2.0 + tol;
        match self.style {
            PadStyle::Oval => {
                let rx = hx.max(0.5);
                let ry = hy.max(0.5);
                let nx = dx / rx;
                let ny = dy / ry;
                nx * nx + ny * ny <= 1.0
            }
            PadStyle::Rectangular | PadStyle::RoundedRect => dx.abs() <= hx && dy.abs() <= hy,
        }
    }

    fn in_hole(&self, x: f64, y: f64) -> bool {
        let hr = self.hole as f64 / 2.0;
        hr > 0.0 && self.pos.dist_sq_xy(x, y) <= hr * hr
    }
}

impl PropSource for PcbPad {
    fn fields() -> &'static [PropField] {
        &[
            PropField::SizeX,
            PropField::SizeY,
            PropField::IntDiam,
            PropField::PadStyle,
        ]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        match field {
            PropField::SizeX => Some(PropFieldValue::Int { value: self.dx }),
            PropField::SizeY => Some(PropFieldValue::Int { value: self.dy }),
            PropField::IntDiam => Some(PropFieldValue::Int { value: self.hole }),
            PropField::PadStyle => Some(PropFieldValue::PadStyle {
                value: self.style.to_string(),
            }),
            _ => read_layer(self.layer, field),
        }
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        match (field, value) {
            (PropField::SizeX, PropFieldValue::Int { value: v }) => {
                self.dx = (*v).clamp(2, 100);
                true
            }
            (PropField::SizeY, PropFieldValue::Int { value: v }) => {
                self.dy = (*v).clamp(2, 100);
                true
            }
            (PropField::IntDiam, PropFieldValue::Int { value: v }) => {
                self.hole = (*v).clamp(2, 100);
                true
            }
            (PropField::PadStyle, PropFieldValue::PadStyle { value: v }) => {
                if let Ok(s) = PadStyle::from_str(v) {
                    self.style = s;
                    true
                } else {
                    false
                }
            }
            _ => apply_layer(&mut self.layer, field, value),
        }
    }
}
