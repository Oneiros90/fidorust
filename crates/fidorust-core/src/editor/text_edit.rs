//! In-place text editing and canvas double-click dispatch.

use super::{DblClickAction, Editor, TextEditSession, Tool};
use crate::primitive::{Primitive, Text};

impl Editor {
    pub fn text_edit_session(&self, index: usize) -> Option<TextEditSession> {
        match self.doc.primitives.get(index)? {
            Primitive::Text(Text {
                pos,
                sy,
                sx,
                angle,
                style,
                text,
                ..
            }) => Some(TextEditSession {
                index,
                text: text.clone(),
                wx: pos.x,
                wy: pos.y,
                sx: *sx,
                sy: *sy,
                angle: *angle,
                style: *style,
            }),
            _ => None,
        }
    }

    fn begin_text_edit_index(&mut self, index: usize) -> Option<TextEditSession> {
        let session = self.text_edit_session(index)?;
        self.selected = vec![index];
        self.commit_drag_checkpoint();
        self.drag = None;
        self.editing_text = Some(index);
        Some(session)
    }

    /// Double-click: finish a polygon draft, edit text in place, or open properties.
    pub fn handle_dblclick(&mut self, world: crate::geom::Point) -> DblClickAction {
        self.handle_dblclick_at(world.x as f64, world.y as f64)
    }

    pub fn handle_dblclick_at(&mut self, x: f64, y: f64) -> DblClickAction {
        if self.draft.as_ref().is_some_and(|d| d.tool == Tool::Poly) {
            self.finish_poly();
            return DblClickAction::None;
        }
        if self.draft.is_some() {
            return DblClickAction::None;
        }
        self.commit_drag_checkpoint();
        self.drag = None;
        let Some(hit) = self.pick_at(x, y) else {
            return DblClickAction::None;
        };
        if let Some(session) = self.begin_text_edit_index(hit.index) {
            return DblClickAction::TextEdit(session);
        }
        if self.tool != Tool::Select {
            return DblClickAction::None;
        }
        if !self.selected.contains(&hit.index) {
            self.selected = vec![hit.index];
        }
        DblClickAction::OpenProperties
    }

    /// Double-click handler: finish a polygon draft, or start in-place text edit.
    pub fn begin_text_edit_at(&mut self, world: crate::geom::Point) -> Option<TextEditSession> {
        match self.handle_dblclick(world) {
            DblClickAction::TextEdit(session) => Some(session),
            _ => None,
        }
    }

    pub fn begin_text_edit_selected(&mut self) -> Option<TextEditSession> {
        let index = *self.selected.first()?;
        self.begin_text_edit_index(index)
    }

    pub fn commit_text_edit(&mut self, text: String) {
        let Some(index) = self.editing_text.take() else {
            return;
        };
        if !self.selected.contains(&index) {
            self.selected = vec![index];
        }
        self.replace_selected_text(text);
    }

    pub fn cancel_text_edit(&mut self) {
        self.editing_text = None;
    }

    pub fn replace_selected_text(&mut self, text: String) {
        let changed = self.selected.iter().any(|&i| {
            matches!(
                self.doc.primitives.get(i),
                Some(Primitive::Text(Text { text: t, .. })) if *t != text
            )
        });
        if !changed {
            return;
        }
        self.push_undo();
        for &i in &self.selected {
            if let Some(Primitive::Text(Text { text: t, .. })) = self.doc.primitives.get_mut(i) {
                *t = text.clone();
            }
        }
    }
}
