//! Tessellate flattened primitives into GPU-friendly batches (world LU coordinates).

use fidocad_core::geom::{bezier_point, Point};
use fidocad_core::layers::LayerSet;
use fidocad_core::primitive::{PadStyle, Primitive};
use fidocad_core::{Editor, Tool};
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineInstance {
    pub ax: f32,
    pub ay: f32,
    pub bx: f32,
    pub by: f32,
    pub width: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub selected: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FillVertexGpu {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub selected: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub x: f32,
    pub y: f32,
    pub rx: f32,
    pub ry: f32,
    pub inner: f32,
    pub stroke: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub selected: f32,
}

#[derive(Default)]
pub struct Scene {
    pub lines: Vec<LineInstance>,
    pub fills: Vec<FillVertexGpu>,
    pub circles: Vec<CircleInstance>,
    pub handles: Vec<CircleInstance>,
}

fn color(layers: &LayerSet, p: &Primitive, selected: bool, dark: bool) -> [f32; 3] {
    if selected {
        return [0.85, 0.42, 0.22];
    }
    display_rgb(layers.color(p.layer()), dark)
}

fn display_rgb(c: [u8; 3], dark: bool) -> [f32; 3] {
    if dark {
        let lum = 0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32;
        if lum < 48.0 {
            return [1.0, 1.0, 1.0];
        }
    }
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
}

fn line(
    scene: &mut Scene,
    a: Point,
    b: Point,
    w: f32,
    rgb: [f32; 3],
    selected: bool,
) {
    scene.lines.push(LineInstance {
        ax: a.x as f32,
        ay: a.y as f32,
        bx: b.x as f32,
        by: b.y as f32,
        width: w,
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
        selected: if selected { 1.0 } else { 0.0 },
    });
}

fn circle(
    scene: &mut Scene,
    x: f32,
    y: f32,
    rx: f32,
    ry: f32,
    inner: f32,
    stroke: f32,
    rgb: [f32; 3],
    selected: bool,
) {
    scene.circles.push(CircleInstance {
        x,
        y,
        rx,
        ry,
        inner,
        stroke,
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
        selected: if selected { 1.0 } else { 0.0 },
    });
}

fn tessellate_path(path: &Path, rgb: [f32; 3], selected: bool, scene: &mut Scene) {
    let mut buffers: VertexBuffers<FillVertexGpu, u16> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let _ = tess.tessellate_path(
        path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| FillVertexGpu {
            x: v.position().x,
            y: v.position().y,
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            selected: if selected { 1.0 } else { 0.0 },
        }),
    );
    for tri in buffers.indices.chunks(3) {
        if tri.len() == 3 {
            for &i in tri {
                if let Some(v) = buffers.vertices.get(i as usize) {
                    scene.fills.push(*v);
                }
            }
        }
    }
}

fn stroke_poly(scene: &mut Scene, pts: &[Point], closed: bool, w: f32, rgb: [f32; 3], sel: bool) {
    for wdw in pts.windows(2) {
        line(scene, wdw[0], wdw[1], w, rgb, sel);
    }
    if closed && pts.len() > 2 {
        line(scene, *pts.last().unwrap(), pts[0], w, rgb, sel);
    }
}

fn add_text(scene: &mut Scene, p: &Primitive, rgb: [f32; 3], sel: bool) {
    let Primitive::Text {
        pos,
        sy,
        sx,
        angle,
        style,
        text,
        ..
    } = p
    else {
        return;
    };
    let h = (*sy as f32).max(2.0);
    let wch = (*sx as f32).max(1.5);
    let mirrored = style & 4 != 0;
    let rad = (*angle as f32).to_radians();
    let (sin, cos) = rad.sin_cos();
    let mut x_off = 0.0f32;
    for ch in text.chars() {
        let glyph = glyph_strokes(ch);
        for (a, b) in glyph {
            let mut ax = a.0 * wch + x_off;
            let ay = (1.0 - a.1) * h;
            let mut bx = b.0 * wch + x_off;
            let by = (1.0 - b.1) * h;
            if mirrored {
                ax = -ax;
                bx = -bx;
            }
            let ra = rot(ax, ay, cos, sin);
            let rb = rot(bx, by, cos, sin);
            line(
                scene,
                Point::new(
                    (pos.x as f32 + ra.0).round() as i32,
                    (pos.y as f32 + ra.1).round() as i32,
                ),
                Point::new(
                    (pos.x as f32 + rb.0).round() as i32,
                    (pos.y as f32 + rb.1).round() as i32,
                ),
                0.35,
                rgb,
                sel,
            );
        }
        x_off += wch * 1.15;
    }
}

