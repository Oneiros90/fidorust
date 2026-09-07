//! SVG export of a tessellated [`Scene`](crate::scene::Scene).

use fidocad_core::geom::Point;

use crate::scene::{CircleInstance, FillVertexGpu, LineInstance, Scene};
use crate::theme::Rgb;

struct SvgWriter<F> {
    tx: F,
    stroke_scale: f32,
    min_stroke: f32,
    emit_circles: bool,
}

impl<F: Fn(f32, f32) -> (f32, f32)> SvgWriter<F> {
    fn write(&self, out: &mut String, scene: &Scene) {
        for tri in scene.fills.chunks(3) {
            if tri.len() != 3 {
                continue;
            }
            write_polygon(out, &self.tx, tri);
        }
        for l in &scene.lines {
            write_line(
                out,
                &self.tx,
                l,
                self.stroke_scale,
                self.min_stroke,
                self.emit_circles,
            );
        }
        if self.emit_circles {
            for c in &scene.circles {
                write_circle(out, &self.tx, c, self.stroke_scale, self.min_stroke);
            }
        }
    }
}

fn fill_rgb(r: f32, g: f32, b: f32) -> String {
    Rgb([r, g, b]).to_svg()
}

fn write_polygon(out: &mut String, tx: &impl Fn(f32, f32) -> (f32, f32), tri: &[FillVertexGpu]) {
    let (x1, y1) = tx(tri[0].x, tri[0].y);
    let (x2, y2) = tx(tri[1].x, tri[1].y);
    let (x3, y3) = tx(tri[2].x, tri[2].y);
    out.push_str(&format!(
        r#"<polygon points="{x1:.2},{y1:.2} {x2:.2},{y2:.2} {x3:.2},{y3:.2}" fill="{}"/>"#,
        fill_rgb(tri[0].r, tri[0].g, tri[0].b),
    ));
}

fn write_line(
    out: &mut String,
    tx: &impl Fn(f32, f32) -> (f32, f32),
    l: &LineInstance,
    scale: f32,
    min_stroke: f32,
    emit_circles: bool,
) {
    let (x1, y1) = tx(l.ax, l.ay);
    let (x2, y2) = tx(l.bx, l.by);
    let stroke = fill_rgb(l.r, l.g, l.b);
    let width = (l.width * scale).max(min_stroke);
    // Export (`emit_circles == false`) used `stroke-width="{}"`; thumb/cursor used `{:.2}`.
    if emit_circles {
        out.push_str(&format!(
            r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{stroke}" stroke-width="{width:.2}" stroke-linecap="round"/>"#
        ));
    } else {
        out.push_str(&format!(
            r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{stroke}" stroke-width="{width}" stroke-linecap="round"/>"#
        ));
    }
}

fn write_circle(
    out: &mut String,
    tx: &impl Fn(f32, f32) -> (f32, f32),
    c: &CircleInstance,
    scale: f32,
    min_stroke: f32,
) {
    let (cx, cy) = tx(c.x, c.y);
    let rx = (c.rx * scale).max(0.4);
    let ry = (c.ry * scale).max(0.4);
    let stroke = fill_rgb(c.r, c.g, c.b);
    if c.stroke > 0.001 {
        out.push_str(&format!(
            r#"<ellipse cx="{cx:.2}" cy="{cy:.2}" rx="{rx:.2}" ry="{ry:.2}" stroke="{stroke}" stroke-width="{:.2}"/>"#,
            (c.stroke * scale).max(min_stroke),
        ));
    } else {
        out.push_str(&format!(
            r#"<ellipse cx="{cx:.2}" cy="{cy:.2}" rx="{rx:.2}" ry="{ry:.2}" fill="{stroke}"/>"#
        ));
    }
}

pub fn scene_to_svg(scene: &Scene, w: f32, h: f32, zoom: f32, pan: (f32, f32)) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#
    ));
    s.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
    let tx = |x: f32, y: f32| (x * zoom + pan.0, y * zoom + pan.1);
    SvgWriter {
        tx,
        stroke_scale: zoom,
        min_stroke: 0.6,
        emit_circles: false,
    }
    .write(&mut s, scene);
    s.push_str("</svg>");
    s
}

fn scene_bounds(scene: &Scene) -> Option<(f32, f32, f32, f32)> {
    let mut minx = f32::MAX;
    let mut miny = f32::MAX;
    let mut maxx = f32::MIN;
    let mut maxy = f32::MIN;
    let mut empty = true;
    let mut include = |x: f32, y: f32| {
        empty = false;
        minx = minx.min(x);
        miny = miny.min(y);
        maxx = maxx.max(x);
        maxy = maxy.max(y);
    };
    for l in &scene.lines {
        include(l.ax, l.ay);
        include(l.bx, l.by);
    }
    for c in &scene.circles {
        include(c.x - c.rx, c.y - c.ry);
        include(c.x + c.rx, c.y + c.ry);
    }
    for v in &scene.fills {
        include(v.x, v.y);
    }
    if empty {
        None
    } else {
        Some((minx, miny, maxx, maxy))
    }
}

pub fn scene_to_thumb_svg(scene: &Scene, size: f32) -> String {
    let Some((minx, miny, maxx, maxy)) = scene_bounds(scene) else {
        return format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"></svg>"#
        );
    };
    let bw = (maxx - minx).max(1.0);
    let bh = (maxy - miny).max(1.0);
    let pad = 0.14 * bw.max(bh);
    let span = bw.max(bh) + 2.0 * pad;
    let ox = minx - (span - bw) * 0.5;
    let oy = miny - (span - bh) * 0.5;
    let s = size / span;
    let tx = |x: f32, y: f32| ((x - ox) * s, (y - oy) * s);
    let mut out = String::new();
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}" fill="none">"#
    ));
    write_scene_svg(&mut out, scene, tx, s);
    out.push_str("</svg>");
    out
}

/// SVG in world LU, hotspot offset from the viewBox origin (for a cursor-following overlay).
#[derive(Clone, Debug)]
pub struct CursorSvg {
    pub svg: String,
    pub ox: f32,
    pub oy: f32,
    pub w: f32,
    pub h: f32,
}

pub fn scene_to_cursor_svg(scene: &Scene, origin: Point) -> CursorSvg {
    let Some((minx, miny, maxx, maxy)) = scene_bounds(scene) else {
        return CursorSvg {
            svg: String::new(),
            ox: 0.0,
            oy: 0.0,
            w: 0.0,
            h: 0.0,
        };
    };
    let pad = 1.5;
    let x0 = minx - pad;
    let y0 = miny - pad;
    let w = (maxx - minx) + 2.0 * pad;
    let h = (maxy - miny) + 2.0 * pad;
    let mut out = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0} {y0} {w} {h}" fill="none" overflow="visible">"#
    );
    write_scene_svg(&mut out, scene, |x, y| (x, y), 1.0);
    out.push_str("</svg>");
    CursorSvg {
        svg: out,
        ox: origin.x as f32 - x0,
        oy: origin.y as f32 - y0,
        w,
        h,
    }
}

fn write_scene_svg(
    out: &mut String,
    scene: &Scene,
    tx: impl Fn(f32, f32) -> (f32, f32),
    scale: f32,
) {
    SvgWriter {
        tx,
        stroke_scale: scale,
        min_stroke: if scale < 1.5 { 0.175 } else { 0.575 },
        emit_circles: true,
    }
    .write(out, scene);
}
