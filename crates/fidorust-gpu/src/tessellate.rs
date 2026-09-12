//! Tessellate flattened primitives into GPU-friendly batches (world LU coordinates).

use fidorust_core::geom::{bezier_point, Point};
use fidorust_core::layers::{LayerId, LayerSet};
use fidorust_core::library::LibrarySet;
use fidorust_core::primitive::{
    Bezier, ComponentRef, Connection, Ellipse, Line, PcbPad, PcbTrack, Poly, Primitive, Rect, Text,
    BEZIER_SEGMENTS_DRAW, ITALIC_SHEAR, STYLE_ITALIC, STYLE_MIRRORED,
};
use fidorust_core::Editor;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::FillRule;

use crate::draft::{add_draft, DraftParams};
use crate::scene::DEFAULT_STROKE_W;
use crate::shapes::{path_ellipse, path_rect, path_rounded_rect, rect_corners};
use crate::theme::Rgb;

pub use crate::scene::{
    CircleInstance, FillVertexGpu, HandleInstance, LineInstance, PadHole, Scene,
};
pub use crate::svg::{
    export_svg, scene_to_cursor_svg, scene_to_svg, scene_to_thumb_svg, CursorSvg,
};

const PCB_TRACK_CAP_SEGS: u32 = 24;

trait Tessellate {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, stroke_w: f32);
}

impl Tessellate for Line {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, stroke_w: f32) {
        scene.push_line(self.a, self.b, stroke_w, rgb, selected);
    }
}

impl Tessellate for Bezier {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, stroke_w: f32) {
        let (x0, y0) = (self.p0.x as f32, self.p0.y as f32);
        let mut prev_x = x0;
        let mut prev_y = y0;
        for i in 1..=BEZIER_SEGMENTS_DRAW {
            let t = i as f32 / BEZIER_SEGMENTS_DRAW as f32;
            let (x, y) = bezier_point(self.p0, self.p1, self.p2, self.p3, t);
            scene.push_line_f(prev_x, prev_y, x, y, stroke_w, rgb, selected);
            prev_x = x;
            prev_y = y;
        }
    }
}

impl Tessellate for Rect {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, stroke_w: f32) {
        let pts = rect_corners(self.a, self.b);
        if self.filled {
            scene.fill_polygon(&pts, rgb, selected);
        } else {
            scene.stroke_poly(&pts, true, stroke_w, rgb, selected);
        }
    }
}

impl Tessellate for Poly {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, stroke_w: f32) {
        if self.filled && self.pts.len() >= 3 {
            scene.fill_polygon(&self.pts, rgb, selected);
        } else {
            scene.stroke_poly(&self.pts, true, stroke_w, rgb, selected);
        }
    }
}

impl Tessellate for Ellipse {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, stroke_w: f32) {
        scene.push_ellipse(self.a, self.b, self.filled, stroke_w, rgb, selected);
    }
}

impl Tessellate for Connection {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, _stroke_w: f32) {
        scene.push_circle(
            self.pos.x as f32,
            self.pos.y as f32,
            1.3,
            1.3,
            0.0,
            0.0,
            rgb,
            selected,
        );
    }
}

impl Tessellate for PcbTrack {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, _stroke_w: f32) {
        add_pcb_track(scene, self.a, self.b, self.width, rgb, selected);
    }
}

impl Tessellate for PcbPad {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, _stroke_w: f32) {
        add_pcb_pad(
            scene, self.pos, self.dx, self.dy, self.hole, self.style, rgb, selected,
        );
    }
}

impl Tessellate for Text {
    fn tessellate(&self, scene: &mut Scene, rgb: [f32; 4], selected: bool, _stroke_w: f32) {
        let h = (self.sy as f32).max(2.0);
        let wch = (self.sx as f32).max(1.5);
        let mirrored = self.style & STYLE_MIRRORED != 0;
        let italic = self.style & STYLE_ITALIC != 0;
        let layout = fidorust_core::TextLayout::from_params(
            self.sx, self.sy, self.angle, self.style, &self.text,
        );
        let mut x_off = 0.0f32;
        let sel_f = Scene::flag(selected);
        for ch in self.text.chars() {
            for v in crate::font::glyph_triangles(&self.font, ch) {
                let mut ax = v[0] * wch + x_off;
                let ay = v[1] * h;
                if italic {
                    ax += (1.0 - v[1]) * wch * ITALIC_SHEAR;
                }
                if mirrored {
                    ax = -ax;
                }
                let ra = layout.map(ax, ay);
                scene.fills.push(FillVertexGpu {
                    x: self.pos.x as f32 + ra.0,
                    y: self.pos.y as f32 + ra.1,
                    r: rgb[0],
                    g: rgb[1],
                    b: rgb[2],
                    a: rgb[3],
                    selected: sel_f,
                });
            }
            x_off += wch;
        }
    }
}

