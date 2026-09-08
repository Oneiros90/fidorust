//! Named limits shared by parse, hit-testing, and the editor.

/// Wheel-zoom and zoom-tool lower bound.
pub const ZOOM_MIN: f32 = 0.4;
/// `wheel_zoom` / zoom-tool upper bound (fit_view uses `ZOOM_MAX_FIT`).
pub const ZOOM_MAX_WHEEL: f32 = 40.0;
/// `fit_view` upper bound (intentionally tighter than wheel zoom).
pub const ZOOM_MAX_FIT: f32 = 20.0;
/// Zoom-tool click multiplier.
pub const ZOOM_TOOL_FACTOR: f32 = 1.3;
/// Mouse-wheel zoom step.
pub const WHEEL_ZOOM_FACTOR: f32 = 1.12;
/// Default / empty-document pan and `fit_view` margin in canvas pixels.
pub const FIT_MARGIN: f32 = 40.0;
/// Undo stack cap (oldest snapshot dropped).
pub const UNDO_CAP: usize = 64;
/// Extra world units around the viewport AABB when culling.
pub const VIEW_CULL_MARGIN: i32 = 50;
/// Extra expand on primitive AABB when testing viewport intersection.
pub const AABB_CULL_EXPAND: i32 = 30;
/// Nested-component expansion depth (original FidoCAD).
pub const COMPONENT_MAX_DEPTH: u8 = 8;
/// Default `TY` font size inserted by the Text tool (`TE` parse uses 5 — keep both).
pub const DEFAULT_TEXT_SY: i32 = 4;
pub const DEFAULT_TEXT_SX: i32 = 3;
/// Default schematic stroke in hundredths of LU (`25` = 0.25 LU).
pub const DEFAULT_STROKE_HUNDREDTHS: i32 = 25;
pub const STROKE_HUNDREDTHS_MIN: i32 = 1;
pub const STROKE_HUNDREDTHS_MAX: i32 = 2000;
/// Grid pitch limits (original FidoCAD `m_xgrid` / `m_ygrid`).
pub const GRID_MIN: i32 = 1;
pub const GRID_MAX: i32 = 40;
/// Snap pitch limits (original FidoCAD `m_xsnap` / `m_ysnap`).
pub const SNAP_MIN: i32 = 1;
pub const SNAP_MAX: i32 = 20;
/// Default PCB track width / pad size for new objects.
pub const DEFAULT_TRACK_WIDTH: i32 = 4;
pub const DEFAULT_PAD_DX: i32 = 18;
pub const DEFAULT_PAD_DY: i32 = 18;
pub const DEFAULT_PAD_HOLE: i32 = 8;
/// Selection-handle radius in canvas pixels (screen-space, independent of zoom).
pub const HANDLE_RADIUS_PX: f32 = 6.0;
/// Connection body hit radius².
pub const CONNECTION_HIT_R2: f64 = 16.0;
/// Unexpanded-component origin hit radius² (fallback when children miss).
pub const COMPONENT_HIT_R2: f64 = 64.0;
/// Rect stroke hit half-width in LU.
pub const RECT_EDGE_TOL: i32 = 3;
/// Filled-ellipse inside threshold.
pub const ELLIPSE_FILL_TOL: f64 = 1.05;
/// Stroked-ellipse ring threshold.
pub const ELLIPSE_STROKE_TOL: f64 = 0.15;
/// Text-hit padding in local glyph space.
pub const TEXT_HIT_PAD: f64 = 2.0;
