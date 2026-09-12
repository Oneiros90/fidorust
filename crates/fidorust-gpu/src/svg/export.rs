//! Native-primitive SVG export (file export path).

use fidorust_core::geom::Aabb;
use fidorust_core::layers::LayerSet;
use fidorust_core::library::{expand_primitive, LibrarySet};
use fidorust_core::primitive::{
    Bezier, Connection, Ellipse, Line, PadStyle, PcbPad, PcbTrack, Poly, Primitive, Rect, Text,
    DEFAULT_FONT, STYLE_BOLD, STYLE_ITALIC, STYLE_MIRRORED, STYLE_UNDERLINE,
};

use crate::scene::PadHole;
use crate::theme::css_color;

const EMPTY_EXPORT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1" viewBox="0 0 1 1" fill="none"></svg>"#;

/// Full-sheet export in world LU from FidoCAD primitives (not GPU tessellation).
pub fn export_svg(
    prims: &[Primitive],
    layers: &LayerSet,
    libs: &LibrarySet,
    margin: f32,
    stroke_w: f32,
) -> String {
    let margin = margin.max(0.0);
    let expanded: Vec<Primitive> = prims
        .iter()
        .flat_map(|p| expand_primitive(p, libs))
        .filter(|p| !p.is_component() && layers.visible(p.layer()))
        .collect();
    let Some((minx, miny, maxx, maxy)) = prims_bounds(&expanded) else {
        return EMPTY_EXPORT.into();
    };
    let x0 = minx - margin;
    let y0 = miny - margin;
    let w = (maxx - minx + 2.0 * margin).max(1.0);
    let h = (maxy - miny + 2.0 * margin).max(1.0);
    let n = layers.len().max(1);
    let mut by_layer: Vec<Vec<Primitive>> = (0..n).map(|_| Vec::new()).collect();
    let mut holes: Vec<Vec<PadHole>> = (0..n).map(|_| Vec::new()).collect();
    for p in expanded {
        let i = p.layer().index();
        if i >= n {
            continue;
        }
        if let Primitive::PcbPad(pad) = &p {
            if let Some(hole) = pad_hole(pad) {
                holes[i].push(hole);
            }
        }
        by_layer[i].push(p);
    }
    let mut out = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.2}" height="{h:.2}" viewBox="{x0:.2} {y0:.2} {w:.2} {h:.2}" fill="none">"#
    );
    write_export_hole_defs(&mut out, &holes, x0, y0, w, h);
    write_prim_layers(
        &mut out,
        &by_layer,
        &holes,
        layers,
        n as isize - 1,
        stroke_w,
    );
    out.push_str("</svg>");
    out
}

fn prims_bounds(prims: &[Primitive]) -> Option<(f32, f32, f32, f32)> {
    let mut bb = Aabb::empty();
    for p in prims {
        bb.include_aabb(&p.aabb());
    }
    if bb.is_empty() {
        None
    } else {
        Some((
            bb.min.x as f32,
            bb.min.y as f32,
            bb.max.x as f32,
            bb.max.y as f32,
        ))
    }
}

fn pad_hole(pad: &PcbPad) -> Option<PadHole> {
    let r = pad.hole as f32 / 2.0;
    (r > 0.001).then_some(PadHole {
        x: pad.pos.x as f32,
        y: pad.pos.y as f32,
        r,
    })
}

fn rgb_u8(c: [u8; 4]) -> String {
    css_color(
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    )
}

fn write_export_hole_defs(
    out: &mut String,
    holes: &[Vec<PadHole>],
    x0: f32,
    y0: f32,
    w: f32,
    h: f32,
) {
    if holes.iter().all(|h| h.is_empty()) {
        return;
    }
    out.push_str("<defs>");
    for (i, layer_holes) in holes.iter().enumerate() {
        if layer_holes.is_empty() {
            continue;
        }
        out.push_str(&format!(
            r#"<mask id="export-h{i}" maskUnits="userSpaceOnUse" x="{x0:.2}" y="{y0:.2}" width="{w:.2}" height="{h:.2}">"#
        ));
        out.push_str(&format!(
            r#"<rect x="{x0:.2}" y="{y0:.2}" width="{w:.2}" height="{h:.2}" fill="white"/>"#
        ));
        for hole in layer_holes {
            write_hole_circle(out, hole, "black");
        }
        out.push_str("</mask>");
    }
    out.push_str("</defs>");
}

