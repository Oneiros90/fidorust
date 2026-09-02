//! Tessellate flattened primitives into GPU-friendly batches (world LU coordinates).

use fidocad_core::geom::{bezier_point, Point};
use fidocad_core::layers::{LayerSet, LAYER_COUNT};
use fidocad_core::primitive::{PadStyle, Primitive};
use fidocad_core::{Editor, Tool};
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, VertexBuffers,
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

/// Drill hole of a PCB pad (world LU). Punched after the pad's layer is drawn.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PadHole {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

#[derive(Default)]
pub struct Scene {
    pub lines: Vec<LineInstance>,
    pub fills: Vec<FillVertexGpu>,
    pub circles: Vec<CircleInstance>,
    pub handles: Vec<CircleInstance>,
    pub marquee: Option<[f32; 4]>,
    pub marquee_color: [f32; 3],
    pub pad_holes: Vec<PadHole>,
    /// Exclusive end of each layer's slice in `fills` / `lines` / `circles` / `pad_holes`.
    pub layer_fill_end: [u32; LAYER_COUNT],
    pub layer_line_end: [u32; LAYER_COUNT],
    pub layer_circ_end: [u32; LAYER_COUNT],
    pub layer_hole_end: [u32; LAYER_COUNT],
}

fn mark_layer_end(scene: &mut Scene, i: usize) {
    scene.layer_fill_end[i] = scene.fills.len() as u32;
    scene.layer_line_end[i] = scene.lines.len() as u32;
    scene.layer_circ_end[i] = scene.circles.len() as u32;
    scene.layer_hole_end[i] = scene.pad_holes.len() as u32;
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
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
    ]
}

const DEFAULT_STROKE_W: f32 = 0.25;
const PCB_TRACK_CAP_SEGS: u32 = 24;
const PCB_PAD_CORNER_SEGS: u32 = 12;

fn line(scene: &mut Scene, a: Point, b: Point, w: f32, rgb: [f32; 3], selected: bool) {
    line_f(
        scene,
        a.x as f32,
        a.y as f32,
        b.x as f32,
        b.y as f32,
        w,
        rgb,
        selected,
    );
}

