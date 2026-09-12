//! Hit-testing against flattened primitives.

use crate::consts::{HANDLE_RADIUS_PX, HIT_TOLERANCE_PX};
use crate::geom::Point;
use crate::layers::LayerSet;
use crate::library::LibrarySet;
use crate::primitive::Primitive;

pub struct Hit {
    pub index: usize,
    pub handle: Option<usize>,
}

struct Candidate {
    handle: bool,
    opaque: bool,
    layer: u8,
    paint: u8,
    index: usize,
    hit: Hit,
}

impl Candidate {
    fn beats(&self, other: &Self) -> bool {
        match (self.handle, other.handle) {
            (true, false) => true,
            (false, true) => false,
            _ => match (self.opaque, other.opaque) {
                (true, false) => true,
                (false, true) => false,
                _ => {
                    self.layer > other.layer
                        || (self.layer == other.layer && self.paint > other.paint)
                        || (self.layer == other.layer
                            && self.paint == other.paint
                            && self.index >= other.index)
                }
            },
        }
    }
}

/// World-space radius² matching a [`crate::consts::HANDLE_RADIUS_PX`] circle at `zoom`.
fn handle_r2_world(zoom: f32) -> f64 {
    let z = zoom.max(0.1) as f64;
    let r = HANDLE_RADIUS_PX as f64 / z;
    r * r
}

fn hit_prim(
    p: &Primitive,
    x: f64,
    y: f64,
    tol2: f64,
    handle_r2: f64,
    test_handles: bool,
) -> Option<usize> {
    if test_handles {
        let handles = p.control_points();
        for (i, h) in handles.iter().enumerate() {
            if h.dist_sq_xy(x, y) <= handle_r2 {
                return Some(i);
            }
        }
    }
    if p.body_hit_xy(x, y, tol2) {
        Some(usize::MAX) // body, not a handle
    } else {
        None
    }
}

/// Pick the top-most primitive under `(x, y)` (world LU, unrounded).
///
/// Priority: selected handles (drawn on top), then **opaque ink** at the point
/// (fill / stroke / copper, not holes or click-slop), then highest layer, then
/// GPU paint batch (fills, then strokes, then circles — so an oval pad sits on
/// a track), then later document order. Transparent hits still win when nothing
/// opaque is under the cursor.
pub fn hit_test(
    prims: &[Primitive],
    libs: &LibrarySet,
    layers: &LayerSet,
    selected: &[usize],
    x: f64,
    y: f64,
    zoom: f32,
    stroke_w: f64,
) -> Option<Hit> {
    let tol = HIT_TOLERANCE_PX / zoom.max(0.1) as f64;
    let tol2 = tol * tol;
    let handle_r2 = handle_r2_world(zoom);
    let mut best: Option<Candidate> = None;

    let consider = |best: &mut Option<Candidate>, cand: Candidate| {
        if best.as_ref().map_or(true, |cur| cand.beats(cur)) {
            *best = Some(cand);
        }
    };

    for (index, p) in prims.iter().enumerate() {
        if !p.uses_component_layers() && !layers.visible(p.layer()) {
            continue;
        }
        let expanded = crate::library::expand_primitive(p, libs);
        let test_handles = selected.contains(&index);
        for q in &expanded {
            if !layers.visible(q.layer()) {
                continue;
            }
            if let Some(h) = hit_prim(q, x, y, tol2, handle_r2, test_handles) {
                let is_handle = expanded.len() == 1 && h != usize::MAX;
                let handle = if is_handle { Some(h) } else { None };
                let ink = q.opaque_at_xy(x, y, stroke_w);
                let opaque = is_handle || (layers.color(q.layer())[3] == 255 && ink);
                consider(
                    &mut best,
                    Candidate {
                        handle: is_handle,
                        opaque,
                        layer: q.layer().0,
                        paint: q.paint_order(),
                        index,
                        hit: Hit { index, handle },
                    },
                );
            }
        }
        if let Primitive::Component(m) = p {
            if m.pos.dist_sq_xy(x, y) <= handle_r2 {
                consider(
                    &mut best,
                    Candidate {
                        handle: test_handles,
                        opaque: true,
                        layer: m.layer.0,
                        paint: 0,
                        index,
                        hit: Hit {
                            index,
                            handle: test_handles.then_some(0),
                        },
                    },
                );
            }
        }
    }
    best.map(|c| c.hit)
}

