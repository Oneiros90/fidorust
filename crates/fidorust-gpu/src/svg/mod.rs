//! SVG export: native primitives for file export, tessellated [`Scene`] for view/thumbs.

mod export;
mod scene;

pub use export::export_svg;
pub use scene::{scene_to_cursor_svg, scene_to_svg, scene_to_thumb_svg, CursorSvg};