fn line_f(
    scene: &mut Scene,
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
    w: f32,
    rgb: [f32; 3],
    selected: bool,
) {
    scene.lines.push(LineInstance {
        ax,
        ay,
        bx,
        by,
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
    tessellate_path_rule(path, FillRule::NonZero, rgb, selected, scene);
}

fn tessellate_path_even_odd(path: &Path, rgb: [f32; 3], selected: bool, scene: &mut Scene) {
    tessellate_path_rule(path, FillRule::EvenOdd, rgb, selected, scene);
}

fn tessellate_path_rule(
    path: &Path,
    fill_rule: FillRule,
    rgb: [f32; 3],
    selected: bool,
    scene: &mut Scene,
) {
    let mut buffers: VertexBuffers<FillVertexGpu, u16> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let _ = tess.tessellate_path(
        path,
        &FillOptions::default().with_fill_rule(fill_rule),
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

fn add_pcb_track(
    scene: &mut Scene,
    a: Point,
    b: Point,
    width: i32,
    rgb: [f32; 3],
    selected: bool,
) {
    let ax = a.x as f32;
    let ay = a.y as f32;
    let bx = b.x as f32;
    let by = b.y as f32;
    let w = width as f32;
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        circle(scene, ax, ay, w * 0.5, w * 0.5, 0.0, 0.0, rgb, selected);
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let nx = -uy;
    let ny = ux;
    let hw = w * 0.5;
    let mut builder = Path::builder();
    builder.begin(point(ax + nx * hw, ay + ny * hw));
    builder.line_to(point(bx + nx * hw, by + ny * hw));
    for i in 1..=PCB_TRACK_CAP_SEGS {
        let t = std::f32::consts::PI * i as f32 / PCB_TRACK_CAP_SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(
            bx + nx * hw * ct + ux * hw * st,
            by + ny * hw * ct + uy * hw * st,
        ));
    }
    builder.line_to(point(ax - nx * hw, ay - ny * hw));
    for i in 1..=PCB_TRACK_CAP_SEGS {
        let t = std::f32::consts::PI * i as f32 / PCB_TRACK_CAP_SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(
            ax - nx * hw * ct - ux * hw * st,
            ay - ny * hw * ct - uy * hw * st,
        ));
    }
    builder.close();
    tessellate_path(&builder.build(), rgb, selected, scene);
}

fn path_ellipse(builder: &mut lyon::path::Builder, cx: f32, cy: f32, rx: f32, ry: f32) {
    const SEGS: u32 = 64;
    builder.begin(point(cx + rx, cy));
    for i in 1..=SEGS {
        let t = std::f32::consts::TAU * i as f32 / SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(cx + rx * ct, cy + ry * st));
    }
    builder.close();
}

fn path_rect(builder: &mut lyon::path::Builder, cx: f32, cy: f32, hx: f32, hy: f32) {
    builder.begin(point(cx - hx, cy - hy));
    builder.line_to(point(cx + hx, cy - hy));
    builder.line_to(point(cx + hx, cy + hy));
    builder.line_to(point(cx - hx, cy + hy));
    builder.close();
}

fn path_rounded_rect(
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

fn path_circle_hole(builder: &mut lyon::path::Builder, cx: f32, cy: f32, r: f32) {
    const SEGS: u32 = 64;
    builder.begin(point(cx + r, cy));
    for i in 1..=SEGS {
        let t = std::f32::consts::TAU - std::f32::consts::TAU * i as f32 / SEGS as f32;
        let (st, ct) = t.sin_cos();
        builder.line_to(point(cx + r * ct, cy + r * st));
    }
    builder.close();
}

fn add_pcb_pad(
    scene: &mut Scene,
    pos: Point,
    dx: i32,
    dy: i32,
    hole: i32,
    style: PadStyle,
    rgb: [f32; 3],
    selected: bool,
) {
    let cx = pos.x as f32;
    let cy = pos.y as f32;
    let hx = dx as f32 / 2.0;
    let hy = dy as f32 / 2.0;
    let hole_r = hole as f32 / 2.0;
    let mut builder = Path::builder();
    match style {
        PadStyle::Oval => path_ellipse(&mut builder, cx, cy, hx, hy),
        PadStyle::Rectangular => path_rect(&mut builder, cx, cy, hx, hy),
        PadStyle::RoundedRect => {
            path_rounded_rect(&mut builder, cx, cy, hx, hy, hx * 0.5, hy * 0.5)
        }
    }
    if hole_r > 0.001 {
        path_circle_hole(&mut builder, cx, cy, hole_r);
        scene.pad_holes.push(PadHole {
            x: cx,
            y: cy,
            r: hole_r,
        });
    }
    tessellate_path_even_odd(&builder.build(), rgb, selected, scene);
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
    let italic = style & 2 != 0;
    let rad = (*angle as f32).to_radians();
    let (sin, cos) = rad.sin_cos();
    let mut x_off = 0.0f32;
    let sel_f = if sel { 1.0 } else { 0.0 };
    for ch in text.chars() {
        for v in crate::font::glyph_triangles(ch) {
            let mut ax = v[0] * wch + x_off;
            let ay = v[1] * h;
            if italic {
                ax += (1.0 - v[1]) * wch * 0.22;
            }
            if mirrored {
                ax = -ax;
            }
            let ra = rot(ax, ay, cos, sin);
            scene.fills.push(FillVertexGpu {
                x: pos.x as f32 + ra.0,
                y: pos.y as f32 + ra.1,
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                selected: sel_f,
            });
        }
        x_off += wch;
    }
}

fn rot(x: f32, y: f32, cos: f32, sin: f32) -> (f32, f32) {
    (x * cos - y * sin, x * sin + y * cos)
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
    let stroke_w = DEFAULT_STROKE_W;
    match p {
        Primitive::Line { a, b, .. } => line(scene, *a, *b, stroke_w, rgb, selected),
        Primitive::Bezier { p0, p1, p2, p3, .. } => {
            let (x0, y0) = (p0.x as f32, p0.y as f32);
            let mut prev_x = x0;
            let mut prev_y = y0;
            const SEGMENTS: u32 = 48;
            for i in 1..=SEGMENTS {
                let t = i as f32 / SEGMENTS as f32;
                let (x, y) = bezier_point(*p0, *p1, *p2, *p3, t);
                line_f(scene, prev_x, prev_y, x, y, stroke_w, rgb, selected);
                prev_x = x;
                prev_y = y;
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
            add_pcb_track(scene, *a, *b, *width, rgb, selected);
        }
        Primitive::PcbPad {
            pos,
            dx,
            dy,
            hole,
            style,
            ..
        } => {
            add_pcb_pad(scene, *pos, *dx, *dy, *hole, *style, rgb, selected);
        }
        Primitive::Text { .. } => add_text(scene, p, rgb, selected),
        Primitive::Macro { .. } => {}
    }
}

pub fn tessellate_primitives(prims: &[Primitive], layers: &LayerSet, dark: bool) -> Scene {
    let mut scene = Scene::default();
    for i in 0..LAYER_COUNT {
        for p in prims {
            if p.layer().index() == i {
                add_prim(&mut scene, p, layers, false, dark);
            }
        }
        mark_layer_end(&mut scene, i);
    }
    scene
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
    let expanded: Vec<(bool, Primitive)> = ed
        .doc
        .primitives
        .iter()
        .enumerate()
        .filter(|(i, _)| ed.editing_text != Some(*i))
        .flat_map(|(i, p)| {
            let sel = selected.contains(&i);
            fidocad_core::library::expand_primitive(p, &ed.libs)
                .into_iter()
                .filter(|q| {
                    view.as_ref()
                        .map(|v| q.aabb().expand(30).intersects(v))
                        .unwrap_or(true)
                })
                .map(move |q| (sel, q))
        })
        .collect();
    let pending = ed.pending_macro_preview();
    for li in 0..LAYER_COUNT {
        for (sel, q) in &expanded {
            if q.layer().index() == li {
                add_prim(&mut scene, q, layers, *sel, dark);
            }
        }
        for q in &pending {
            if q.layer().index() == li {
                add_prim(&mut scene, q, layers, false, dark);
            }
        }
        if ed.layer.index() == li {
            add_draft(&mut scene, ed, preview);
        }
        mark_layer_end(&mut scene, li);
    }
    for (i, p) in ed.doc.primitives.iter().enumerate() {
        if !selected.contains(&i) {
            continue;
        }
        if ed.hide_macro_origin && matches!(p, Primitive::Macro { .. }) {
            continue;
        }
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
    if let Some((x0, y0, x1, y1)) = ed.marquee_screen_rect() {
        scene.marquee = Some([x0, y0, x1, y1]);
        scene.marquee_color = preview;
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
                add_ellipse(scene, a, b, ed.filled, DEFAULT_STROKE_W, preview, false);
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
                    stroke_poly(scene, &corners, true, DEFAULT_STROKE_W, preview, false);
                }
            }
        }
        Some(Tool::Poly) | Some(Tool::Bezier) => {
            stroke_poly(scene, pts, false, DEFAULT_STROKE_W, preview, false);
            if pts.len() >= 2 {
                // last point already in pts while dragging two-point tools; poly rubber-bands hover
            }
            if let Some(h) = ed.hover {
                if let Some(&last) = pts.last() {
                    if last != h {
                        line(scene, last, h, DEFAULT_STROKE_W, preview, false);
                    }
                }
            }
        }
        _ => {
            if a != b {
                line(scene, a, b, DEFAULT_STROKE_W, preview, false);
            }
        }
    }
}

pub fn scene_to_svg(scene: &Scene, w: f32, h: f32, zoom: f32, pan: (f32, f32)) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#
    ));
    s.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
    let tx = |x: f32, y: f32| (x * zoom + pan.0, y * zoom + pan.1);
    for tri in scene.fills.chunks(3) {
        if tri.len() != 3 {
            continue;
        }
        let (x1, y1) = tx(tri[0].x, tri[0].y);
        let (x2, y2) = tx(tri[1].x, tri[1].y);
        let (x3, y3) = tx(tri[2].x, tri[2].y);
        s.push_str(&format!(
            r#"<polygon points="{x1:.2},{y1:.2} {x2:.2},{y2:.2} {x3:.2},{y3:.2}" fill="rgb({},{},{})"/>"#,
            (tri[0].r * 255.0) as u8,
            (tri[0].g * 255.0) as u8,
            (tri[0].b * 255.0) as u8,
        ));
    }
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
    for tri in scene.fills.chunks(3) {
        if tri.len() != 3 {
            continue;
        }
        let (x1, y1) = tx(tri[0].x, tri[0].y);
        let (x2, y2) = tx(tri[1].x, tri[1].y);
        let (x3, y3) = tx(tri[2].x, tri[2].y);
        out.push_str(&format!(
            r#"<polygon points="{x1:.2},{y1:.2} {x2:.2},{y2:.2} {x3:.2},{y3:.2}" fill="rgb({},{},{})"/>"#,
            (tri[0].r * 255.0) as u8,
            (tri[0].g * 255.0) as u8,
            (tri[0].b * 255.0) as u8,
        ));
    }
    for l in &scene.lines {
        let (x1, y1) = tx(l.ax, l.ay);
        let (x2, y2) = tx(l.bx, l.by);
        out.push_str(&format!(
            r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="rgb({},{},{})" stroke-width="{:.2}" stroke-linecap="round"/>"#,
            (l.r * 255.0) as u8,
            (l.g * 255.0) as u8,
            (l.b * 255.0) as u8,
            (l.width * scale).max(if scale < 1.5 { 0.175 } else { 0.575 }),
        ));
    }
    for c in &scene.circles {
        let (cx, cy) = tx(c.x, c.y);
        let rx = (c.rx * scale).max(0.4);
        let ry = (c.ry * scale).max(0.4);
        let stroke = format!(
            "rgb({},{},{})",
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8
        );
        if c.stroke > 0.001 {
            out.push_str(&format!(
                r#"<ellipse cx="{cx:.2}" cy="{cy:.2}" rx="{rx:.2}" ry="{ry:.2}" stroke="{stroke}" stroke-width="{:.2}"/>"#,
                (c.stroke * scale).max(if scale < 1.5 { 0.175 } else { 0.575 }),
            ));
        } else {
            out.push_str(&format!(
                r#"<ellipse cx="{cx:.2}" cy="{cy:.2}" rx="{rx:.2}" ry="{ry:.2}" fill="{stroke}"/>"#
            ));
        }
    }
}
