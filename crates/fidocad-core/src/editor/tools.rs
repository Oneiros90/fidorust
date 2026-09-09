//! Drawing tools, drafts, and editor errors.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tool {
    Select,
    Line,
    Rect,
    Ellipse,
    Poly,
    Bezier,
    Text,
    Connection,
    PcbTrack,
    PcbPad,
    Component,
    Zoom,
    Pan,
}

impl Tool {
    pub fn from_id(id: &str) -> Self {
        id.parse().unwrap_or(Self::Select)
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Line => "line",
            Self::Rect => "rect",
            Self::Ellipse => "ellipse",
            Self::Poly => "poly",
            Self::Bezier => "bezier",
            Self::Text => "text",
            Self::Connection => "connection",
            Self::PcbTrack => "pcb-track",
            Self::PcbPad => "pcb-pad",
            Self::Component => "component",
            Self::Zoom => "zoom",
            Self::Pan => "pan",
        }
    }
}

impl std::str::FromStr for Tool {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "line" => Self::Line,
            "rect" => Self::Rect,
            "ellipse" => Self::Ellipse,
            "poly" => Self::Poly,
            "bezier" => Self::Bezier,
            "text" => Self::Text,
            "connection" => Self::Connection,
            "pcb-track" => Self::PcbTrack,
            "pcb-pad" => Self::PcbPad,
            "component" | "macro" => Self::Component,
            "zoom" => Self::Zoom,
            "pan" => Self::Pan,
            "select" => Self::Select,
            _ => return Err(()),
        })
    }
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

#[derive(Debug, Error)]
pub enum EditorError {
    #[error(transparent)]
    Parse(#[from] crate::fcd::ParseError),
    #[error("invalid property patch: {0}")]
    InvalidPatch(String),
}

#[derive(Clone, Debug)]
pub(super) struct Draft {
    pub tool: Tool,
    pub points: Vec<crate::geom::Point>,
}

#[derive(Clone, Debug)]
pub(super) enum Drag {
    Move {
        start: crate::geom::Point,
        last: crate::geom::Point,
        duplicate: bool,
    },
    Marquee {
        start: (f32, f32),
        current: (f32, f32),
    },
    Handle {
        index: usize,
        handle: usize,
    },
    Pan {
        start_screen: (f32, f32),
        pan0: (f32, f32),
    },
}

/// Layout of a text primitive for an in-scene editor overlay.
#[derive(Clone, Debug, Serialize)]
pub struct TextEditSession {
    pub index: usize,
    pub text: String,
    pub wx: i32,
    pub wy: i32,
    pub sx: i32,
    pub sy: i32,
    pub angle: i32,
    pub style: u32,
}
