//! Undo / redo history.

use super::Editor;
use crate::consts::UNDO_CAP;
use crate::document::Document;
use crate::library::Library;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct HistorySnapshot {
    pub doc: Document,
    pub project: Library,
    pub user: Vec<Library>,
}

impl Editor {
    pub(super) fn capture_snapshot(&self) -> HistorySnapshot {
        HistorySnapshot {
            doc: self.doc.clone(),
            project: self
                .libs
                .project()
                .cloned()
                .unwrap_or_else(Library::empty_project),
            user: self.libs.user_libraries_cloned(),
        }
    }

    pub(super) fn apply_snapshot(&mut self, snap: HistorySnapshot) {
        let libs_changed = self.libs.project() != Some(&snap.project)
            || self.libs.user_libraries_cloned() != snap.user;
        self.doc = snap.doc;
        self.libs.set_project(snap.project);
        self.libs.replace_user_libraries(snap.user);
        if libs_changed {
            self.bump_libs_rev();
        }
    }

    pub(super) fn push_undo(&mut self) {
        self.layer_color_edit = None;
        self.push_undo_snapshot(self.capture_snapshot());
    }

    fn push_undo_snapshot(&mut self, snapshot: HistorySnapshot) {
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
            let mut snap = self.capture_snapshot();
            snap.doc = checkpoint;
            self.push_undo_snapshot(snap);
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
            self.redo.push(self.capture_snapshot());
            self.apply_snapshot(prev);
            self.selected.clear();
        }
    }

    pub fn redo(&mut self) {
        self.commit_drag_checkpoint();
        self.layer_color_edit = None;
        self.drag = None;
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.capture_snapshot());
            self.apply_snapshot(next);
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
