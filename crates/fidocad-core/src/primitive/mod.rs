//! Graphic primitives of FidoCAD 0.96.

mod bezier;
mod component;
mod connection;
mod ellipse;
mod line;
mod pad;
mod pcb_pad;
mod pcb_track;
mod poly;
mod rect;
mod text;
mod traits;

pub use bezier::Bezier;
pub use component::ComponentRef;
pub use connection::Connection;
pub use ellipse::Ellipse;
pub use line::Line;
pub use pad::PadStyle;
pub use pcb_pad::PcbPad;
pub use pcb_track::PcbTrack;
pub use poly::Poly;
pub use rect::Rect;
pub use text::{
    Text, TextLayout, TextStyle, ITALIC_SHEAR, STYLE_BOLD, STYLE_ITALIC, STYLE_MIRRORED,
    STYLE_UNDERLINE,
};
pub use traits::{Geometry, HitTest};

use crate::geom::{Point, Transform};
use crate::layers::LayerId;
use crate::COMPONENT_ORIGIN;
use serde::{Deserialize, Serialize};

pub const MAX_POLY_VERTICES: usize = 10;
pub const BEZIER_SEGMENTS_HIT: u32 = 16;
pub const BEZIER_SEGMENTS_DRAW: u32 = 48;
/// FidoCad `*` token and FidoRust canvas default (bundled Courier Prime).
pub const DEFAULT_FONT: &str = "Courier Prime";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Primitive {
    Line(Line),
    Rect(Rect),
    Poly(Poly),
    Ellipse(Ellipse),
    Bezier(Bezier),
    Text(Text),
    Connection(Connection),
    PcbTrack(PcbTrack),
    PcbPad(PcbPad),
    Component(ComponentRef),
}

#[macro_export]
macro_rules! dispatch_primitive {
    ($self:expr, |$p:ident| $body:expr) => {
        match $self {
            $crate::primitive::Primitive::Line($p) => $body,
            $crate::primitive::Primitive::Rect($p) => $body,
            $crate::primitive::Primitive::Poly($p) => $body,
            $crate::primitive::Primitive::Ellipse($p) => $body,
            $crate::primitive::Primitive::Bezier($p) => $body,
            $crate::primitive::Primitive::Text($p) => $body,
            $crate::primitive::Primitive::Connection($p) => $body,
            $crate::primitive::Primitive::PcbTrack($p) => $body,
            $crate::primitive::Primitive::PcbPad($p) => $body,
            $crate::primitive::Primitive::Component($p) => $body,
        }
    };
}

impl Primitive {
    pub fn line(a: Point, b: Point, layer: LayerId) -> Self {
        Self::Line(Line { a, b, layer })
    }
    pub fn rect(a: Point, b: Point, filled: bool, layer: LayerId) -> Self {
        Self::Rect(Rect {
            a,
            b,
            filled,
            layer,
        })
    }
    pub fn ellipse(a: Point, b: Point, filled: bool, layer: LayerId) -> Self {
        Self::Ellipse(Ellipse {
            a,
            b,
            filled,
            layer,
        })
    }
    pub fn poly(pts: Vec<Point>, filled: bool, layer: LayerId) -> Self {
        Self::Poly(Poly { pts, filled, layer })
    }
    pub fn bezier(p0: Point, p1: Point, p2: Point, p3: Point, layer: LayerId) -> Self {
        Self::Bezier(Bezier {
            p0,
            p1,
            p2,
            p3,
            layer,
        })
    }
    pub fn text(t: Text) -> Self {
        Self::Text(t)
    }
    pub fn connection(pos: Point, layer: LayerId) -> Self {
        Self::Connection(Connection { pos, layer })
    }
    pub fn pcb_track(a: Point, b: Point, width: i32, layer: LayerId) -> Self {
        Self::PcbTrack(PcbTrack { a, b, width, layer })
    }
    pub fn pcb_pad(pad: PcbPad) -> Self {
        Self::PcbPad(pad)
    }
    pub fn component_ref(m: ComponentRef) -> Self {
        Self::Component(m)
    }

    pub fn is_component(&self) -> bool {
        matches!(self, Self::Component(_))
    }

    pub fn layer(&self) -> LayerId {
        dispatch_primitive!(self, |p| p.layer())
    }

    pub fn set_layer(&mut self, l: LayerId) {
        dispatch_primitive!(self, |p| p.set_layer(l));
    }

    pub fn transform(&mut self, f: impl Fn(Point) -> Point + Copy) {
        dispatch_primitive!(self, |p| p.transform_points(f));
    }

    pub fn apply_transform(&mut self, xf: Transform) {
        self.transform(|p| xf.apply(p, COMPONENT_ORIGIN));
        if let Self::Component(m) = self {
            m.rotations = (m.rotations + xf.rotations) % 4;
            if xf.mirrored {
                m.mirrored = !m.mirrored;
            }
        }
        if let Self::Text(t) = self {
            // MapCoordinates `rotations` is clockwise 90° steps. `rotate_at` is
            // CCW (`rotate90_cw`) and adds +90° to `Text.angle`; one CW step is
            // three CCW steps, so subtract 90° per stored rotation.
            t.angle = (t.angle - 90 * i32::from(xf.rotations)).rem_euclid(360);
            if xf.mirrored {
                t.style ^= STYLE_MIRRORED;
            }
        }
        if let Self::PcbPad(pad) = self {
            if xf.rotations % 2 == 1 {
                std::mem::swap(&mut pad.dx, &mut pad.dy);
            }
        }
    }

    pub fn aabb(&self) -> crate::geom::Aabb {
        dispatch_primitive!(self, |p| p.aabb())
    }

    pub fn control_points(&self) -> Vec<Point> {
        dispatch_primitive!(self, |p| p.control_points())
    }

    pub fn set_control_point(&mut self, index: usize, p: Point) {
        dispatch_primitive!(self, |s| s.set_control_point(index, p));
    }

    pub fn body_hit(&self, pt: Point, tol2: f64) -> bool {
        dispatch_primitive!(self, |p| p.body_hit(pt, tol2))
    }
}
