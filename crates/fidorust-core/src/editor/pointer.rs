//! Pointer, draft, and marquee interaction.

use super::{Draft, Drag, Editor, Tool};
use crate::consts::{DEFAULT_TEXT_SX, DEFAULT_TEXT_SY, ZOOM_MAX_WHEEL, ZOOM_TOOL_FACTOR};
use crate::geom::Point;
use crate::hit::{hit_test, marquee_select, HitQuery};
use crate::primitive::{
    Bezier, Connection, Ellipse, Line, PcbPad, PcbTrack, Poly, Primitive, Rect, Text, DEFAULT_FONT,
};

impl Editor {
    pub(super) fn pick_at(&self, x: f64, y: f64) -> Option<crate::hit::Hit> {
        hit_test(
            &self.doc.primitives,
            &self.libs,
            &self.doc.layers,
            &self.selected,
            HitQuery {
                x,
                y,
                zoom: self.zoom,
                stroke_w: self.doc.stroke_width() as f64,
            },
        )
    }

    pub(super) fn refresh_hover_hit(&mut self, x: f64, y: f64) {
        let hit = self.pick_at(x, y);
        self.hover_hit = hit.is_some();
        self.hover_index = if self.tool == Tool::Select {
            hit.map(|h| h.index)
        } else {
            None
        };
    }

    /// Start a selection marquee at `screen`, even if a primitive is under the cursor.
    /// Used by right-drag; left-click empty space shares the same path.
    pub fn begin_marquee(&mut self, screen: (f32, f32), shift: bool) {
        if self.tool != Tool::Select {
            return;
        }
        if !shift {
            self.selected.clear();
        }
        let kept = self.selected.clone();
        self.drag = Some(Drag::Marquee {
            start: screen,
            current: screen,
            kept,
        });
        self.apply_marquee_hits();
    }

    fn apply_marquee_hits(&mut self) {
        let Some(Drag::Marquee {
            start,
            current,
            kept,
        }) = &self.drag
        else {
            return;
        };
        let (start, current, kept) = (*start, *current, kept.clone());
        let a = self.screen_to_world(start.0, start.1);
        let b = self.screen_to_world(current.0, current.1);
        let extra = marquee_select(&self.doc.primitives, &self.libs, a, b);
        self.selected.clear();
        self.selected.extend(kept);
        for i in extra {
            if !self.selected.contains(&i) {
                self.selected.push(i);
            }
        }
    }

    pub fn pointer_down(&mut self, world: Point, screen: (f32, f32), shift: bool, pan_mod: bool) {
        self.pointer_down_at(
            world.x as f64,
            world.y as f64,
            world,
            screen,
            shift,
            pan_mod,
        );
    }

