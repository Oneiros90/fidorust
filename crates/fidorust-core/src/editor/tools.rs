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
    Ruler,
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
            Self::Ruler => "ruler",
        }
    }

    /// Line, rect, ellipse, and PCB track: drag or click-click to place two points.
    pub fn is_two_point_draw(self) -> bool {
        matches!(
            self,
            Self::Line | Self::Rect | Self::Ellipse | Self::PcbTrack
        )
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
            "ruler" => Self::Ruler,
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
        /// Selection at marquee start (empty unless Shift). Recomputed hits union this.
        kept: Vec<usize>,
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

/// Result of a canvas double-click.
#[derive(Clone, Debug, PartialEq)]
pub enum DblClickAction {
    None,
    TextEdit(TextEditSession),
    OpenProperties,
}

/// Layout of a text primitive for an in-scene editor overlay.
#[derive(Clone, Debug, Serialize, PartialEq)]
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
