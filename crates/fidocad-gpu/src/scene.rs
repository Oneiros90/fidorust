//! GPU batches (world LU coordinates).

use fidocad_core::geom::Point;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, VertexBuffers,
};

pub const DEFAULT_STROKE_W: f32 = 0.25;

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
    pub a: f32,
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
    pub a: f32,
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
    pub a: f32,
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
    pub layer_fill_end: Vec<u32>,
    pub layer_line_end: Vec<u32>,
    pub layer_circ_end: Vec<u32>,
    pub layer_hole_end: Vec<u32>,
}

impl Scene {
    pub fn flag(sel: bool) -> f32 {
        if sel {
            1.0
        } else {
            0.0
        }
    }

    pub fn mark_layer_end(&mut self) {
        self.layer_fill_end.push(self.fills.len() as u32);
        self.layer_line_end.push(self.lines.len() as u32);
        self.layer_circ_end.push(self.circles.len() as u32);
        self.layer_hole_end.push(self.pad_holes.len() as u32);
    }

    pub fn layer_count(&self) -> usize {
        self.layer_fill_end
            .len()
            .max(self.layer_line_end.len())
            .max(self.layer_circ_end.len())
            .max(self.layer_hole_end.len())
    }

    pub fn layer_items<'a, T>(ends: &[u32], items: &'a [T], i: usize) -> &'a [T] {
        let start = if i == 0 {
            0
        } else {
            ends.get(i - 1).copied().unwrap_or(0) as usize
        };
        let end = ends.get(i).copied().unwrap_or(start as u32) as usize;
        let end = end.min(items.len());
        let start = start.min(end);
        &items[start..end]
    }

    pub fn push_line(&mut self, a: Point, b: Point, w: f32, rgb: [f32; 4], selected: bool) {
        self.push_line_f(
            a.x as f32, a.y as f32, b.x as f32, b.y as f32, w, rgb, selected,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push_line_f(
        &mut self,
        ax: f32,
        ay: f32,
        bx: f32,
        by: f32,
        w: f32,
        rgb: [f32; 4],
        selected: bool,
    ) {
        self.lines.push(LineInstance {
            ax,
            ay,
            bx,
            by,
            width: w,
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: rgb[3],
            selected: Self::flag(selected),
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push_circle(
        &mut self,
        x: f32,
        y: f32,
        rx: f32,
        ry: f32,
        inner: f32,
        stroke: f32,
        rgb: [f32; 4],
        selected: bool,
    ) {
        self.circles.push(CircleInstance {
            x,
            y,
            rx,
            ry,
            inner,
            stroke,
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: rgb[3],
            selected: Self::flag(selected),
        });
    }

    pub fn push_ellipse(
        &mut self,
        a: Point,
        b: Point,
        filled: bool,
        stroke_w: f32,
        rgb: [f32; 4],
        selected: bool,
    ) {
        let cx = (a.x + b.x) as f32 / 2.0;
        let cy = (a.y + b.y) as f32 / 2.0;
        let rx = ((a.x - b.x).abs() as f32 / 2.0).max(0.5);
        let ry = ((a.y - b.y).abs() as f32 / 2.0).max(0.5);
        if filled {
            self.push_circle(cx, cy, rx, ry, 0.0, 0.0, rgb, selected);
        } else {
            self.push_circle(cx, cy, rx, ry, 0.0, stroke_w, rgb, selected);
        }
    }

    pub fn stroke_poly(&mut self, pts: &[Point], closed: bool, w: f32, rgb: [f32; 4], sel: bool) {
        for wdw in pts.windows(2) {
            self.push_line(wdw[0], wdw[1], w, rgb, sel);
        }
        if closed && pts.len() > 2 {
            self.push_line(*pts.last().unwrap(), pts[0], w, rgb, sel);
        }
    }

    pub fn fill_polygon(&mut self, pts: &[Point], rgb: [f32; 4], selected: bool) {
        let mut builder = Path::builder();
        builder.begin(point(pts[0].x as f32, pts[0].y as f32));
        for p in &pts[1..] {
            builder.line_to(point(p.x as f32, p.y as f32));
        }
        builder.close();
        self.fill_path(&builder.build(), FillRule::NonZero, rgb, selected);
    }

    pub fn fill_path(&mut self, path: &Path, fill_rule: FillRule, rgb: [f32; 4], selected: bool) {
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
                a: rgb[3],
                selected: Self::flag(selected),
            }),
        );
        for tri in buffers.indices.chunks(3) {
            if tri.len() == 3 {
                for &i in tri {
                    if let Some(v) = buffers.vertices.get(i as usize) {
                        self.fills.push(*v);
                    }
                }
            }
        }
    }
}
