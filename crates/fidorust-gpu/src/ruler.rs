//! Non-serialized ruler overlay: ticks and length labels.

use fidorust_core::geom::Point;
use fidorust_core::layers::MICRON_PER_LU;
use fidorust_core::primitive::{TextLayout, DEFAULT_FONT};
use fidorust_core::Tool;

use crate::draft::DraftParams;
use crate::scene::{FillVertexGpu, Scene};
use crate::theme::Rgb;

const TICK_PX: f32 = 8.0;
const STROKE_PX: f32 = 1.5;
const LABEL_H_PX: f32 = 13.0;
const LABEL_W_PX: f32 = 8.0;
const LABEL_GAP_PX: f32 = 4.0;

pub(crate) fn add_overlay(
    scene: &mut Scene,
    segments: &[(Point, Point)],
    draft: &DraftParams<'_>,
    zoom: f32,
    stroke_w: f32,
    dark: bool,
) {
    let rgb = Rgb::accent(dark).rgba(1.0);
    let z = zoom.max(0.01);
    let tick = TICK_PX / z;
    let w = stroke_w.max(STROKE_PX / z);
    let sy = LABEL_H_PX / z;
    let sx = LABEL_W_PX / z;
    let gap = LABEL_GAP_PX / z;

    for &(a, b) in segments {
        add_segment(scene, a, b, tick, w, sx, sy, gap, rgb);
    }
    if draft.tool == Some(Tool::Ruler) {
        if let Some((a, b)) = draft_ends(draft) {
            if a != b {
                add_segment(scene, a, b, tick, w, sx, sy, gap, rgb);
            }
        }
    }
}

fn draft_ends(draft: &DraftParams<'_>) -> Option<(Point, Point)> {
    let pts = draft.points;
    if pts.is_empty() {
        return None;
    }
    let a = pts[0];
    let b = if pts.len() >= 2 {
        pts[pts.len() - 1]
    } else {
        draft.hover?
    };
    Some((a, b))
}

#[allow(clippy::too_many_arguments)]
fn add_segment(
    scene: &mut Scene,
    a: Point,
    b: Point,
    tick: f32,
    stroke: f32,
    sx: f32,
    sy: f32,
    gap: f32,
    rgb: [f32; 4],
) {
    let ax = a.x as f32;
    let ay = a.y as f32;
    let bx = b.x as f32;
    let by = b.y as f32;
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }
    let mut ux = dx / len;
    let mut uy = dy / len;
    scene.push_line_f(ax, ay, bx, by, stroke, rgb, false);

    let nx = -uy;
    let ny = ux;
    let half = tick * 0.5;
    scene.push_line_f(
        ax + nx * half,
        ay + ny * half,
        ax - nx * half,
        ay - ny * half,
        stroke,
        rgb,
        false,
    );
    scene.push_line_f(
        bx + nx * half,
        by + ny * half,
        bx - nx * half,
        by - ny * half,
        stroke,
        rgb,
        false,
    );

    if ux < 0.0 || (ux.abs() <= 1e-6 && uy < 0.0) {
        ux = -ux;
        uy = -uy;
    }
    let mx = (ax + bx) * 0.5;
    let my = (ay + by) * 0.5;
    push_label(
        scene,
        &format_ruler_label(a, b),
        mx,
        my,
        ux,
        uy,
        sx,
        sy,
        gap,
        rgb,
    );
}

pub(crate) fn format_ruler_label(a: Point, b: Point) -> String {
    let lu = ((a.x - b.x) as f64).hypot((a.y - b.y) as f64);
    let mm = lu * f64::from(MICRON_PER_LU) / 1000.0;
    format!("{} ({} mm)", format_num(lu), format_num(mm))
}

fn format_num(v: f64) -> String {
    let s = format!("{v:.2}");
    match s.find('.') {
        Some(dot) => {
            let frac = s[dot..].trim_end_matches('0');
            if frac == "." {
                s[..dot].to_string()
            } else {
                format!("{}{frac}", &s[..dot])
            }
        }
        None => s,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_label(
    scene: &mut Scene,
    label: &str,
    mx: f32,
    my: f32,
    ux: f32,
    uy: f32,
    sx: f32,
    sy: f32,
    gap: f32,
    rgb: [f32; 4],
) {
    let n = label.chars().count().max(1) as f32;
    let ox = -sx * n * 0.5;
    let oy = -sy - gap;
    let cos = ux;
    let sin = -uy;
    let mut x_off = 0.0f32;
    let sel_f = Scene::flag(false);
    for ch in label.chars() {
        for v in crate::font::glyph_triangles(DEFAULT_FONT, ch) {
            let ra = TextLayout::map_offset(v[0] * sx + x_off + ox, v[1] * sy + oy, sin, cos);
            scene.fills.push(FillVertexGpu {
                x: mx + ra.0,
                y: my + ra.1,
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                a: rgb[3],
                selected: sel_f,
            });
        }
        x_off += sx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_integer_lu_trims_mm() {
        assert_eq!(
            format_ruler_label(Point::new(0, 0), Point::new(100, 0)),
            "100 (12.7 mm)"
        );
    }

    #[test]
    fn label_diagonal_345() {
        assert_eq!(
            format_ruler_label(Point::new(0, 0), Point::new(3, 4)),
            "5 (0.64 mm)"
        );
    }
}