impl Tessellate for ComponentRef {
    fn tessellate(&self, _scene: &mut Scene, _rgb: [f32; 4], _selected: bool, _stroke_w: f32) {}
}

fn add_pcb_track(scene: &mut Scene, a: Point, b: Point, width: i32, rgb: [f32; 4], selected: bool) {
    let ax = a.x as f32;
    let ay = a.y as f32;
    let bx = b.x as f32;
    let by = b.y as f32;
    let w = width as f32;
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        scene.push_circle(ax, ay, w * 0.5, w * 0.5, 0.0, 0.0, rgb, selected);
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
    scene.fill_path(&builder.build(), FillRule::NonZero, rgb, selected);
}

#[allow(clippy::too_many_arguments)]
fn add_pcb_pad(
    scene: &mut Scene,
    pos: Point,
    dx: i32,
    dy: i32,
    hole: i32,
    style: fidorust_core::PadStyle,
    rgb: [f32; 4],
    selected: bool,
) {
    let cx = pos.x as f32;
    let cy = pos.y as f32;
    let hx = dx as f32 / 2.0;
    let hy = dy as f32 / 2.0;
    let hole_r = hole as f32 / 2.0;
    match style {
        fidorust_core::PadStyle::Oval => {
            let inner = if hole_r > 0.001 {
                (hole_r / hx.max(hy).max(0.001)).clamp(0.0, 0.98)
            } else {
                0.0
            };
            scene.push_circle(cx, cy, hx.max(0.5), hy.max(0.5), inner, 0.0, rgb, selected);
        }
        fidorust_core::PadStyle::Rectangular => {
            let mut builder = Path::builder();
            path_rect(&mut builder, cx, cy, hx, hy);
            if hole_r > 0.001 {
                path_ellipse(&mut builder, cx, cy, hole_r, hole_r, -1.0);
            }
            scene.fill_path(&builder.build(), FillRule::EvenOdd, rgb, selected);
        }
        fidorust_core::PadStyle::RoundedRect => {
            let mut builder = Path::builder();
            path_rounded_rect(&mut builder, cx, cy, hx, hy, hx * 0.5, hy * 0.5);
            if hole_r > 0.001 {
                path_ellipse(&mut builder, cx, cy, hole_r, hole_r, -1.0);
            }
            scene.fill_path(&builder.build(), FillRule::EvenOdd, rgb, selected);
        }
    }
    if hole_r > 0.001 {
        scene.pad_holes.push(PadHole {
            x: cx,
            y: cy,
            r: hole_r,
        });
    }
}

fn color(layers: &LayerSet, p: &Primitive, selected: bool, hovered: bool) -> [f32; 4] {
    if selected {
        return Rgb::SELECTION.rgba(1.0);
    }
    let base = Rgb::from_rgba_u8(layers.color(p.layer()));
    if hovered {
        Rgb::mix_white(base, Rgb::HOVER_LIGHTEN)
    } else {
        base
    }
}

fn add_prim(
    scene: &mut Scene,
    p: &Primitive,
    layers: &LayerSet,
    selected: bool,
    hovered: bool,
    stroke_w: f32,
) {
    if !layers.visible(p.layer()) {
        return;
    }
    let rgb = color(layers, p, selected, hovered);
    fidorust_core::dispatch_primitive!(p, |q| q.tessellate(scene, rgb, selected, stroke_w));
}

fn group_by_layer<T>(
    n: usize,
    items: impl IntoIterator<Item = T>,
    layer_of: impl Fn(&T) -> usize,
) -> Vec<Vec<T>> {
    let mut by_layer: Vec<Vec<T>> = (0..n).map(|_| Vec::new()).collect();
    for item in items {
        let i = layer_of(&item);
        if i < n {
            by_layer[i].push(item);
        }
    }
    by_layer
}

pub fn tessellate_primitives(prims: &[Primitive], layers: &LayerSet) -> Scene {
    tessellate_primitives_with_stroke(prims, layers, DEFAULT_STROKE_W)
}

fn tessellate_primitives_with_stroke(
    prims: &[Primitive],
    layers: &LayerSet,
    stroke_w: f32,
) -> Scene {
    let mut scene = Scene::default();
    let n = layers.len();
    let by_layer = group_by_layer(n, prims.iter(), |p| p.layer().index());
    for bucket in by_layer {
        for p in bucket {
            add_prim(&mut scene, p, layers, false, false, stroke_w);
        }
        scene.mark_layer_end();
    }
    scene
}

