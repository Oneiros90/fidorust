//! Path builders for GPU tessellation.

use fidorust_core::geom::Point;
use lyon::math::point;

pub const PCB_PAD_CORNER_SEGS: u32 = 12;

/// `dir` is `+1.0` (CCW, filled ellipse) or `-1.0` (CW, even-odd hole).
pub fn path_ellipse(
    builder: &mut lyon::path::Builder,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    dir: f32,
) {
    const SEGS: u32 = 24;
    builder.begin(point(cx + rx, cy));
    for i in 1..=SEGS {
        let t = dir * std::f32::consts::TAU * i as f32 / SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(cx + rx * ct, cy + ry * st));
    }
    builder.close();
}

pub fn path_rect(builder: &mut lyon::path::Builder, cx: f32, cy: f32, hx: f32, hy: f32) {
    builder.begin(point(cx - hx, cy - hy));
    builder.line_to(point(cx + hx, cy - hy));
    builder.line_to(point(cx + hx, cy + hy));
    builder.line_to(point(cx - hx, cy + hy));
    builder.close();
}

pub fn path_rounded_rect(
    builder: &mut lyon::path::Builder,
    cx: f32,
    cy: f32,
    hx: f32,
    hy: f32,
    rx: f32,
    ry: f32,
) {
    let x0 = cx - hx;
    let x1 = cx + hx;
    let y0 = cy - hy;
    let y1 = cy + hy;
    let rx = rx.min(hx);
    let ry = ry.min(hy);
    builder.begin(point(x0 + rx, y0));
    builder.line_to(point(x1 - rx, y0));
    for i in 1..=PCB_PAD_CORNER_SEGS {
        let t = -std::f32::consts::FRAC_PI_2
            + std::f32::consts::FRAC_PI_2 * i as f32 / PCB_PAD_CORNER_SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(x1 - rx + rx * ct, y0 + ry + ry * st));
    }
    builder.line_to(point(x1, y1 - ry));
    for i in 1..=PCB_PAD_CORNER_SEGS {
        let t = std::f32::consts::FRAC_PI_2 * i as f32 / PCB_PAD_CORNER_SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(x1 - rx + rx * ct, y1 - ry + ry * st));
    }
    builder.line_to(point(x0 + rx, y1));
    for i in 1..=PCB_PAD_CORNER_SEGS {
        let t = std::f32::consts::FRAC_PI_2
            + std::f32::consts::FRAC_PI_2 * i as f32 / PCB_PAD_CORNER_SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(x0 + rx + rx * ct, y1 - ry + ry * st));
    }
    builder.line_to(point(x0, y0 + ry));
    for i in 1..=PCB_PAD_CORNER_SEGS {
        let t = std::f32::consts::PI
            + std::f32::consts::FRAC_PI_2 * i as f32 / PCB_PAD_CORNER_SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(x0 + rx + rx * ct, y0 + ry + ry * st));
    }
    builder.close();
}

pub fn rect_corners(a: Point, b: Point) -> [Point; 4] {
    [
        Point::new(a.x, a.y),
        Point::new(b.x, a.y),
        Point::new(b.x, b.y),
        Point::new(a.x, b.y),
    ]
}