fn rot(x: f32, y: f32, cos: f32, sin: f32) -> (f32, f32) {
    (x * cos - y * sin, x * sin + y * cos)
}

/// Very small vector font (unit square 0..1) for CAD labels.
fn glyph_strokes(ch: char) -> Vec<((f32, f32), (f32, f32))> {
    let c = if ch.is_ascii() { ch } else { '?' };
    match c {
        ' ' => vec![],
        '-' => vec![((0.1, 0.5), (0.9, 0.5))],
        '+' => vec![((0.5, 0.15), (0.5, 0.85)), ((0.15, 0.5), (0.85, 0.5))],
        '=' => vec![((0.1, 0.4), (0.9, 0.4)), ((0.1, 0.6), (0.9, 0.6))],
        '.' => vec![((0.4, 0.1), (0.6, 0.1)), ((0.4, 0.1), (0.4, 0.2))],
        ',' => vec![((0.4, 0.15), (0.55, 0.0))],
        '/' => vec![((0.15, 0.05), (0.85, 0.95))],
        '0' => box_glyph(true),
        '1' => vec![((0.5, 0.1), (0.5, 0.9)), ((0.35, 0.75), (0.5, 0.9))],
        '2' => vec![
            ((0.15, 0.85), (0.85, 0.85)),
            ((0.85, 0.85), (0.85, 0.55)),
            ((0.85, 0.55), (0.15, 0.55)),
            ((0.15, 0.55), (0.15, 0.15)),
            ((0.15, 0.15), (0.85, 0.15)),
        ],
        '3' => vec![
            ((0.15, 0.85), (0.85, 0.85)),
            ((0.85, 0.85), (0.85, 0.15)),
            ((0.15, 0.15), (0.85, 0.15)),
            ((0.35, 0.5), (0.85, 0.5)),
        ],
        '4' => vec![
            ((0.2, 0.9), (0.2, 0.5)),
            ((0.2, 0.5), (0.85, 0.5)),
            ((0.7, 0.9), (0.7, 0.1)),
        ],
        '5' => vec![
            ((0.85, 0.85), (0.15, 0.85)),
            ((0.15, 0.85), (0.15, 0.55)),
            ((0.15, 0.55), (0.8, 0.55)),
            ((0.8, 0.55), (0.8, 0.15)),
            ((0.8, 0.15), (0.15, 0.15)),
        ],
        '6' => vec![
            ((0.8, 0.85), (0.2, 0.85)),
            ((0.2, 0.85), (0.2, 0.15)),
            ((0.2, 0.15), (0.8, 0.15)),
            ((0.8, 0.15), (0.8, 0.5)),
            ((0.8, 0.5), (0.2, 0.5)),
        ],
        '7' => vec![((0.15, 0.85), (0.85, 0.85)), ((0.85, 0.85), (0.35, 0.1))],
        '8' => {
            let mut g = box_glyph(true);
            g.push(((0.15, 0.5), (0.85, 0.5)));
            g
        }
        '9' => vec![
            ((0.2, 0.15), (0.8, 0.15)),
            ((0.8, 0.15), (0.8, 0.85)),
            ((0.8, 0.85), (0.2, 0.85)),
            ((0.2, 0.85), (0.2, 0.5)),
            ((0.2, 0.5), (0.8, 0.5)),
        ],
        'A' => vec![
            ((0.1, 0.1), (0.5, 0.9)),
            ((0.5, 0.9), (0.9, 0.1)),
            ((0.28, 0.4), (0.72, 0.4)),
        ],
        'B' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.2, 0.9), (0.7, 0.9)),
            ((0.7, 0.9), (0.8, 0.7)),
            ((0.8, 0.7), (0.7, 0.5)),
            ((0.2, 0.5), (0.7, 0.5)),
            ((0.7, 0.5), (0.85, 0.3)),
            ((0.85, 0.3), (0.7, 0.1)),
            ((0.7, 0.1), (0.2, 0.1)),
        ],
        'C' => vec![
            ((0.85, 0.8), (0.25, 0.8)),
            ((0.25, 0.8), (0.2, 0.2)),
            ((0.2, 0.2), (0.85, 0.2)),
        ],
        'D' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.2, 0.9), (0.65, 0.9)),
            ((0.65, 0.9), (0.85, 0.55)),
            ((0.85, 0.55), (0.65, 0.1)),
            ((0.65, 0.1), (0.2, 0.1)),
        ],
        'E' => vec![
            ((0.8, 0.9), (0.2, 0.9)),
            ((0.2, 0.9), (0.2, 0.1)),
            ((0.2, 0.1), (0.8, 0.1)),
            ((0.2, 0.5), (0.65, 0.5)),
        ],
        'F' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.2, 0.9), (0.8, 0.9)),
            ((0.2, 0.5), (0.65, 0.5)),
        ],
        'G' => vec![
            ((0.8, 0.8), (0.25, 0.8)),
            ((0.25, 0.8), (0.2, 0.2)),
            ((0.2, 0.2), (0.8, 0.2)),
            ((0.8, 0.2), (0.8, 0.5)),
            ((0.8, 0.5), (0.55, 0.5)),
        ],
        'H' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.8, 0.1), (0.8, 0.9)),
            ((0.2, 0.5), (0.8, 0.5)),
        ],
        'I' => vec![
            ((0.3, 0.9), (0.7, 0.9)),
            ((0.5, 0.9), (0.5, 0.1)),
            ((0.3, 0.1), (0.7, 0.1)),
        ],
        'K' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.2, 0.5), (0.8, 0.9)),
            ((0.2, 0.5), (0.8, 0.1)),
        ],
        'L' => vec![((0.2, 0.9), (0.2, 0.1)), ((0.2, 0.1), (0.8, 0.1))],
        'M' => vec![
            ((0.15, 0.1), (0.15, 0.9)),
            ((0.15, 0.9), (0.5, 0.5)),
            ((0.5, 0.5), (0.85, 0.9)),
            ((0.85, 0.9), (0.85, 0.1)),
        ],
        'N' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.2, 0.9), (0.8, 0.1)),
            ((0.8, 0.1), (0.8, 0.9)),
        ],
        'O' => box_glyph(true),
        'P' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.2, 0.9), (0.75, 0.9)),
            ((0.75, 0.9), (0.75, 0.5)),
            ((0.75, 0.5), (0.2, 0.5)),
        ],
        'Q' => {
            let mut g = box_glyph(true);
            g.push(((0.55, 0.35), (0.9, 0.05)));
            g
        }
        'R' => vec![
            ((0.2, 0.1), (0.2, 0.9)),
            ((0.2, 0.9), (0.75, 0.9)),
            ((0.75, 0.9), (0.75, 0.5)),
            ((0.75, 0.5), (0.2, 0.5)),
            ((0.45, 0.5), (0.8, 0.1)),
        ],
        'S' => vec![
            ((0.8, 0.8), (0.2, 0.8)),
            ((0.2, 0.8), (0.2, 0.5)),
            ((0.2, 0.5), (0.8, 0.5)),
            ((0.8, 0.5), (0.8, 0.2)),
            ((0.8, 0.2), (0.2, 0.2)),
        ],
        'T' => vec![((0.15, 0.9), (0.85, 0.9)), ((0.5, 0.9), (0.5, 0.1))],
        'U' => vec![
            ((0.2, 0.9), (0.2, 0.2)),
            ((0.2, 0.2), (0.8, 0.2)),
            ((0.8, 0.2), (0.8, 0.9)),
        ],
        'V' => vec![((0.15, 0.9), (0.5, 0.1)), ((0.5, 0.1), (0.85, 0.9))],
        'W' => vec![
            ((0.1, 0.9), (0.3, 0.1)),
            ((0.3, 0.1), (0.5, 0.55)),
            ((0.5, 0.55), (0.7, 0.1)),
            ((0.7, 0.1), (0.9, 0.9)),
        ],
        'X' => vec![((0.2, 0.9), (0.8, 0.1)), ((0.8, 0.9), (0.2, 0.1))],
        'Y' => vec![
            ((0.2, 0.9), (0.5, 0.5)),
            ((0.8, 0.9), (0.5, 0.5)),
            ((0.5, 0.5), (0.5, 0.1)),
        ],
        'Z' => vec![
            ((0.15, 0.9), (0.85, 0.9)),
            ((0.85, 0.9), (0.15, 0.1)),
            ((0.15, 0.1), (0.85, 0.1)),
        ],
        _ => {
            let up = c.to_ascii_uppercase();
            if up != c && up.is_ascii_alphabetic() {
                return glyph_strokes(up);
            }
            box_glyph(false)
        }
    }
}

