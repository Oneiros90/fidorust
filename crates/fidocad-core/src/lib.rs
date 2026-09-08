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

pub use document::{Document, ProjectSettings};
pub use editor::{Editor, EditorError, TextEditSession, Tool};
pub use fcd::{
    parse_document, parse_document_with_project_library, parse_library, serialize_document,
    serialize_document_with_policy, ParseError, SaveLibraryPolicy,
};
pub use geom::{Aabb, Point, Transform};
pub use layers::{LayerId, LayerSet, MAX_LAYERS, MICRON_PER_LU};
pub use library::{ComponentDef, Library, LibraryKind, LibrarySet, LOCAL_STEM, PROJECT_STEM};
pub use primitive::{
    Bezier, ComponentRef, Connection, Ellipse, Line, PadStyle, PcbPad, PcbTrack, Poly, Primitive,
    Rect, Text, TextLayout, TextStyle,
};
pub use properties::{PropFormField, PropPatch};

/// Compatibility path used by tests and sibling crates (`fidocad_core::parse::…`).
pub use fcd as parse;
/// Compatibility path used by tests and sibling crates (`fidocad_core::serialize::…`).
pub use fcd as serialize;

/// Insertion origin in component definition space (FidoCAD `MC` local origin).
pub const COMPONENT_ORIGIN: Point = Point { x: 100, y: 100 };
