//! FidoRust core: FidoCAD 0.96 document model, .fcd/.fcl I/O, libraries, hit-testing, undo.

pub mod commands;
pub mod document;
pub mod geom;
pub mod hit;
pub mod layers;
pub mod library;
pub mod parse;
pub mod primitive;
pub mod serialize;

pub use commands::{Command, Editor, TextEditSession, Tool};
pub use document::{Document, SaveOptions};
pub use geom::{Aabb, Point, Transform};
pub use layers::{LayerId, LayerSet, LAYER_COUNT, MICRON_PER_LU};
pub use library::{Library, LibrarySet, MacroDef};
pub use parse::{parse_document, parse_library, ParseError};
pub use primitive::{PadStyle, Primitive, PrimitiveKind};
pub use serialize::serialize_document;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MACRO_ORIGIN: Point = Point { x: 100, y: 100 };