struct TessellateInput<'a> {
    primitives: &'a [Primitive],
    layers: &'a LayerSet,
    libs: &'a LibrarySet,
    selected: &'a [usize],
    hover: Option<usize>,
    editing_text: Option<usize>,
    hide_component_origin: bool,
    stroke_w: f32,
    zoom: f32,
    pan: (f32, f32),
    layer: LayerId,
    viewport: Option<(f32, f32)>,
    dark: bool,
    pending: Vec<Primitive>,
    marquee: Option<(f32, f32, f32, f32)>,
}

impl<'a> TessellateInput<'a> {
    fn from_editor(ed: &'a Editor, viewport: Option<(f32, f32)>, dark: bool) -> Self {
        Self {
            primitives: &ed.doc().primitives,
            layers: &ed.doc().layers,
            libs: ed.libs(),
            selected: ed.selected(),
            hover: ed.hover_index(),
            editing_text: ed.editing_text(),
            hide_component_origin: ed.hide_component_origin(),
            stroke_w: ed.doc().stroke_width(),
            zoom: ed.zoom(),
            pan: ed.pan(),
            layer: ed.layer(),
            viewport,
            dark,
            pending: {
                let mut pending = ed.pending_component_preview();
                pending.extend(ed.duplicate_drag_preview());
                pending
            },
            marquee: ed.marquee_screen_rect(),
        }
    }
}

pub fn tessellate_editor(ed: &Editor) -> Scene {
    tessellate_impl(
        TessellateInput::from_editor(ed, None, false),
        &DraftParams::from_editor(ed),
    )
}

/// Flattened document geometry for file export: no draft, selection, handles, or pending components.
pub fn tessellate_export(ed: &Editor, layers: &LayerSet) -> Scene {
    let expanded: Vec<Primitive> = ed
        .doc()
        .primitives
        .iter()
        .flat_map(|p| fidorust_core::library::expand_primitive(p, ed.libs()))
        .collect();
    tessellate_primitives_with_stroke(&expanded, layers, ed.doc().stroke_width())
}

pub fn tessellate_view(ed: &Editor, viewport: Option<(f32, f32)>) -> Scene {
    tessellate_impl(
        TessellateInput::from_editor(ed, viewport, ed.canvas_dark()),
        &DraftParams::from_editor(ed),
    )
}

fn tessellate_impl(input: TessellateInput<'_>, draft: &DraftParams<'_>) -> Scene {
    let view = input.viewport.map(|(w, h)| {
        let z = input.zoom.max(0.01);
        let x0 = ((0.0 - input.pan.0) / z).floor() as i32 - 50;
        let y0 = ((0.0 - input.pan.1) / z).floor() as i32 - 50;
        let x1 = ((w - input.pan.0) / z).ceil() as i32 + 50;
        let y1 = ((h - input.pan.1) / z).ceil() as i32 + 50;
        fidorust_core::geom::Aabb {
            min: Point::new(x0, y0),
            max: Point::new(x1, y1),
        }
    });
    let mut scene = Scene::default();
    let layers = input.layers;
    let preview = Rgb::preview(input.dark).rgba(1.0);
    let expanded: Vec<(bool, bool, Primitive)> = input
        .primitives
        .iter()
        .enumerate()
        .filter(|(i, _)| input.editing_text != Some(*i))
        .flat_map(|(i, p)| {
            let sel = input.selected.contains(&i);
            let hov = !sel && input.hover == Some(i);
            fidorust_core::library::expand_primitive(p, input.libs)
                .into_iter()
                .filter(|q| {
                    view.as_ref()
                        .map(|v| q.aabb().expand(30).intersects(v))
                        .unwrap_or(true)
                })
                .map(move |q| (sel, hov, q))
        })
        .collect();
    let n = layers.len();
    let mut by_layer = group_by_layer(n, expanded, |(_, _, q)| q.layer().index());
    for q in input.pending {
        let i = q.layer().index();
        if i < n {
            by_layer[i].push((false, false, q));
        }
    }
    for (li, bucket) in by_layer.into_iter().enumerate() {
        for (sel, hov, q) in &bucket {
            add_prim(&mut scene, q, layers, *sel, *hov, input.stroke_w);
        }
        if input.layer.index() == li {
            add_draft(&mut scene, draft, preview, input.stroke_w);
        }
        scene.mark_layer_end();
    }
    for (i, p) in input.primitives.iter().enumerate() {
        if !input.selected.contains(&i) {
            continue;
        }
        if input.hide_component_origin && p.is_component() {
            continue;
        }
        for h in p.control_points() {
            scene.handles.push(HandleInstance {
                x: h.x as f32,
                y: h.y as f32,
            });
        }
    }
    if let Some((x0, y0, x1, y1)) = input.marquee {
        scene.marquee = Some([x0, y0, x1, y1]);
        scene.marquee_color = [preview[0], preview[1], preview[2]];
    }
    scene
}