fn box_glyph(closed: bool) -> Vec<((f32, f32), (f32, f32))> {
    let mut g = vec![
        ((0.2, 0.15), (0.8, 0.15)),
        ((0.8, 0.15), (0.8, 0.85)),
        ((0.8, 0.85), (0.2, 0.85)),
    ];
    if closed {
        g.push(((0.2, 0.85), (0.2, 0.15)));
    }
    g
}

fn add_prim(scene: &mut Scene, p: &Primitive, layers: &LayerSet, selected: bool, dark: bool) {
    if !layers.visible(p.layer()) && !matches!(p, Primitive::Macro { .. }) {
        if !matches!(p, Primitive::Text { .. }) {
            return;
        }
        if !layers.visible(p.layer()) {
            return;
        }
    }
    let rgb = color(layers, p, selected, dark);
    let stroke_w = 0.55;
    match p {
        Primitive::Line { a, b, .. } => line(scene, *a, *b, stroke_w, rgb, selected),
        Primitive::Bezier { p0, p1, p2, p3, .. } => {
            let mut prev = *p0;
            for i in 1..=24 {
                let t = i as f32 / 24.0;
                let (x, y) = bezier_point(*p0, *p1, *p2, *p3, t);
                let cur = Point::new(x.round() as i32, y.round() as i32);
                line(scene, prev, cur, stroke_w, rgb, selected);
                prev = cur;
            }
        }
        Primitive::Rect { a, b, filled, .. } => {
            let pts = [
                Point::new(a.x, a.y),
                Point::new(b.x, a.y),
                Point::new(b.x, b.y),
                Point::new(a.x, b.y),
            ];
            if *filled {
                let mut builder = Path::builder();
                builder.begin(point(pts[0].x as f32, pts[0].y as f32));
                for p in &pts[1..] {
                    builder.line_to(point(p.x as f32, p.y as f32));
                }
                builder.close();
                tessellate_path(&builder.build(), rgb, selected, scene);
            } else {
                stroke_poly(scene, &pts, true, stroke_w, rgb, selected);
            }
        }
        Primitive::Poly { pts, filled, .. } => {
            if *filled && pts.len() >= 3 {
                let mut builder = Path::builder();
                builder.begin(point(pts[0].x as f32, pts[0].y as f32));
                for p in &pts[1..] {
                    builder.line_to(point(p.x as f32, p.y as f32));
                }
                builder.close();
                tessellate_path(&builder.build(), rgb, selected, scene);
            } else {
                stroke_poly(scene, pts, true, stroke_w, rgb, selected);
            }
        }
        Primitive::Ellipse { a, b, filled, .. } => {
            add_ellipse(scene, *a, *b, *filled, stroke_w, rgb, selected);
        }
        Primitive::Connection { pos, .. } => {
            circle(
                scene,
                pos.x as f32,
                pos.y as f32,
                1.3,
                1.3,
                0.0,
                0.0,
                rgb,
                selected,
            );
        }
        Primitive::PcbTrack { a, b, width, .. } => {
            line(scene, *a, *b, *width as f32, rgb, selected);
        }
        Primitive::PcbPad {
            pos,
            dx,
            dy,
            hole,
            style,
            ..
        } => {
            let hx = *dx as f32 / 2.0;
            let hy = *dy as f32 / 2.0;
            match style {
                PadStyle::Oval => {
                    circle(
                        scene,
                        pos.x as f32,
                        pos.y as f32,
                        hx,
                        hy,
                        (*hole as f32 / (*dx).max(1) as f32).clamp(0.0, 0.85),
                        0.0,
                        rgb,
                        selected,
                    );
                }
                PadStyle::Rectangular | PadStyle::RoundedRect => {
                    let pts = [
                        Point::new(pos.x - dx / 2, pos.y - dy / 2),
                        Point::new(pos.x + dx / 2, pos.y - dy / 2),
                        Point::new(pos.x + dx / 2, pos.y + dy / 2),
                        Point::new(pos.x - dx / 2, pos.y + dy / 2),
                    ];
                    let mut builder = Path::builder();
                    builder.begin(point(pts[0].x as f32, pts[0].y as f32));
                    for p in &pts[1..] {
                        builder.line_to(point(p.x as f32, p.y as f32));
                    }
                    builder.close();
                    tessellate_path(&builder.build(), rgb, selected, scene);
                    circle(
                        scene,
                        pos.x as f32,
                        pos.y as f32,
                        *hole as f32 / 2.0,
                        *hole as f32 / 2.0,
                        0.0,
                        0.0,
                        [1.0, 1.0, 1.0],
                        false,
                    );
                }
            }
        }
        Primitive::Text { .. } => add_text(scene, p, rgb, selected),
        Primitive::Macro { .. } => {}
    }
}