fn write_prim_layers(
    out: &mut String,
    by_layer: &[Vec<Primitive>],
    holes: &[Vec<PadHole>],
    layers: &LayerSet,
    i: isize,
    stroke_w: f32,
) {
    if i < 0 {
        return;
    }
    let i = i as usize;
    let layer_holes = holes.get(i).map(Vec::as_slice).unwrap_or(&[]);
    if !layer_holes.is_empty() {
        out.push_str(&format!(r#"<g mask="url(#export-h{i})">"#));
        write_prim_layers(out, by_layer, holes, layers, i as isize - 1, stroke_w);
        write_layer_prims(out, by_layer, layers, i, stroke_w);
        for hole in layer_holes {
            write_hole_marker(out, hole);
        }
        out.push_str("</g>");
    } else {
        write_prim_layers(out, by_layer, holes, layers, i as isize - 1, stroke_w);
        write_layer_prims(out, by_layer, layers, i, stroke_w);
    }
}

fn write_layer_prims(
    out: &mut String,
    by_layer: &[Vec<Primitive>],
    layers: &LayerSet,
    i: usize,
    stroke_w: f32,
) {
    let Some(prims) = by_layer.get(i) else {
        return;
    };
    for p in prims {
        write_prim(out, p, &rgb_u8(layers.color(p.layer())), stroke_w);
    }
}

fn write_prim(out: &mut String, p: &Primitive, color: &str, stroke_w: f32) {
    match p {
        Primitive::Line(l) => write_line_prim(out, l, color, stroke_w),
        Primitive::Rect(r) => write_rect_prim(out, r, color, stroke_w),
        Primitive::Poly(poly) => write_poly_prim(out, poly, color, stroke_w),
        Primitive::Ellipse(e) => write_ellipse_prim(out, e, color, stroke_w),
        Primitive::Bezier(b) => write_bezier_prim(out, b, color, stroke_w),
        Primitive::Text(t) => write_text_prim(out, t, color, stroke_w),
        Primitive::Connection(c) => write_connection_prim(out, c, color),
        Primitive::PcbTrack(t) => write_track_prim(out, t, color),
        Primitive::PcbPad(pad) => write_pad_prim(out, pad, color),
        Primitive::Component(_) => {}
    }
}

fn write_stroke_line(
    out: &mut String,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: &str,
    width: f32,
) {
    out.push_str(&format!(
        r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{color}" stroke-width="{width}" stroke-linecap="round"/>"#
    ));
}

fn write_line_prim(out: &mut String, l: &Line, color: &str, stroke_w: f32) {
    write_stroke_line(
        out,
        l.a.x as f32,
        l.a.y as f32,
        l.b.x as f32,
        l.b.y as f32,
        color,
        stroke_w,
    );
}

fn write_rect_prim(out: &mut String, r: &Rect, color: &str, stroke_w: f32) {
    let x = r.a.x.min(r.b.x) as f32;
    let y = r.a.y.min(r.b.y) as f32;
    let w = (r.a.x - r.b.x).unsigned_abs() as f32;
    let h = (r.a.y - r.b.y).unsigned_abs() as f32;
    if r.filled {
        out.push_str(&format!(
            r#"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="{color}"/>"#
        ));
    } else {
        out.push_str(&format!(
            r#"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="none" stroke="{color}" stroke-width="{stroke_w}"/>"#
        ));
    }
}

fn write_poly_prim(out: &mut String, poly: &Poly, color: &str, stroke_w: f32) {
    if poly.pts.len() < 2 {
        return;
    }
    let pts = poly
        .pts
        .iter()
        .map(|p| format!("{:.2},{:.2}", p.x as f32, p.y as f32))
        .collect::<Vec<_>>()
        .join(" ");
    if poly.filled && poly.pts.len() >= 3 {
        out.push_str(&format!(r#"<polygon points="{pts}" fill="{color}"/>"#));
    } else {
        out.push_str(&format!(
            r#"<polygon points="{pts}" fill="none" stroke="{color}" stroke-width="{stroke_w}" stroke-linejoin="round"/>"#
        ));
    }
}

fn write_ellipse_prim(out: &mut String, e: &Ellipse, color: &str, stroke_w: f32) {
    let cx = (e.a.x + e.b.x) as f32 / 2.0;
    let cy = (e.a.y + e.b.y) as f32 / 2.0;
    let rx = ((e.a.x - e.b.x).abs() as f32 / 2.0).max(0.5);
    let ry = ((e.a.y - e.b.y).abs() as f32 / 2.0).max(0.5);
    if e.filled {
        out.push_str(&format!(
            r#"<ellipse cx="{cx:.2}" cy="{cy:.2}" rx="{rx:.2}" ry="{ry:.2}" fill="{color}"/>"#
        ));
    } else {
        out.push_str(&format!(
            r#"<ellipse cx="{cx:.2}" cy="{cy:.2}" rx="{rx:.2}" ry="{ry:.2}" fill="none" stroke="{color}" stroke-width="{stroke_w:.2}"/>"#
        ));
    }
}

fn write_bezier_prim(out: &mut String, b: &Bezier, color: &str, stroke_w: f32) {
    out.push_str(&format!(
        r#"<path d="M {x0:.2},{y0:.2} C {x1:.2},{y1:.2} {x2:.2},{y2:.2} {x3:.2},{y3:.2}" fill="none" stroke="{color}" stroke-width="{stroke_w}" stroke-linecap="round"/>"#,
        x0 = b.p0.x as f32,
        y0 = b.p0.y as f32,
        x1 = b.p1.x as f32,
        y1 = b.p1.y as f32,
        x2 = b.p2.x as f32,
        y2 = b.p2.y as f32,
        x3 = b.p3.x as f32,
        y3 = b.p3.y as f32,
    ));
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn write_text_prim(out: &mut String, t: &Text, color: &str, stroke_w: f32) {
    if t.text.is_empty() {
        return;
    }
    let px = t.pos.x as f32;
    let py = t.pos.y as f32;
    let size = (t.sy as f32).max(1.0);
    let wch = (t.sx as f32).max(1.0);
    let n = t.text.chars().count() as f32;
    let font = if t.font.trim().is_empty() {
        DEFAULT_FONT
    } else {
        t.font.trim()
    };
    let angle = t.angle as f32;
    let mirrored = t.style & STYLE_MIRRORED != 0;
    let mut attrs = String::new();
    if angle.abs() > 0.001 || mirrored {
        attrs.push_str(&format!(
            r#" transform="translate({px:.2},{py:.2}) rotate({}){}""#,
            -angle,
            if mirrored { " scale(-1,1)" } else { "" }
        ));
        attrs.push_str(r#" x="0" y="0""#);
    } else {
        attrs.push_str(&format!(r#" x="{px:.2}" y="{py:.2}""#));
    }
    attrs.push_str(&format!(
        r#" font-family="&quot;{}&quot;,monospace" font-size="{size:.2}" fill="{color}" textLength="{:.2}" lengthAdjust="spacingAndGlyphs" dominant-baseline="hanging""#,
        xml_escape(font),
        n * wch,
    ));
    if t.style & STYLE_ITALIC != 0 {
        attrs.push_str(r#" font-style="italic""#);
    }
    if t.style & STYLE_BOLD != 0 {
        attrs.push_str(r#" font-weight="bold""#);
    }
    out.push_str(&format!("<text{attrs}>{}</text>", xml_escape(&t.text)));
    if t.style & STYLE_UNDERLINE != 0 {
        let x0 = if mirrored { -n * wch } else { 0.0 };
        let x1 = if mirrored { 0.0 } else { n * wch };
        let (sin, cos) = angle.to_radians().sin_cos();
        let map = |lx: f32, ly: f32| {
            let (dx, dy) = fidorust_core::TextLayout::map_offset(lx, ly, sin, cos);
            (px + dx, py + dy)
        };
        let (ax, ay) = map(x0, size);
        let (bx, by) = map(x1, size);
        write_stroke_line(out, ax, ay, bx, by, color, stroke_w);
    }
}

fn write_connection_prim(out: &mut String, c: &Connection, color: &str) {
    out.push_str(&format!(
        r#"<circle cx="{:.2}" cy="{:.2}" r="1.30" fill="{color}"/>"#,
        c.pos.x as f32, c.pos.y as f32
    ));
}

fn write_track_prim(out: &mut String, t: &PcbTrack, color: &str) {
    let ax = t.a.x as f32;
    let ay = t.a.y as f32;
    let bx = t.b.x as f32;
    let by = t.b.y as f32;
    let w = t.width as f32;
    let dx = bx - ax;
    let dy = by - ay;
    if dx * dx + dy * dy < 0.001 {
        out.push_str(&format!(
            r#"<circle cx="{ax:.2}" cy="{ay:.2}" r="{:.2}" fill="{color}"/>"#,
            w * 0.5
        ));
        return;
    }
    write_stroke_line(out, ax, ay, bx, by, color, w);
}

fn write_pad_prim(out: &mut String, pad: &PcbPad, color: &str) {
    let cx = pad.pos.x as f32;
    let cy = pad.pos.y as f32;
    let hx = pad.dx as f32 / 2.0;
    let hy = pad.dy as f32 / 2.0;
    match pad.style {
        PadStyle::Oval => {
            out.push_str(&format!(
                r#"<ellipse cx="{cx:.2}" cy="{cy:.2}" rx="{hx:.2}" ry="{hy:.2}" fill="{color}"/>"#
            ));
        }
        PadStyle::Rectangular => {
            out.push_str(&format!(
                r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{color}"/>"#,
                cx - hx,
                cy - hy,
                pad.dx as f32,
                pad.dy as f32,
            ));
        }
        PadStyle::RoundedRect => {
            let rx = hx * 0.5;
            let ry = hy * 0.5;
            out.push_str(&format!(
                r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="{rx:.2}" ry="{ry:.2}" fill="{color}"/>"#,
                cx - hx,
                cy - hy,
                pad.dx as f32,
                pad.dy as f32,
            ));
        }
    }
}

fn write_hole_circle(out: &mut String, hole: &PadHole, fill: &str) {
    out.push_str(&format!(
        r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="{fill}"/>"#,
        hole.x, hole.y, hole.r
    ));
}

fn write_hole_marker(out: &mut String, hole: &PadHole) {
    out.push_str(&format!(
        r#"<circle class="pad-hole" cx="{:.2}" cy="{:.2}" r="{:.2}" fill="none"/>"#,
        hole.x, hole.y, hole.r
    ));
}