    pub fn pointer_down_at(
        &mut self,
        hx: f64,
        hy: f64,
        world: Point,
        screen: (f32, f32),
        shift: bool,
        pan_mod: bool,
    ) {
        let pt = self.snap_pt(world);
        self.refresh_hover_hit(hx, hy);
        if matches!(&self.drag, Some(Drag::PlaceClone { .. })) {
            if !pan_mod {
                self.place_pending_clone_at(pt);
            }
            return;
        }
        if pan_mod || self.tool == Tool::Pan {
            self.drag = Some(Drag::Pan {
                start_screen: screen,
                pan0: self.pan,
            });
            return;
        }
        match self.tool {
            Tool::Select => {
                if let Some(hit) = self.pick_at(hx, hy) {
                    if !shift && !self.selected.contains(&hit.index) {
                        self.selected.clear();
                    }
                    if !self.selected.contains(&hit.index) {
                        self.selected.push(hit.index);
                    }
                    if let Some(h) = hit.handle {
                        self.begin_drag_checkpoint();
                        self.drag = Some(Drag::Handle {
                            index: hit.index,
                            handle: h,
                        });
                    } else {
                        self.begin_drag_checkpoint();
                        self.drag = Some(Drag::Move {
                            start: pt,
                            last: pt,
                            duplicate: false,
                        });
                    }
                } else {
                    self.begin_marquee(screen, shift);
                }
            }
            Tool::Connection => {
                self.push_undo();
                self.doc.insert(Primitive::Connection(Connection {
                    pos: pt,
                    layer: self.layer,
                }));
            }
            Tool::PcbPad => {
                self.push_undo();
                self.doc.insert(Primitive::PcbPad(PcbPad {
                    pos: pt,
                    dx: self.pad_dx,
                    dy: self.pad_dy,
                    hole: self.pad_hole,
                    style: self.pad_style,
                    layer: self.layer,
                }));
            }
            Tool::Text => {
                self.push_undo();
                self.doc.insert(Primitive::Text(Text {
                    pos: pt,
                    sy: DEFAULT_TEXT_SY,
                    sx: DEFAULT_TEXT_SX,
                    angle: 0,
                    style: 0,
                    layer: self.layer,
                    font: DEFAULT_FONT.into(),
                    text: self.pending_text.clone(),
                    simple: false,
                }));
            }
            Tool::Component => {
                let _ = self.insert_pending_component_at(pt);
            }
            Tool::Zoom => {
                self.zoom = (self.zoom * ZOOM_TOOL_FACTOR).min(ZOOM_MAX_WHEEL);
            }
            Tool::Line | Tool::Rect | Tool::Ellipse | Tool::PcbTrack | Tool::Ruler => {
                if let Some(d) = self.draft.as_mut() {
                    Self::set_draft_endpoint(d, pt);
                } else {
                    self.draft = Some(Draft {
                        tool: self.tool,
                        points: vec![pt],
                    });
                }
            }
            Tool::Poly | Tool::Bezier => {
                if let Some(d) = self.draft.as_mut() {
                    d.points.push(pt);
                } else {
                    self.draft = Some(Draft {
                        tool: self.tool,
                        points: vec![pt],
                    });
                }
            }
            Tool::Pan => {}
        }
    }

    pub fn pointer_move(&mut self, world: Point, screen: (f32, f32)) {
        self.pointer_move_at(world.x as f64, world.y as f64, world, screen);
    }

    pub fn pointer_move_at(&mut self, hx: f64, hy: f64, world: Point, screen: (f32, f32)) {
        let pt = self.snap_pt(world);
        self.hover = Some(pt);
        if self.drag.is_none() {
            self.refresh_hover_hit(hx, hy);
        }
        if let Some(d) = &mut self.draft {
            if d.tool.is_two_point_draw() || d.tool == Tool::Ruler {
                Self::set_draft_endpoint(d, pt);
            } else if matches!(d.tool, Tool::Poly | Tool::Bezier) {
                match d.points.len() {
                    0 | 1 => d.points.push(pt),
                    _ => {}
                }
            }
        }
        match &self.drag {
            Some(Drag::Move {
                last, duplicate, ..
            }) => {
                let duplicate = *duplicate;
                let delta = Point::new(pt.x - last.x, pt.y - last.y);
                if delta != Point::new(0, 0) {
                    if !duplicate {
                        self.translate_selected(delta);
                    }
                    if let Some(Drag::Move { last, .. }) = &mut self.drag {
                        *last = pt;
                    }
                }
            }
            Some(Drag::Handle { index, handle }) => {
                let (i, h) = (*index, *handle);
                if let Some(p) = self.doc.primitives.get_mut(i) {
                    p.set_control_point(h, pt);
                }
            }
            Some(Drag::Pan { start_screen, pan0 }) => {
                self.pan = (
                    pan0.0 + (screen.0 - start_screen.0),
                    pan0.1 + (screen.1 - start_screen.1),
                );
            }
            Some(Drag::Marquee { .. }) => {
                if let Some(Drag::Marquee { current, .. }) = &mut self.drag {
                    *current = screen;
                }
                self.apply_marquee_hits();
            }
            Some(Drag::PlaceClone { .. }) => self.move_place_clone(pt),
            None => {}
        }
    }

    pub fn pointer_up(&mut self, world: Point) {
        self.pointer_up_at(world.x as f64, world.y as f64, world);
    }

