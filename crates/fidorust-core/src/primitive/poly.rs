//! Polyline / filled polygon.

use serde::{Deserialize, Serialize};

use crate::geom::{dist_point_xy_segment_sq, Aabb, Point};
use crate::layers::LayerId;

use super::traits::{Geometry, HitTest};
use crate::properties::{apply_filled, read_filled, PropField, PropFieldValue, PropSource};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Poly {
    pub pts: Vec<Point>,
    pub filled: bool,
    pub layer: LayerId,
}

impl Geometry for Poly {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        Aabb::from_points(self.pts.iter().copied())
    }
    fn control_points(&self) -> Vec<Point> {
        self.pts.clone()
    }
    fn set_control_point(&mut self, index: usize, p: Point) {
        if let Some(slot) = self.pts.get_mut(index) {
            *slot = p;
        }
    }
    fn transform_points(&mut self, f: impl Fn(Point) -> Point) {
        for p in &mut self.pts {
            *p = f(*p);
        }
    }
}

impl HitTest for Poly {
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        if self.pts.len() < 2 {
            return false;
        }
        if self.filled {
            point_in_poly(x, y, &self.pts)
        } else {
            self.pts
                .windows(2)
                .any(|w| dist_point_xy_segment_sq(x, y, w[0], w[1]) <= tol2)
                || dist_point_xy_segment_sq(x, y, *self.pts.last().unwrap(), self.pts[0]) <= tol2
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

impl PropSource for Poly {
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

fn point_in_poly(x: f64, y: f64, pts: &[Point]) -> bool {
    let mut inside = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let pi = pts[i];
        let pj = pts[j];
        let (pix, piy) = (pi.x as f64, pi.y as f64);
        let (pjx, pjy) = (pj.x as f64, pj.y as f64);
        if ((piy > y) != (pjy > y)) && (x < (pjx - pix) * (y - piy) / (pjy - piy) + pix) {
            inside = !inside;
        }
        j = i;
    }
    inside
}
