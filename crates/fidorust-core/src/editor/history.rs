//! Undo / redo history (hybrid: sheet geometry vs project-wide).

use super::Editor;
use crate::consts::UNDO_CAP;
use crate::document::Sheet;
use crate::layers::LayerSet;
use crate::library::Library;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum HistoryOp {
    Sheet { index: usize, sheet: Sheet },
    Project(ProjectSnap),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ProjectSnap {
    pub layers: LayerSet,
    pub hide_component_origin: bool,
    pub stroke_hundredths: i32,
    pub default_filled: bool,
    pub pcb_mode: bool,
    pub sheets: Vec<Sheet>,
    pub project: Library,
    pub user: Vec<Library>,
}

impl Editor {
    fn capture_sheet(&self) -> HistoryOp {
        let index = self.doc.view_index();
        HistoryOp::Sheet {
            index,
            sheet: self.doc.sheets[index].clone(),
        }
    }

    fn capture_project(&self) -> HistoryOp {
        HistoryOp::Project(ProjectSnap {
            layers: self.doc.layers.clone(),
            hide_component_origin: self.doc.hide_component_origin,
            stroke_hundredths: self.doc.stroke_hundredths,
            default_filled: self.doc.default_filled,
            pcb_mode: self.doc.pcb_mode,
            sheets: self.doc.sheets.clone(),
            project: self
                .libs
                .project()
                .cloned()
                .unwrap_or_else(Library::empty_project),
            user: self.libs.user_libraries_cloned(),
        })
    }

    fn apply_op(&mut self, op: HistoryOp) {
        match op {
            HistoryOp::Sheet { index, sheet } => {
                if index < self.doc.sheets.len() {
                    self.doc.sheets[index] = sheet;
                }
                self.clear_sheet_selections(index);
            }
            HistoryOp::Project(snap) => {
                let libs_changed = self.libs.project() != Some(&snap.project)
                    || self.libs.user_libraries_cloned() != snap.user;
                self.doc.layers = snap.layers;
                self.doc.hide_component_origin = snap.hide_component_origin;
                self.doc.stroke_hundredths = snap.stroke_hundredths;
                self.doc.default_filled = snap.default_filled;
                self.doc.pcb_mode = snap.pcb_mode;
                self.doc.sheets = snap.sheets;
                if self.doc.sheets.is_empty() {
                    self.doc.sheets.push(Sheet::default());
                }
                self.doc.set_view_index(self.doc.view_index());
                self.libs.set_project(snap.project);
                self.libs.replace_user_libraries(snap.user);
                if libs_changed {
                    self.bump_libs_rev();
                }
                self.clamp_pane_sheets();
                self.clear_all_selections();
            }
        }
    }

    /// Geometry / grid-snap of the view sheet.
    pub(super) fn push_undo(&mut self) {
        self.layer_color_edit = None;
        self.push_undo_op(self.capture_sheet());
    }

    /// Layers, libraries, and project drawing defaults.
    pub(super) fn push_project_undo(&mut self) {
        self.layer_color_edit = None;
        self.push_undo_op(self.capture_project());
    }

    fn push_undo_op(&mut self, op: HistoryOp) {
        self.undo.push(op);
        if self.undo.len() > UNDO_CAP {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub(super) fn begin_drag_checkpoint(&mut self) {
        if self.drag_checkpoint.is_none() {
            let index = self.doc.view_index();
            self.drag_checkpoint = Some((index, self.doc.sheets[index].clone()));
        }
    }

    pub(super) fn commit_drag_checkpoint(&mut self) {
        let Some((index, checkpoint)) = self.drag_checkpoint.take() else {
            return;
        };
        if self.doc.sheets.get(index) != Some(&checkpoint) {
            self.push_undo_op(HistoryOp::Sheet {
                index,
                sheet: checkpoint,
            });
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
            let current = match &prev {
                HistoryOp::Sheet { index, .. } => HistoryOp::Sheet {
                    index: *index,
                    sheet: self
                        .doc
                        .sheets
                        .get(*index)
                        .cloned()
                        .unwrap_or_else(Sheet::default),
                },
                HistoryOp::Project(_) => self.capture_project(),
            };
            self.redo.push(current);
            self.apply_op(prev);
        }
    }

    pub fn redo(&mut self) {
        self.commit_drag_checkpoint();
        self.layer_color_edit = None;
        self.drag = None;
        if let Some(next) = self.redo.pop() {
            let current = match &next {
                HistoryOp::Sheet { index, .. } => HistoryOp::Sheet {
                    index: *index,
                    sheet: self
                        .doc
                        .sheets
                        .get(*index)
                        .cloned()
                        .unwrap_or_else(Sheet::default),
                },
                HistoryOp::Project(_) => self.capture_project(),
            };
            self.undo.push(current);
            self.apply_op(next);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

impl Editor {
    pub(super) fn clear_sheet_selections(&mut self, sheet: usize) {
        if self.doc.view_index() == sheet {
            self.selected.clear();
        }
        for pane in &mut self.panes {
            if pane.sheet_index == sheet {
                pane.selected.clear();
            }
        }
    }

    pub(super) fn clear_all_selections(&mut self) {
        self.selected.clear();
        for pane in &mut self.panes {
            pane.selected.clear();
        }
    }
}
