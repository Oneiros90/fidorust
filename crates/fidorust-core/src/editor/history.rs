//! Undo / redo history (hybrid: sheet geometry vs project-wide).

use super::Editor;
use crate::consts::UNDO_CAP;
use crate::document::{PadLink, PrimitiveDefaults, Sheet};
use crate::layers::LayerSet;
use crate::library::Library;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum HistoryOp {
    Sheet {
        index: usize,
        sheet: Sheet,
        links: Vec<PadLink>,
    },
    Project(ProjectSnap),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ProjectSnap {
    pub layers: LayerSet,
    pub hide_component_origin: bool,
    pub defaults: PrimitiveDefaults,
    pub pcb_mode: bool,
    pub common_node_groups: Vec<Vec<String>>,
    pub pad_links: Vec<PadLink>,
    pub pcb_ext_lines: Vec<String>,
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
            links: self.doc.pad_links.clone(),
        }
    }

    fn capture_project(&self) -> HistoryOp {
        HistoryOp::Project(ProjectSnap {
            layers: self.doc.layers.clone(),
            hide_component_origin: self.doc.hide_component_origin,
            defaults: self.doc.defaults.clone(),
            pcb_mode: self.doc.pcb_mode,
            common_node_groups: self.doc.common_node_groups.clone(),
            pad_links: self.doc.pad_links.clone(),
            pcb_ext_lines: self.doc.pcb_ext_lines.clone(),
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
            HistoryOp::Sheet {
                index,
                sheet,
                links,
            } => {
                if index < self.doc.sheets.len() {
                    self.doc.sheets[index] = sheet;
                }
                self.doc.pad_links = links;
                self.clear_sheet_selections(index);
            }
            HistoryOp::Project(snap) => {
                let libs_changed = self.libs.project() != Some(&snap.project)
                    || self.libs.user_libraries_cloned() != snap.user;
                self.doc.layers = snap.layers;
                self.doc.hide_component_origin = snap.hide_component_origin;
                self.doc.defaults = snap.defaults;
                self.doc.pcb_mode = snap.pcb_mode;
                self.doc.common_node_groups = snap.common_node_groups;
                self.doc.pad_links = snap.pad_links;
                self.doc.pcb_ext_lines = snap.pcb_ext_lines;
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

    /// Geometry / grid-snap of the view sheet, plus the current pad-link list.
    pub(super) fn push_undo(&mut self) {
        self.layer_color_edit = None;
        self.push_undo_op(self.capture_sheet());
    }

    /// Snapshot before a pad-link edit that does not otherwise touch geometry.
    pub fn snapshot_sheet_undo(&mut self) {
        self.push_undo();
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
            self.drag_checkpoint = Some((
                index,
                self.doc.sheets[index].clone(),
                self.doc.pad_links.clone(),
            ));
        }
    }

    pub(super) fn commit_drag_checkpoint(&mut self) {
        let Some((index, checkpoint, links)) = self.drag_checkpoint.take() else {
            return;
        };
        let sheet_changed = self.doc.sheets.get(index) != Some(&checkpoint);
        let links_changed = self.doc.pad_links != links;
        if sheet_changed || links_changed {
            self.push_undo_op(HistoryOp::Sheet {
                index,
                sheet: checkpoint,
                links,
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
                    links: self.doc.pad_links.clone(),
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
                    links: self.doc.pad_links.clone(),
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