pub fn tessellate_editor(ed: &Editor) -> Scene {
    tessellate_impl(ed, None, false)
}

pub fn tessellate_view(ed: &Editor, viewport: Option<(f32, f32)>) -> Scene {
    tessellate_impl(ed, viewport, ed.canvas_dark)
}

fn tessellate_impl(ed: &Editor, viewport: Option<(f32, f32)>, dark: bool) -> Scene {
    let view = viewport.map(|(w, h)| {
        let z = ed.zoom.max(0.01);
        let x0 = ((0.0 - ed.pan.0) / z).floor() as i32 - 50;
        let y0 = ((0.0 - ed.pan.1) / z).floor() as i32 - 50;
        let x1 = ((w - ed.pan.0) / z).ceil() as i32 + 50;
        let y1 = ((h - ed.pan.1) / z).ceil() as i32 + 50;
        fidocad_core::geom::Aabb {
            min: Point::new(x0, y0),
            max: Point::new(x1, y1),
        }
    });
    let mut scene = Scene::default();
    let selected = &ed.selected;
    let layers = &ed.doc.layers;
    let preview = if dark {
        [0.85, 0.55, 0.32]
    } else {
        [0.72, 0.42, 0.22]
    };
    for (i, p) in ed.doc.primitives.iter().enumerate() {
        let sel = selected.contains(&i);
        for q in fidocad_core::library::expand_primitive(p, &ed.libs) {
            if let Some(v) = &view {
                if !q.aabb().expand(30).intersects(v) {
                    continue;
                }
            }
            add_prim(&mut scene, &q, layers, sel, dark);
        }
        if sel {
            for h in p.control_points() {
                scene.handles.push(CircleInstance {
                    x: h.x as f32,
                    y: h.y as f32,
                    rx: 2.4,
                    ry: 2.4,
                    inner: 0.0,
                    stroke: 0.0,
                    r: 0.85,
                    g: 0.42,
                    b: 0.22,
                    selected: 1.0,
                });
            }
        }
    }
    add_draft(&mut scene, ed, preview);
    if let Some((a, b)) = ed.marquee_rect() {
        add_marquee(&mut scene, a, b, preview);
    }
    scene
}

