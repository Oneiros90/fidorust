//! Integer logical coordinates. One unit = 127 µm (200 dpi), matching FidoCAD 0.96.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn rotate90_cw(self, origin: Self) -> Self {
        let d = self.sub(origin);
        Self {
            x: origin.x + d.y,
            y: origin.y - d.x,
        }
    }

    pub fn rotate90_ccw(self, origin: Self) -> Self {
        let d = self.sub(origin);
        Self {
            x: origin.x - d.y,
            y: origin.y + d.x,
        }
    }

    /// Mirror across a vertical axis at `axis_x` (original `Swap`).
    pub fn mirror_vertical(self, axis_x: i32) -> Self {
        Self {
            x: 2 * axis_x - self.x,
            y: self.y,
        }
    }

    pub fn dist_sq(self, other: Self) -> i64 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        dx * dx + dy * dy
    }

    pub fn as_f32(self) -> (f32, f32) {
        (self.x as f32, self.y as f32)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Point,
    pub max: Point,
}

impl Aabb {
    pub fn empty() -> Self {
        Self {
            min: Point::new(i32::MAX, i32::MAX),
            max: Point::new(i32::MIN, i32::MIN),
        }
    }

    pub fn from_points(pts: impl IntoIterator<Item = Point>) -> Self {
        let mut a = Self::empty();
        for p in pts {
            a.include(p);
        }
        a
    }

    pub fn include(&mut self, p: Point) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn include_aabb(&mut self, other: &Aabb) {
        if other.is_empty() {
            return;
        }
        self.include(other.min);
        self.include(other.max);
    }

    pub fn expand(&self, pad: i32) -> Self {
        Self {
            min: Point::new(self.min.x - pad, self.min.y - pad),
            max: Point::new(self.max.x + pad, self.max.y + pad),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn width(&self) -> i32 {
        (self.max.x - self.min.x).max(0)
    }

    pub fn height(&self) -> i32 {
        (self.max.y - self.min.y).max(0)
    }
}

/// Affine-ish drawing transform used when expanding macros.
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub origin: Point,
    pub rotations: u8,
    pub mirrored: bool,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            origin: Point::new(0, 0),
            rotations: 0,
            mirrored: false,
        }
    }

    pub fn apply(&self, p: Point, macro_origin: Point) -> Point {
        let mut q = p.add(self.origin.sub(macro_origin));
        for _ in 0..self.rotations {
            q = q.rotate90_cw(self.origin);
        }
        if self.mirrored {
            q = q.mirror_vertical(self.origin.x);
        }
        q
    }
}

pub fn snap(value: i32, grid: i32) -> i32 {
    if grid <= 0 {
        return value;
    }
    let g = grid as i64;
    let v = value as i64;
    let half = g / 2;
    let q = if v >= 0 {
        (v + half) / g
    } else {
        (v - half) / g
    };
    (q * g) as i32
}

pub fn dist_point_segment_sq(p: Point, a: Point, b: Point) -> f64 {
    let (px, py) = (p.x as f64, p.y as f64);
    let (ax, ay) = (a.x as f64, a.y as f64);
    let (bx, by) = (b.x as f64, b.y as f64);
    let dx = bx - ax;
    let dy = by - ay;
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-9 {
        let ex = px - ax;
        let ey = py - ay;
        return ex * ex + ey * ey;
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / len2;
    let t = t.clamp(0.0, 1.0);
    let qx = ax + t * dx;
    let qy = ay + t * dy;
    let ex = px - qx;
    let ey = py - qy;
    ex * ex + ey * ey
}

/// Cubic Bézier sample (original FidoCAD formula).
pub fn bezier_point(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    let b03 = u * u * u;
    let b13 = 3.0 * t * u * u;
    let b23 = 3.0 * t * t * u;
    let b33 = t * t * t;
    (
        p0.x as f32 * b03 + p1.x as f32 * b13 + p2.x as f32 * b23 + p3.x as f32 * b33,
        p0.y as f32 * b03 + p1.y as f32 * b13 + p2.y as f32 * b23 + p3.y as f32 * b33,
    )
}
