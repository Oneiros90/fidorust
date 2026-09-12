//! User/project library load, import, rename, and remove.

use super::Editor;
use crate::library::{
    drawing_uses_user_library_components, explode_library_instances, primitives_use_nonzero_layers,
    Library, LibraryKind, PROJECT_STEM,
};

impl Editor {
    pub fn load_user_libraries(&mut self, libs: Vec<Library>) {
        self.libs.replace_user_libraries(Vec::new());
        for lib in libs {
            self.libs.add_user_library(lib);
        }
        self.bump_libs_rev();
    }

    pub fn load_local_library(&mut self, lib: Library) {
        self.libs.set_local(lib);
        self.bump_libs_rev();
    }

    pub fn create_user_library(&mut self, title: &str) -> String {
        self.push_undo();
        let title = self.libs.unique_library_title(title.trim());
        let stem = self.libs.unique_stem(&title);
        self.libs.add(Library::empty_user(&stem, title));
        self.bump_libs_rev();
        stem
    }

    pub fn import_library(&mut self, lib: Library) -> String {
        self.push_undo();
        let stem = self.libs.add_user_library(lib);
        self.bump_libs_rev();
        stem
    }

    pub fn rename_library(&mut self, stem: &str, title: &str) -> Option<String> {
        let title = title.trim();
        if title.is_empty() || stem.eq_ignore_ascii_case(PROJECT_STEM) {
            return None;
        }
        let lib = self.libs.library(stem)?;
        if lib.kind != LibraryKind::Local {
            return None;
        }
        if lib.name == title {
            return None;
        }
        self.push_undo();
        if let Some(lib) = self.libs.library_mut(stem) {
            lib.name = title.to_string();
        }
        self.bump_libs_rev();
        Some(stem.to_string())
    }

    pub fn clear_project_library(&mut self) -> bool {
        let Some(project) = self.libs.project() else {
            return false;
        };
        if project.components.is_empty() {
            return false;
        }
        if self
            .component_edit
            .as_ref()
            .is_some_and(|s| s.stem.eq_ignore_ascii_case(PROJECT_STEM))
        {
            self.cancel_component_edit();
        }
        self.push_undo();
        explode_library_instances(&mut self.doc.primitives, &mut self.libs, PROJECT_STEM);
        if let Some(project) = self.libs.project_mut() {
            project.components.clear();
        }
        self.selected.clear();
        self.bump_libs_rev();
        true
    }

    pub fn remove_user_library(&mut self, stem: &str) -> bool {
        if stem.eq_ignore_ascii_case(PROJECT_STEM) {
            return false;
        }
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if lib.kind != LibraryKind::Local {
            return false;
        }
        if self
            .component_edit
            .as_ref()
            .is_some_and(|s| s.stem.eq_ignore_ascii_case(stem))
        {
            self.cancel_component_edit();
        }
        self.push_undo();
        explode_library_instances(&mut self.doc.primitives, &mut self.libs, stem);
        if let Some(pending) = &self.pending_component {
            let prefix = format!("{stem}.");
            if pending.len() > prefix.len() && pending[..prefix.len()].eq_ignore_ascii_case(&prefix)
            {
                self.pending_component = None;
            }
        }
        self.libs.remove_library(stem);
        self.selected.clear();
        self.bump_libs_rev();
        true
    }

    pub fn uses_user_library_components(&self) -> bool {
        drawing_uses_user_library_components(&self.persistent_doc().primitives, &self.libs)
    }

    pub fn local_component_uses_nonzero_layers(&self, stem: &str, key: &str) -> bool {
        let Some(lib) = self.libs.library(stem) else {
            return false;
        };
        if lib.kind != LibraryKind::Local {
            return false;
        }
        lib.find(key)
            .is_some_and(|d| primitives_use_nonzero_layers(&d.primitives))
    }

    pub fn editing_local_component_uses_nonzero_layers(&self) -> bool {
        let Some(session) = &self.component_edit else {
            return false;
        };
        let Some(lib) = self.libs.library(&session.stem) else {
            return false;
        };
        if lib.kind != LibraryKind::Local {
            return false;
        }
        primitives_use_nonzero_layers(&self.doc.primitives)
    }

    pub fn unresolved_component_count(&self) -> usize {
        crate::library::unresolved_component_count(&self.doc.primitives, &self.libs)
    }

    pub fn unresolved_components(&self) -> Vec<(String, usize)> {
        crate::library::unresolved_components(&self.doc.primitives, &self.libs)
    }

    pub fn set_project_library(&mut self, lib: Option<Library>) {
        self.libs
            .set_project(lib.unwrap_or_else(Library::empty_project));
        self.bump_libs_rev();
    }
}
