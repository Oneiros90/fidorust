//! Live draft / rubber-band preview.

use fidorust_core::geom::Point;
use fidorust_core::{Editor, Tool};

use crate::scene::Scene;
use crate::shapes::rect_corners;

pub(crate) struct DraftParams<'a> {
    pub points: &'a [Point],
    pub tool: Option<Tool>,
    pub filled: bool,
    pub hover: Option<Point>,
}

impl<'a> DraftParams<'a> {
    pub(crate) fn from_editor(ed: &'a Editor) -> Self {
        Self {
            points: ed.draft_points(),
            tool: ed.draft_tool(),
            filled: ed.filled(),
            hover: ed.hover(),
        }
    }
}

pub(crate) fn add_draft(
    scene: &mut Scene,
    params: &DraftParams<'_>,
    preview: [f32; 4],
    stroke_w: f32,
) {
    let pts = params.points;
    if pts.is_empty() {
        return;
    }
    let a = pts[0];
    let b = if pts.len() >= 2 {
        pts[pts.len() - 1]
    } else if let Some(h) = params.hover {
        h
    } else {
        return;
    };
    match params.tool {
        Some(Tool::Ellipse) => {
            if a != b {
                scene.push_ellipse(a, b, params.filled, stroke_w, preview, false);
            }
        }
        Some(Tool::Rect) => {
            if a != b {
                let corners = rect_corners(a, b);
                if params.filled {
                    scene.fill_polygon(&corners, preview, false);
                } else {
                    scene.stroke_poly(&corners, true, stroke_w, preview, false);
                }
            }
        }
        Some(Tool::Poly) | Some(Tool::Bezier) => {
            scene.stroke_poly(pts, false, stroke_w, preview, false);
            if let Some(h) = params.hover {
                if let Some(&last) = pts.last() {
                    if last != h {
                        scene.push_line(last, h, stroke_w, preview, false);
                    }
                }
            }
        }
        _ => {
            if a != b {
                scene.push_line(a, b, stroke_w, preview, false);
            }
        }
    }
}