fn add_ellipse(
    scene: &mut Scene,
    a: Point,
    b: Point,
    filled: bool,
    stroke_w: f32,
    rgb: [f32; 3],
    selected: bool,
) {
    let cx = (a.x + b.x) as f32 / 2.0;
    let cy = (a.y + b.y) as f32 / 2.0;
    let rx = ((a.x - b.x).abs() as f32 / 2.0).max(0.5);
    let ry = ((a.y - b.y).abs() as f32 / 2.0).max(0.5);
    if filled {
        circle(scene, cx, cy, rx, ry, 0.0, 0.0, rgb, selected);
    } else {
        circle(scene, cx, cy, rx, ry, 0.0, stroke_w, rgb, selected);
    }
}

fn add_draft(scene: &mut Scene, ed: &Editor, preview: [f32; 3]) {
    let pts = ed.draft_points();
    if pts.is_empty() {
        return;
    }
    let a = pts[0];
    let b = if pts.len() >= 2 {
        pts[pts.len() - 1]
    } else if let Some(h) = ed.hover {
        h
    } else {
        return;
    };
    match ed.draft_tool() {
        Some(Tool::Ellipse) => {
            if a != b {
                add_ellipse(scene, a, b, ed.filled, 0.55, preview, false);
            }
        }
        Some(Tool::Rect) => {
            if a != b {
                let corners = [
                    Point::new(a.x, a.y),
                    Point::new(b.x, a.y),
                    Point::new(b.x, b.y),
                    Point::new(a.x, b.y),
                ];
                if ed.filled {
                    let mut builder = Path::builder();
                    builder.begin(point(corners[0].x as f32, corners[0].y as f32));
                    for p in &corners[1..] {
                        builder.line_to(point(p.x as f32, p.y as f32));
                    }
                    builder.close();
                    tessellate_path(&builder.build(), preview, false, scene);
                } else {
                    stroke_poly(scene, &corners, true, 0.55, preview, false);
                }
            }
        }
        Some(Tool::Poly) | Some(Tool::Bezier) => {
            stroke_poly(scene, pts, false, 0.55, preview, false);
            if pts.len() >= 2 {
                // last point already in pts while dragging two-point tools; poly rubber-bands hover
            }
            if let Some(h) = ed.hover {
                if let Some(&last) = pts.last() {
                    if last != h {
                        line(scene, last, h, 0.55, preview, false);
                    }
                }
            }
        }
        _ => {
            if a != b {
                line(scene, a, b, 0.55, preview, false);
            }
        }
    }
}

