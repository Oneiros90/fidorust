//! In-place text editing.

use super::{Editor, TextEditSession, Tool};
use crate::hit::hit_test;
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

    /// Double-click handler: finish a polygon draft, or start in-place text edit.
    pub fn begin_text_edit_at(&mut self, world: crate::geom::Point) -> Option<TextEditSession> {
        if self.draft.as_ref().is_some_and(|d| d.tool == Tool::Poly) {
            self.finish_poly();
            return None;
        }
        if self.draft.is_some() {
            return None;
        }
        self.commit_drag_checkpoint();
        self.drag = None;
        let hit = hit_test(
            &self.doc.primitives,
            &self.libs,
            &self.doc.layers,
            world,
            self.zoom,
        )?;
        self.begin_text_edit_index(hit.index)
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
