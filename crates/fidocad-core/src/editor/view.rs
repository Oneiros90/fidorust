//! Zoom, pan, and coordinate mapping.

use super::Editor;
use crate::consts::{FIT_MARGIN, WHEEL_ZOOM_FACTOR, ZOOM_MAX_FIT, ZOOM_MAX_WHEEL, ZOOM_MIN};
use crate::geom::{snap, Point};

impl Editor {
    pub fn snap_pt(&self, p: Point) -> Point {
        if !self.doc.snap_enable {
            return p;
        }
        Point::new(snap(p.x, self.doc.snap), snap(p.y, self.doc.snap_y))
    }

    pub fn wheel_zoom(&mut self, screen: (f32, f32), delta: f32) {
        let old = self.zoom;
        let factor = if delta < 0.0 {
            WHEEL_ZOOM_FACTOR
        } else {
            1.0 / WHEEL_ZOOM_FACTOR
        };
        self.zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX_WHEEL);
        let wx = (screen.0 - self.pan.0) / old;
        let wy = (screen.1 - self.pan.1) / old;
        self.pan.0 = screen.0 - wx * self.zoom;
        self.pan.1 = screen.1 - wy * self.zoom;
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> Point {
        Point::new(
            ((sx - self.pan.0) / self.zoom).round() as i32,
            ((sy - self.pan.1) / self.zoom).round() as i32,
        )
    }

    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (wx * self.zoom + self.pan.0, wy * self.zoom + self.pan.1)
    }

    pub fn fit_view(&mut self, w: f32, h: f32) {
        let bb = self.doc.aabb(&self.libs);
        if bb.is_empty() {
            self.zoom = 4.0;
            self.pan = (FIT_MARGIN, FIT_MARGIN);
            return;
        }
        let margin = FIT_MARGIN;
        let zw = (w - margin * 2.0) / bb.width().max(1) as f32;
        let zh = (h - margin * 2.0) / bb.height().max(1) as f32;
        self.zoom = zw.min(zh).clamp(ZOOM_MIN, ZOOM_MAX_FIT);
        self.pan = (
            margin - bb.min.x as f32 * self.zoom,
            margin - bb.min.y as f32 * self.zoom,
        );
    }
}
