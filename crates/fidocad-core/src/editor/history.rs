//! Undo / redo history.

use super::Editor;
use crate::consts::UNDO_CAP;
use crate::document::Document;

impl Editor {
    pub(super) fn push_undo(&mut self) {
        self.layer_color_edit = None;
        self.push_undo_snapshot(self.doc.clone());
    }

    fn push_undo_snapshot(&mut self, snapshot: Document) {
        self.undo.push(snapshot);
        if self.undo.len() > UNDO_CAP {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub(super) fn begin_drag_checkpoint(&mut self) {
        if self.drag_checkpoint.is_none() {
            self.drag_checkpoint = Some(self.doc.clone());
        }
    }

    pub(super) fn commit_drag_checkpoint(&mut self) {
        let Some(checkpoint) = self.drag_checkpoint.take() else {
            return;
        };
        if checkpoint != self.doc {
            self.push_undo_snapshot(checkpoint);
        }
    }

    pub(super) fn clear_history(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.drag_checkpoint = None;
        self.layer_color_edit = None;
    }

    pub fn undo(&mut self) {
        self.commit_drag_checkpoint();
        self.layer_color_edit = None;
        self.drag = None;
        if let Some(prev) = self.undo.pop() {
            self.redo.push(self.doc.clone());
            self.doc = prev;
            self.selected.clear();
        }
    }

    pub fn redo(&mut self) {
        self.commit_drag_checkpoint();
        self.layer_color_edit = None;
        self.drag = None;
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.doc.clone());
            self.doc = next;
            self.selected.clear();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
