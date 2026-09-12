//! GPU tessellation, SVG export, WebGL renderer, and bundled/system fonts.
//!
//! Tessellation takes [`TessellateInput`] plus [`DraftParams`]; this crate does not
//! import `fidorust_core::Editor`. Callers (wasm, tests) map an editor into those
//! structs. [`font::install_hit_hooks`] injects glyph coverage into core at runtime.

mod draft;
pub mod font;
#[cfg(target_arch = "wasm32")]
pub mod renderer;
mod ruler;
mod scene;
mod shapes;
mod svg;
pub mod tessellate;
pub mod theme;

pub use tessellate::{
    export_svg, scene_to_cursor_svg, scene_to_svg, scene_to_thumb_svg, tessellate,
    tessellate_export, tessellate_primitives, CursorSvg, DraftParams, Scene, TessellateInput,
};
pub use theme::{Rgb, Theme};
