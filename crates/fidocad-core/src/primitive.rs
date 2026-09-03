//! Graphic primitives of FidoCAD 0.96.

use crate::geom::{bezier_point, Aabb, Point, Transform};
use crate::layers::LayerId;
use crate::MACRO_ORIGIN;
use serde::{Deserialize, Serialize};

pub const MAX_POLY_VERTICES: usize = 10; // original MAX_DEFINING_POINTS

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadStyle {
    Oval = 0,
    Rectangular = 1,
    RoundedRect = 2,
}

impl PadStyle {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Rectangular,
            2 => Self::RoundedRect,
            _ => Self::Oval,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveKind {
    Line,
    Rect { filled: bool },
    Poly { filled: bool },
    Ellipse { filled: bool },
    Bezier,
    Text,
    Connection,
    PcbTrack,
    PcbPad,
    Macro,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Primitive {
    Line {
        a: Point,
        b: Point,
        layer: LayerId,
    },
    Rect {
        a: Point,
        b: Point,
        filled: bool,
        layer: LayerId,
    },
    Poly {
        pts: Vec<Point>,
        filled: bool,
        layer: LayerId,
    },
    Ellipse {
        a: Point,
        b: Point,
        filled: bool,
        layer: LayerId,
    },
    Bezier {
        p0: Point,
        p1: Point,
        p2: Point,
        p3: Point,
        layer: LayerId,
    },
    /// Simple `TE` (obsolete) and advanced `TY`/`TX`.
    Text {
        pos: Point,
        sy: i32,
        sx: i32,
        angle: i32,
        style: u32,
        layer: LayerId,
        font: String,
        text: String,
        /// True if this came from a `TE` command (round-trip as TE if desired).
        simple: bool,
    },
    Connection {
        pos: Point,
        layer: LayerId,
    },
    PcbTrack {
        a: Point,
        b: Point,
        width: i32,
        layer: LayerId,
    },
    PcbPad {
        pos: Point,
        dx: i32,
        dy: i32,
        hole: i32,
        style: PadStyle,
        layer: LayerId,
    },
    /// Unexpanded macro instance. Body is expanded at draw/hit time.
    Macro {
        pos: Point,
        rotations: u8,
        mirrored: bool,
        name: String,
        standard: bool,
    },
}

impl Primitive {
    pub fn kind(&self) -> PrimitiveKind {
        match self {
            Self::Line { .. } => PrimitiveKind::Line,
            Self::Rect { filled, .. } => PrimitiveKind::Rect { filled: *filled },
            Self::Poly { filled, .. } => PrimitiveKind::Poly { filled: *filled },
            Self::Ellipse { filled, .. } => PrimitiveKind::Ellipse { filled: *filled },
            Self::Bezier { .. } => PrimitiveKind::Bezier,
            Self::Text { .. } => PrimitiveKind::Text,
            Self::Connection { .. } => PrimitiveKind::Connection,
            Self::PcbTrack { .. } => PrimitiveKind::PcbTrack,
            Self::PcbPad { .. } => PrimitiveKind::PcbPad,
            Self::Macro { .. } => PrimitiveKind::Macro,
        }
    }

    pub fn layer(&self) -> LayerId {
        match self {
            Self::Line { layer, .. }
            | Self::Rect { layer, .. }
            | Self::Poly { layer, .. }
            | Self::Ellipse { layer, .. }
            | Self::Bezier { layer, .. }
            | Self::Text { layer, .. }
            | Self::Connection { layer, .. }
            | Self::PcbTrack { layer, .. }
            | Self::PcbPad { layer, .. } => *layer,
            Self::Macro { .. } => LayerId(0),
        }
    }

    pub fn set_layer(&mut self, l: LayerId) {
        match self {
            Self::Line { layer, .. }
            | Self::Rect { layer, .. }
            | Self::Poly { layer, .. }
            | Self::Ellipse { layer, .. }
            | Self::Bezier { layer, .. }
            | Self::Text { layer, .. }
            | Self::Connection { layer, .. }
            | Self::PcbTrack { layer, .. }
            | Self::PcbPad { layer, .. } => *layer = l,
            Self::Macro { .. } => {}
        }
    }

    pub fn transform(&mut self, f: impl Fn(Point) -> Point) {
        match self {
            Self::Line { a, b, .. } | Self::Rect { a, b, .. } | Self::Ellipse { a, b, .. } => {
                *a = f(*a);
                *b = f(*b);
            }
            Self::Poly { pts, .. } => {
                for p in pts {
                    *p = f(*p);
                }
            }
            Self::Bezier { p0, p1, p2, p3, .. } => {
                *p0 = f(*p0);
                *p1 = f(*p1);
                *p2 = f(*p2);
                *p3 = f(*p3);
            }
            Self::Text { pos, .. } | Self::Connection { pos, .. } | Self::PcbPad { pos, .. } => {
                *pos = f(*pos);
            }
            Self::PcbTrack { a, b, .. } => {
                *a = f(*a);
                *b = f(*b);
            }
            Self::Macro { pos, .. } => *pos = f(*pos),
        }
    }

    pub fn apply_transform(&mut self, xf: Transform) {
        self.transform(|p| xf.apply(p, MACRO_ORIGIN));
        if let Self::Macro {
            rotations,
            mirrored,
            ..
        } = self
        {
            *rotations = (*rotations + xf.rotations) % 4;
            if xf.mirrored {
                *mirrored = !*mirrored;
            }
        }
        if let Self::PcbPad { dx, dy, .. } = self {
            if xf.rotations % 2 == 1 {
                std::mem::swap(dx, dy);
            }
        }
    }

    pub fn aabb(&self) -> Aabb {
        match self {
            Self::Line { a, b, .. } | Self::Rect { a, b, .. } | Self::Ellipse { a, b, .. } => {
                Aabb::from_points([*a, *b])
            }
            Self::Poly { pts, .. } => Aabb::from_points(pts.iter().copied()),
            Self::Bezier { p0, p1, p2, p3, .. } => {
                let mut bb = Aabb::from_points([*p0, *p3]);
                for i in 1..16 {
                    let t = i as f32 / 16.0;
                    let (x, y) = bezier_point(*p0, *p1, *p2, *p3, t);
                    bb.include(Point::new(x.round() as i32, y.round() as i32));
                }
                bb
            }
            Self::Text {
                pos,
                sy,
                sx,
                angle,
                style,
                text,
                ..
            } => text_aabb(*pos, *sx, *sy, *angle, *style, text),
            Self::Connection { pos, .. } => Aabb {
                min: Point::new(pos.x - 2, pos.y - 2),
                max: Point::new(pos.x + 2, pos.y + 2),
            },
            Self::PcbTrack { a, b, width, .. } => {
                let pad = (*width / 2).max(1);
                Aabb::from_points([*a, *b]).expand(pad)
            }
            Self::PcbPad { pos, dx, dy, .. } => Aabb {
                min: Point::new(pos.x - dx / 2, pos.y - dy / 2),
                max: Point::new(pos.x + dx / 2, pos.y + dy / 2),
            },
            Self::Macro { pos, .. } => Aabb {
                min: Point::new(pos.x - 10, pos.y - 10),
                max: Point::new(pos.x + 10, pos.y + 10),
            },
        }
    }

    pub fn control_points(&self) -> Vec<Point> {
        match self {
            Self::Line { a, b, .. }
            | Self::Rect { a, b, .. }
            | Self::Ellipse { a, b, .. }
            | Self::PcbTrack { a, b, .. } => vec![*a, *b],
            Self::Poly { pts, .. } => pts.clone(),
            Self::Bezier { p0, p1, p2, p3, .. } => vec![*p0, *p1, *p2, *p3],
            Self::Text { pos, .. }
            | Self::Connection { pos, .. }
            | Self::PcbPad { pos, .. }
            | Self::Macro { pos, .. } => vec![*pos],
        }
    }

    pub fn set_control_point(&mut self, index: usize, p: Point) {
        match self {
            Self::Line { a, b, .. }
            | Self::Rect { a, b, .. }
            | Self::Ellipse { a, b, .. }
            | Self::PcbTrack { a, b, .. } => match index {
                0 => *a = p,
                1 => *b = p,
                _ => {}
            },
            Self::Poly { pts, .. } => {
                if let Some(slot) = pts.get_mut(index) {
                    *slot = p;
                }
            }
            Self::Bezier { p0, p1, p2, p3, .. } => match index {
                0 => *p0 = p,
                1 => *p1 = p,
                2 => *p2 = p,
                3 => *p3 = p,
                _ => {}
            },
            Self::Text { pos, .. }
            | Self::Connection { pos, .. }
            | Self::PcbPad { pos, .. }
            | Self::Macro { pos, .. } => {
                if index == 0 {
                    *pos = p;
                }
            }
        }
    }
}

/// World AABB of text glyphs (same transform as the GPU tessellator / hit test).
fn text_aabb(pos: Point, sx: i32, sy: i32, angle: i32, style: u32, text: &str) -> Aabb {
    let n = text.chars().count().max(1) as f32;
    let w = sx.max(1) as f32 * n;
    let h = sy.max(1) as f32;
    // Italic shear in the tessellator can stick out a little past `w`.
    let pad = if style & 2 != 0 { w * 0.22 } else { 0.0 };
    let (x0, x1) = if style & 4 != 0 {
        (-w - pad, 0.0)
    } else {
        (0.0, w + pad)
    };
    let rad = (angle as f32).to_radians();
    let (sin, cos) = rad.sin_cos();
    let mut bb = Aabb::empty();
    for (lx, ly) in [(x0, 0.0), (x1, 0.0), (x0, h), (x1, h)] {
        let wx = pos.x as f32 + lx * cos - ly * sin;
        let wy = pos.y as f32 + lx * sin + ly * cos;
        bb.include(Point::new(wx.round() as i32, wy.round() as i32));
    }
    bb
}