pub fn hit_test_pt(
    prims: &[Primitive],
    libs: &LibrarySet,
    layers: &LayerSet,
    selected: &[usize],
    pt: Point,
    zoom: f32,
) -> Option<Hit> {
    hit_test(
        prims,
        libs,
        layers,
        selected,
        pt.x as f64,
        pt.y as f64,
        zoom,
        crate::consts::DEFAULT_STROKE_HUNDREDTHS as f64 / 100.0,
    )
}

pub fn marquee_select(prims: &[Primitive], libs: &LibrarySet, a: Point, b: Point) -> Vec<usize> {
    let minx = a.x.min(b.x);
    let maxx = a.x.max(b.x);
    let miny = a.y.min(b.y);
    let maxy = a.y.max(b.y);
    let mut out = Vec::new();
    for (i, p) in prims.iter().enumerate() {
        let bb = crate::library::expanded_aabb(p, libs);
        if bb.min.x >= minx && bb.max.x <= maxx && bb.min.y >= miny && bb.max.y <= maxy {
            out.push(i);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::LayerId;
    use crate::primitive::{Line, Primitive, Rect, Text, DEFAULT_FONT};

    fn line_prims() -> Vec<Primitive> {
        vec![Primitive::Line(Line {
            a: Point::new(0, 0),
            b: Point::new(100, 0),
            layer: LayerId(0),
        })]
    }

    fn pick(prims: &[Primitive], selected: &[usize], pt: Point, zoom: f32) -> Option<Hit> {
        hit_test_pt(
            prims,
            &LibrarySet::default(),
            &LayerSet::default(),
            selected,
            pt,
            zoom,
        )
    }

    #[test]
    fn handle_hit_is_screen_space() {
        let prims = line_prims();
        let selected = [0usize];
        // 8 LU above the origin handle: 4 px at zoom 0.5 (inside 6 px), 32 px at zoom 4.
        let pt = Point::new(0, 8);
        let lo = pick(&prims, &selected, pt, 0.5);
        assert_eq!(lo.and_then(|h| h.handle), Some(0));
        let hi = pick(&prims, &selected, pt, 4.0);
        assert!(hi.is_none());
    }

    #[test]
    fn handle_hit_at_center_survives_high_zoom() {
        let prims = line_prims();
        let hit = pick(&prims, &[0], Point::new(0, 0), 40.0);
        assert_eq!(hit.and_then(|h| h.handle), Some(0));
    }

    #[test]
    fn higher_layer_wins_over_later_primitive() {
        let prims = vec![
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(1),
            }),
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(0),
            }),
        ];
        let hit = pick(&prims, &[], Point::new(10, 10), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(0));
    }

    #[test]
    fn later_primitive_wins_on_same_layer() {
        let prims = vec![
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(0),
            }),
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(0),
            }),
        ];
        let hit = pick(&prims, &[], Point::new(10, 10), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(1));
    }

    #[test]
    fn miss_when_not_on_geometry() {
        let prims = line_prims();
        let hit = pick(&prims, &[], Point::new(50, 20), 4.0);
        assert!(hit.is_none());
    }

    fn pick_layers(prims: &[Primitive], layers: &LayerSet, pt: Point, zoom: f32) -> Option<Hit> {
        hit_test_pt(prims, &LibrarySet::default(), layers, &[], pt, zoom)
    }

    #[test]
    fn opaque_under_hollow_stroke_slop_wins() {
        let prims = vec![
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(0),
            }),
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: false,
                layer: LayerId(1),
            }),
        ];
        // 1 LU inside the top edge: hollow hit is click-slop only, fill is opaque.
        let hit = pick(&prims, &[], Point::new(10, 1), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(0));
    }

    #[test]
    fn hollow_stroke_ink_still_wins_on_higher_layer() {
        let prims = vec![
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(0),
            }),
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: false,
                layer: LayerId(1),
            }),
        ];
        let hit = pick(&prims, &[], Point::new(10, 0), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(1));
    }

    #[test]
    fn pad_hole_clicks_through_to_track() {
        use crate::primitive::{PadStyle, PcbPad, PcbTrack};
        let prims = vec![
            Primitive::PcbTrack(PcbTrack {
                a: Point::new(0, 10),
                b: Point::new(20, 10),
                width: 4,
                layer: LayerId(0),
            }),
            Primitive::PcbPad(PcbPad {
                pos: Point::new(10, 10),
                dx: 18,
                dy: 18,
                hole: 8,
                style: PadStyle::Oval,
                layer: LayerId(1),
            }),
        ];
        let through = pick(&prims, &[], Point::new(10, 10), 4.0);
        assert_eq!(through.map(|h| h.index), Some(0));
        let copper = pick(&prims, &[], Point::new(10, 16), 4.0);
        assert_eq!(copper.map(|h| h.index), Some(1));
    }

    #[test]
    fn oval_pad_beats_later_track_on_same_layer() {
        use crate::primitive::{PadStyle, PcbPad, PcbTrack};
        // Same stacking as GPU: tracks are fills, oval pads are circles drawn after.
        let prims = vec![
            Primitive::PcbPad(PcbPad {
                pos: Point::new(10, 10),
                dx: 12,
                dy: 12,
                hole: 4,
                style: PadStyle::Oval,
                layer: LayerId(0),
            }),
            Primitive::PcbTrack(PcbTrack {
                a: Point::new(0, 10),
                b: Point::new(20, 10),
                width: 8,
                layer: LayerId(0),
            }),
        ];
        // Copper overlap (on the pad ring and on the track).
        let overlap = pick(&prims, &[], Point::new(10, 13), 4.0);
        assert_eq!(overlap.map(|h| h.index), Some(0));
        // Drill: pad is hollow, track copper remains.
        let hole = pick(&prims, &[], Point::new(10, 10), 4.0);
        assert_eq!(hole.map(|h| h.index), Some(1));
    }

    #[test]
    fn translucent_layer_defers_to_opaque_below() {
        let mut layers = LayerSet::default();
        layers.update(1, |l| l.color[3] = 80);
        let prims = vec![
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(0),
            }),
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(1),
            }),
        ];
        let hit = pick_layers(&prims, &layers, Point::new(10, 10), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(0));
    }

    #[test]
    fn translucent_only_keeps_highest_layer() {
        let mut layers = LayerSet::default();
        layers.update(0, |l| l.color[3] = 80);
        layers.update(1, |l| l.color[3] = 80);
        let prims = vec![
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(0),
            }),
            Primitive::Rect(Rect {
                a: Point::new(0, 0),
                b: Point::new(20, 20),
                filled: true,
                layer: LayerId(1),
            }),
        ];
        let hit = pick_layers(&prims, &layers, Point::new(10, 10), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(1));
    }

    fn sample_text(pos: Point, layer: LayerId, text: &str) -> Primitive {
        Primitive::Text(Text {
            pos,
            sy: 4,
            sx: 3,
            angle: 0,
            style: 0,
            layer,
            font: DEFAULT_FONT.into(),
            text: text.into(),
            simple: false,
        })
    }

    #[test]
    fn text_box_without_glyph_ink_clicks_through_to_fill() {
        let prims = vec![
            Primitive::Rect(Rect {
                a: Point::new(10, 20),
                b: Point::new(16, 24),
                filled: true,
                layer: LayerId(0),
            }),
            sample_text(Point::new(10, 20), LayerId(1), "AB"),
        ];
        // Inside the glyph cell, but core has no font tessellation so the box is hollow.
        let hit = pick(&prims, &[], Point::new(11, 21), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(0));
    }

    #[test]
    fn text_box_still_selectable_when_nothing_opaque_underneath() {
        let prims = vec![sample_text(Point::new(10, 20), LayerId(0), "AB")];
        let hit = pick(&prims, &[], Point::new(11, 21), 4.0);
        assert_eq!(hit.map(|h| h.index), Some(0));
    }
}
