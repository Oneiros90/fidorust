//! FidoRust core: FidoCAD 0.96 document model, .fcd/.fcl I/O, libraries, hit-testing, undo.

pub mod commands;
pub mod consts;
pub mod document;
pub mod editor;
pub mod fcd;
pub mod geom;
pub mod hit;
pub mod layers;
pub mod library;
pub mod primitive;
pub mod properties;

pub use document::{Document, SaveOptions};
pub use editor::{Editor, EditorError, TextEditSession, Tool};
pub use fcd::{parse_document, parse_library, serialize_document, ParseError};
pub use geom::{Aabb, Point, Transform};
pub use layers::{LayerId, LayerSet, MAX_LAYERS, MICRON_PER_LU};
pub use library::{Library, LibrarySet, MacroDef};
pub use primitive::{
    Bezier, Connection, Ellipse, Line, MacroRef, PadStyle, PcbPad, PcbTrack, Poly, Primitive, Rect,
    Text, TextLayout, TextStyle,
};
pub use properties::{PropFormField, PropPatch};

/// Compatibility path used by tests and sibling crates (`fidocad_core::parse::…`).
pub use fcd as parse;
/// Compatibility path used by tests and sibling crates (`fidocad_core::serialize::…`).
pub use fcd as serialize;

pub const MACRO_ORIGIN: Point = Point { x: 100, y: 100 };