    pub fn pointer_up_at(&mut self, hx: f64, hy: f64, world: Point) {
        let pt = self.snap_pt(world);
        if matches!(&self.drag, Some(Drag::PlaceClone { .. })) {
            return;
        }
        if matches!(&self.drag, Some(Drag::Marquee { .. })) {
            self.apply_marquee_hits();
        }
        match self.drag.take() {
            Some(Drag::Marquee { .. }) => {
                self.drag_checkpoint = None;
                self.refresh_hover_hit(hx, hy);
                return;
            }
            Some(Drag::Move {
                start,
                duplicate: true,
                ..
            }) => {
                let dx = pt.x - start.x;
                let dy = pt.y - start.y;
                if dx != 0 || dy != 0 {
                    self.insert_translated_clones(dx, dy);
                }
            }
            _ => {}
        }
        self.commit_drag_checkpoint();
        if let Some(d) = self.draft.take() {
            if d.tool.is_two_point_draw() {
                if !self.commit_two_point_draft(&d) {
                    if let Some(&start) = d.points.first() {
                        self.draft = Some(Draft {
                            tool: d.tool,
                            points: vec![start],
                        });
                    }
                }
            } else {
                match d.tool {
                    Tool::Ruler => {
                        if d.points.len() >= 2 && d.points[0] != d.points[1] {
                            self.ruler_segments.push((d.points[0], d.points[1]));
                        } else {
                            self.draft = Some(d);
                        }
                    }
                    Tool::Poly => {
                        let mut pts = d.points;
                        pts.push(pt);
                        if pts.len() >= 2 {
                            self.draft = Some(Draft {
                                tool: Tool::Poly,
                                points: pts,
                            });
                        }
                    }
                    Tool::Bezier => {
                        let mut pts = d.points;
                        pts.push(pt);
                        let ready = pts.len() >= 4;
                        self.draft = Some(Draft {
                            tool: Tool::Bezier,
                            points: pts.clone(),
                        });
                        if ready {
                            self.push_undo();
                            self.doc.insert(Primitive::Bezier(Bezier {
                                p0: pts[0],
                                p1: pts[1],
                                p2: pts[2],
                                p3: pts[3],
                                layer: self.layer,
                            }));
                            self.draft = None;
                        }
                    }
                    _ => {}
                }
            }
        }
        self.refresh_hover_hit(hx, hy);
    }

    fn set_draft_endpoint(d: &mut Draft, pt: Point) {
        if d.points.len() < 2 {
            d.points.push(pt);
        } else {
            d.points[1] = pt;
        }
    }

    fn commit_two_point_draft(&mut self, d: &Draft) -> bool {
        if d.points.len() < 2 || d.points[0] == d.points[1] {
            return false;
        }
        let a = d.points[0];
        let b = d.points[1];
        let prim = match d.tool {
            Tool::Line => Primitive::Line(Line {
                a,
                b,
                layer: self.layer,
            }),
            Tool::Rect => Primitive::Rect(Rect {
                a,
                b,
                filled: self.doc.default_filled,
                layer: self.layer,
            }),
            Tool::Ellipse => Primitive::Ellipse(Ellipse {
                a,
                b,
                filled: self.doc.default_filled,
                layer: self.layer,
            }),
            Tool::PcbTrack => Primitive::PcbTrack(PcbTrack {
                a,
                b,
                width: self.track_width,
                layer: self.layer,
            }),
            _ => return false,
        };
        self.push_undo();
        self.doc.insert(prim);
        true
    }

    pub fn finish_poly(&mut self) {
        if let Some(d) = self.draft.take() {
            if d.tool == Tool::Poly && d.points.len() >= 2 {
                self.push_undo();
                self.doc.insert(Primitive::Poly(Poly {
                    pts: d.points,
                    filled: self.doc.default_filled,
                    layer: self.layer,
                }));
            }
        }
    }

    pub fn cancel_draft(&mut self) {
        self.draft = None;
        self.drag = None;
        self.commit_drag_checkpoint();
    }
}