fn add_marquee(scene: &mut Scene, a: Point, b: Point, rgb: [f32; 3]) {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    if x0 == x1 && y0 == y1 {
        return;
    }
    let corners = [
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ];
    for i in 0..4 {
        dash_line(scene, corners[i], corners[(i + 1) % 4], 0.7, rgb, 6.0, 4.0);
    }
}

fn dash_line(
    scene: &mut Scene,
    a: Point,
    b: Point,
    w: f32,
    rgb: [f32; 3],
    dash: f32,
    gap: f32,
) {
    let dx = (b.x - a.x) as f32;
    let dy = (b.y - a.y) as f32;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let mut t = 0.0;
    let mut on = true;
    while t < len {
        let seg = if on { dash } else { gap };
        let t1 = (t + seg).min(len);
        if on {
            line(
                scene,
                Point::new(
                    (a.x as f32 + ux * t).round() as i32,
                    (a.y as f32 + uy * t).round() as i32,
                ),
                Point::new(
                    (a.x as f32 + ux * t1).round() as i32,
                    (a.y as f32 + uy * t1).round() as i32,
                ),
                w,
                rgb,
                false,
            );
        }
        t = t1;
        on = !on;
    }
}

pub fn scene_to_svg(scene: &Scene, w: f32, h: f32, zoom: f32, pan: (f32, f32)) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#
    ));
    s.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
    let tx = |x: f32, y: f32| (x * zoom + pan.0, y * zoom + pan.1);
    for l in &scene.lines {
        let (x1, y1) = tx(l.ax, l.ay);
        let (x2, y2) = tx(l.bx, l.by);
        s.push_str(&format!(
            r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="rgb({},{},{})" stroke-width="{}" stroke-linecap="round"/>"#,
            (l.r * 255.0) as u8,
            (l.g * 255.0) as u8,
            (l.b * 255.0) as u8,
            (l.width * zoom).max(0.6),
        ));
    }
    s.push_str("</svg>");
    s
}
