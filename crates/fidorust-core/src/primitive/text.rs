//! Text primitive and style flags.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{set_pos, Geometry, HitTest};

pub const STYLE_BOLD: u32 = 1;
pub const STYLE_ITALIC: u32 = 2;
pub const STYLE_MIRRORED: u32 = 4;
pub const STYLE_UNDERLINE: u32 = 8;
pub const ITALIC_SHEAR: f32 = 0.22;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextStyle(pub u32);

impl TextStyle {
    pub fn is_bold(self) -> bool {
        self.0 & STYLE_BOLD != 0
    }
    pub fn is_italic(self) -> bool {
        self.0 & STYLE_ITALIC != 0
    }
    pub fn is_mirrored(self) -> bool {
        self.0 & STYLE_MIRRORED != 0
    }
    pub fn is_underlined(self) -> bool {
        self.0 & STYLE_UNDERLINE != 0
    }
    pub fn set(&mut self, mask: u32, on: bool) {
        if on {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
}

/// Raw glyph-box metrics. Consumers apply their own clamps (AABB vs hit vs tessellate).
pub struct TextLayout {
    pub w: f32,
    pub h: f32,
    pub pad: f32,
    pub x0: f32,
    pub x1: f32,
    pub sin: f32,
    pub cos: f32,
}

impl TextLayout {
    pub fn from_params(sx: i32, sy: i32, angle: i32, style: u32, text: &str) -> Self {
        let n = text.chars().count().max(1) as f32;
        let w = sx.max(1) as f32 * n;
        let h = sy.max(1) as f32;
        let pad = if style & STYLE_ITALIC != 0 {
            w * ITALIC_SHEAR
        } else {
            0.0
        };
        let (x0, x1) = if style & STYLE_MIRRORED != 0 {
            (-w - pad, 0.0)
        } else {
            (0.0, w + pad)
        };
        let rad = (angle as f32).to_radians();
        let (sin, cos) = rad.sin_cos();
        Self {
            w,
            h,
            pad,
            x0,
            x1,
            sin,
            cos,
        }
    }

    /// Local text offsets (x right, y down) → world delta. Positive angle is CCW on Y-down.
    pub fn map_offset(lx: f32, ly: f32, sin: f32, cos: f32) -> (f32, f32) {
        (lx * cos + ly * sin, -lx * sin + ly * cos)
    }

    pub fn map(&self, lx: f32, ly: f32) -> (f32, f32) {
        Self::map_offset(lx, ly, self.sin, self.cos)
    }

    /// Inverse of [`Self::map_offset`].
    pub fn unmap_offset(dx: f64, dy: f64, sin: f64, cos: f64) -> (f64, f64) {
        (dx * cos - dy * sin, dx * sin + dy * cos)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub pos: Point,
    pub sy: i32,
    pub sx: i32,
    pub angle: i32,
    pub style: u32,
    pub layer: LayerId,
    pub font: String,
    pub text: String,
    /// True if this came from a `TE` command (round-trip as TE if desired).
    pub simple: bool,
}

impl Geometry for Text {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        let layout = TextLayout::from_params(self.sx, self.sy, self.angle, self.style, &self.text);
        let mut bb = Aabb::empty();
        for (lx, ly) in [
            (layout.x0, 0.0),
            (layout.x1, 0.0),
            (layout.x0, layout.h),
            (layout.x1, layout.h),
        ] {
            let (dx, dy) = layout.map(lx, ly);
            bb.include(Point::new(
                (self.pos.x as f32 + dx).round() as i32,
                (self.pos.y as f32 + dy).round() as i32,
            ));
        }
        bb
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

impl HitTest for Text {
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        let Some((lx, ly, w, h)) = self.local_hit(x, y) else {
            return false;
        };
        let pad = crate::consts::TEXT_HIT_PAD.max(tol2.sqrt());
        lx >= -pad && lx <= w + pad && ly >= -pad && ly <= h + pad
    }

    fn opaque_at(&self, x: f64, y: f64, _stroke_w: f64) -> bool {
        self.glyph_ink_at(x, y)
    }
}

impl Text {
    fn local_hit(&self, x: f64, y: f64) -> Option<(f64, f64, f64, f64)> {
        let n = self.text.chars().count().max(1) as f64;
        let w = self.sx.max(1) as f64 * n;
        let h = self.sy.max(1) as f64;
        let dx = x - self.pos.x as f64;
        let dy = y - self.pos.y as f64;
        let rad = (self.angle as f64).to_radians();
        let (sin, cos) = (rad.sin(), rad.cos());
        let (mut lx, ly) = TextLayout::unmap_offset(dx, dy, sin, cos);
        if self.style & STYLE_MIRRORED != 0 {
            lx = -lx;
        }
        Some((lx, ly, w, h))
    }

    /// True when `(x, y)` lands on tessellated glyph ink (not the empty cell).
    fn glyph_ink_at(&self, x: f64, y: f64) -> bool {
        let Some((mut lx, ly, _, h)) = self.local_hit(x, y) else {
            return false;
        };
        let wch = self.sx.max(1) as f64;
        let h = h.max(1e-9);
        if self.style & STYLE_ITALIC != 0 {
            lx -= (1.0 - ly / h) * wch * ITALIC_SHEAR as f64;
        }
        if lx < 0.0 {
            return false;
        }
        let i = (lx / wch).floor();
        if i < 0.0 {
            return false;
        }
        let i = i as usize;
        let Some(ch) = self.text.chars().nth(i) else {
            return false;
        };
        let u = (lx / wch - i as f64) as f32;
        let v = (ly / h) as f32;
        glyph_ink(&self.font, ch, u, v)
    }
}

static GLYPH_INK: OnceLock<fn(&str, char, f32, f32) -> bool> = OnceLock::new();

/// GPU installs real Courier-Prime coverage; without it, glyph cells are hollow.
pub fn set_glyph_ink(hit: fn(&str, char, f32, f32) -> bool) {
    let _ = GLYPH_INK.set(hit);
}

fn glyph_ink(font: &str, ch: char, u: f32, v: f32) -> bool {
    GLYPH_INK.get().is_some_and(|f| f(font, ch, u, v))
}
