//! Polyline / filled polygon.

use serde::{Deserialize, Serialize};

use crate::geom::{dist_point_segment_sq, Aabb, Point};
use crate::layers::LayerId;

use super::traits::{Geometry, HitTest};

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
    fn body_hit(&self, pt: Point, tol2: f64) -> bool {
        if self.pts.len() < 2 {
            return false;
        }
        if self.filled {
            point_in_poly(pt, &self.pts)
        } else {
            self.pts
                .windows(2)
                .any(|w| dist_point_segment_sq(pt, w[0], w[1]) <= tol2)
                || dist_point_segment_sq(pt, *self.pts.last().unwrap(), self.pts[0]) <= tol2
        }
    }
}

fn point_in_poly(pt: Point, pts: &[Point]) -> bool {
    let mut inside = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let pi = pts[i];
        let pj = pts[j];
        if ((pi.y > pt.y) != (pj.y > pt.y))
            && (pt.x as f64)
                < (pj.x - pi.x) as f64 * (pt.y - pi.y) as f64 / (pj.y - pi.y) as f64 + pi.x as f64
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}
