//! FidoCAD library (.fcl) model and component expansion.

mod expand;
mod rewrite;
mod save;

pub use expand::{
    expand_component, expand_primitive, expanded_aabb, max_used_layer_index, paint_primitives,
    primitives_use_nonzero_layers, unresolved_component_count, unresolved_components,
};
pub use rewrite::{
    explode_library_instances, explode_named, explode_named_everywhere, rewrite_component_names,
    rewrite_component_names_in_libs, rewrite_library_stem, translate_primitives,
};
pub use save::{
    drawing_uses_user_library_components, explode_user_components_for_save,
    fold_user_components_into_project,
};

use crate::primitive::Primitive;
use serde::{Deserialize, Serialize};

pub const PROJECT_STEM: &str = "project";
pub const LOCAL_STEM: &str = "local";

const RESERVED_STEMS: &[&str] = &[PROJECT_STEM, "stdlib", "PCB", "lib1"];

pub fn sanitize_stem(raw: &str) -> String {
    let mut out: String = raw.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if out.is_empty() {
        return "user".into();
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, 'L');
    }
    out
}

/// Alphanumeric-only key used to compare FidoCAD library prefixes that may contain spaces.
fn stem_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn normalized_stems_eq(a: &str, b: &str) -> bool {
    let ka = stem_key(a);
    !ka.is_empty() && ka == stem_key(b)
}

/// Filename without directory or `.fcl`, as used by FidoCadJ as the MC prefix.
pub fn fcl_filename_stem(filename: &str) -> String {
    let name = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    if name.len() >= 4 && name[name.len() - 4..].eq_ignore_ascii_case(".fcl") {
        name[..name.len() - 4].trim().to_string()
    } else {
        name.to_string()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LibraryKind {
    #[default]
    Builtin,
    Project,
    Local,
}

impl LibraryKind {
    pub fn writable(self) -> bool {
        !matches!(self, Self::Builtin)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentDef {
    pub key: String,
    pub name: String,
    pub category: String,
    pub primitives: Vec<Primitive>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub file_stem: String,
    pub standard: bool,
    #[serde(default)]
    pub kind: LibraryKind,
    pub components: Vec<ComponentDef>,
    /// Extra MC prefixes (typically the original `.fcl` filename when it differs from the title).
    #[serde(default)]
    pub aliases: Vec<String>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            name: String::new(),
            file_stem: String::new(),
            standard: false,
            kind: LibraryKind::Builtin,
            components: Vec::new(),
            aliases: Vec::new(),
        }
    }
}

impl Library {
    pub fn empty_project() -> Self {
        Self {
            name: "Project library".into(),
            file_stem: PROJECT_STEM.into(),
            standard: false,
            kind: LibraryKind::Project,
            components: Vec::new(),
            aliases: Vec::new(),
        }
    }

    pub fn empty_user(stem: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            file_stem: stem.into(),
            standard: false,
            kind: LibraryKind::Local,
            components: Vec::new(),
            aliases: Vec::new(),
        }
    }

    pub fn empty_local() -> Self {
        Self::empty_user(LOCAL_STEM, "Local library")
    }

    pub fn writable(&self) -> bool {
        self.kind.writable()
    }

    pub fn next_key(&self) -> String {
        let mut n = 1u32;
        loop {
            let key = format!("C{n:02}");
            if !self
                .components
                .iter()
                .any(|c| c.key.eq_ignore_ascii_case(&key))
            {
                return key;
            }
            n += 1;
        }
    }

    pub fn unique_display_name(&self, base: &str) -> String {
        if !self.components.iter().any(|c| c.name == base) {
            return base.to_string();
        }
        let mut n = 2u32;
        loop {
            let name = format!("{base} {n}");
            if !self.components.iter().any(|c| c.name == name) {
                return name;
            }
            n += 1;
        }
    }

    pub fn find(&self, key: &str) -> Option<&ComponentDef> {
        self.components
            .iter()
            .find(|c| c.key.eq_ignore_ascii_case(key))
    }

    pub fn find_mut(&mut self, key: &str) -> Option<&mut ComponentDef> {
        self.components
            .iter_mut()
            .find(|c| c.key.eq_ignore_ascii_case(key))
    }

    /// Record the `.fcl` filename as an extra lookup prefix when it differs from title/stem.
    pub fn add_filename_alias(&mut self, filename: &str) {
        let stem = fcl_filename_stem(filename);
        if stem.is_empty() {
            return;
        }
        if self.file_stem.eq_ignore_ascii_case(&stem) || self.name.eq_ignore_ascii_case(&stem) {
            return;
        }
        if self.aliases.iter().any(|a| a.eq_ignore_ascii_case(&stem)) {
            return;
        }
        self.aliases.push(stem);
    }

    fn prefix_candidates(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.file_stem.as_str())
            .chain(std::iter::once(self.name.as_str()))
            .chain(self.aliases.iter().map(String::as_str))
            .filter(|s| !s.is_empty())
    }

    fn matches_mc_prefix_exact(&self, prefix: &str) -> bool {
        self.prefix_candidates()
            .any(|s| s.eq_ignore_ascii_case(prefix))
    }

    fn matches_mc_prefix_normalized(&self, prefix: &str) -> bool {
        self.prefix_candidates()
            .any(|s| normalized_stems_eq(s, prefix))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibrarySet {
    pub libraries: Vec<Library>,
}

impl LibrarySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, lib: Library) {
        self.libraries.push(lib);
    }

    pub fn ensure_user_libraries(&mut self) {
        if !self.has_stem(PROJECT_STEM) {
            self.add(Library::empty_project());
        }
    }

    pub fn has_stem(&self, stem: &str) -> bool {
        self.libraries
            .iter()
            .any(|l| l.file_stem.eq_ignore_ascii_case(stem))
    }

    pub fn library(&self, stem: &str) -> Option<&Library> {
        self.libraries
            .iter()
            .find(|l| l.file_stem.eq_ignore_ascii_case(stem))
    }

    pub fn library_mut(&mut self, stem: &str) -> Option<&mut Library> {
        self.libraries
            .iter_mut()
            .find(|l| l.file_stem.eq_ignore_ascii_case(stem))
    }

    pub fn project(&self) -> Option<&Library> {
        self.library(PROJECT_STEM)
    }

    pub fn project_mut(&mut self) -> Option<&mut Library> {
        self.library_mut(PROJECT_STEM)
    }

    pub fn local(&self) -> Option<&Library> {
        self.library(LOCAL_STEM)
    }

    pub fn local_mut(&mut self) -> Option<&mut Library> {
        self.library_mut(LOCAL_STEM)
    }

    pub fn set_project(&mut self, mut lib: Library) {
        lib.file_stem = PROJECT_STEM.into();
        lib.kind = LibraryKind::Project;
        lib.standard = false;
        if let Some(slot) = self.library_mut(PROJECT_STEM) {
            *slot = lib;
        } else {
            self.add(lib);
        }
    }

    pub fn set_local(&mut self, mut lib: Library) {
        lib.file_stem = LOCAL_STEM.into();
        lib.kind = LibraryKind::Local;
        lib.standard = false;
        if let Some(slot) = self.library_mut(LOCAL_STEM) {
            *slot = lib;
        } else {
            self.add(lib);
        }
    }

    pub fn user_libraries(&self) -> impl Iterator<Item = &Library> {
        self.libraries
            .iter()
            .filter(|l| l.kind == LibraryKind::Local)
    }

    pub fn user_libraries_cloned(&self) -> Vec<Library> {
        self.user_libraries().cloned().collect()
    }

    pub fn replace_user_libraries(&mut self, libs: Vec<Library>) {
        self.libraries.retain(|l| l.kind != LibraryKind::Local);
        for mut lib in libs {
            lib.kind = LibraryKind::Local;
            lib.standard = false;
            self.add(lib);
        }
    }

    pub fn remove_library(&mut self, stem: &str) -> bool {
        if stem.eq_ignore_ascii_case(PROJECT_STEM) {
            return false;
        }
        let before = self.libraries.len();
        self.libraries
            .retain(|l| !l.file_stem.eq_ignore_ascii_case(stem));
        before != self.libraries.len()
    }

    pub fn unique_library_title(&self, base: &str) -> String {
        if !self.libraries.iter().any(|l| l.name == base) {
            return base.to_string();
        }
        let mut n = 2u32;
        loop {
            let name = format!("{base} {n}");
            if !self.libraries.iter().any(|l| l.name == name) {
                return name;
            }
            n += 1;
        }
    }

    pub fn unique_stem(&self, desired: &str) -> String {
        self.unique_stem_excluding(desired, "")
    }

    pub fn unique_stem_excluding(&self, desired: &str, allow: &str) -> String {
        let base = desired.trim();
        let base = if base.is_empty() { "user" } else { base };
        if !self.stem_taken(base, allow) {
            return base.to_string();
        }
        let mut n = 2u32;
        loop {
            let candidate = format!("{base} {n}");
            if !self.stem_taken(&candidate, allow) {
                return candidate;
            }
            n += 1;
        }
    }

    fn stem_taken(&self, stem: &str, allow: &str) -> bool {
        if !allow.is_empty() && stem.eq_ignore_ascii_case(allow) {
            return false;
        }
        RESERVED_STEMS.iter().any(|r| r.eq_ignore_ascii_case(stem)) || self.has_stem(stem)
    }

    pub fn add_user_library(&mut self, mut lib: Library) -> String {
        lib.kind = LibraryKind::Local;
        lib.standard = false;
        let old_stem = if lib.file_stem.is_empty() {
            if lib.name.trim().is_empty() {
                "user".into()
            } else {
                lib.name.clone()
            }
        } else {
            lib.file_stem.clone()
        };
        let stem = self.unique_stem(&old_stem);
        if stem != old_stem {
            rewrite::rewrite_component_names_in_lib(&mut lib, &old_stem, &stem);
        }
        lib.file_stem = stem.clone();
        if lib.name.trim().is_empty() {
            lib.name = stem.clone();
        }
        self.add(lib);
        stem
    }

    pub fn lookup(&self, mc_name: &str) -> Option<(&Library, &ComponentDef)> {
        let name = mc_name.trim_start_matches('~');
        if let Some((lib_stem, key)) = name.split_once('.') {
            return self.lookup_prefixed(lib_stem, key);
        }
        for lib in &self.libraries {
            if lib.file_stem == "stdlib" {
                if let Some(m) = lib.find(name) {
                    return Some((lib, m));
                }
            }
        }
        for lib in &self.libraries {
            if let Some(m) = lib.find(name) {
                return Some((lib, m));
            }
        }
        None
    }

    fn lookup_prefixed<'a>(
        &'a self,
        lib_stem: &str,
        key: &str,
    ) -> Option<(&'a Library, &'a ComponentDef)> {
        for lib in &self.libraries {
            if lib.matches_mc_prefix_exact(lib_stem) {
                if let Some(m) = lib.find(key) {
                    return Some((lib, m));
                }
            }
        }
        for lib in &self.libraries {
            if lib.matches_mc_prefix_normalized(lib_stem) {
                if let Some(m) = lib.find(key) {
                    return Some((lib, m));
                }
            }
        }
        None
    }

    pub fn is_standard(&self, mc_name: &str) -> bool {
        self.lookup(mc_name)
            .map(|(lib, _)| lib.standard)
            .unwrap_or(false)
    }

    pub fn is_writable_ref(&self, mc_name: &str) -> bool {
        self.lookup(mc_name)
            .map(|(lib, _)| lib.writable())
            .unwrap_or(false)
    }

    pub fn tree(&self) -> Vec<LibTreeNode> {
        self.libraries
            .iter()
            .map(|lib| {
                let mut cats: Vec<(String, Vec<LibComponentItem>)> = Vec::new();
                for m in &lib.components {
                    let cat = if m.category.is_empty() {
                        lib.name.clone()
                    } else {
                        m.category.clone()
                    };
                    let item = LibComponentItem {
                        key: m.key.clone(),
                        name: m.name.clone(),
                    };
                    if let Some((_, items)) = cats.iter_mut().find(|(c, _)| c == &cat) {
                        items.push(item);
                    } else {
                        cats.push((cat, vec![item]));
                    }
                }
                LibTreeNode {
                    stem: lib.file_stem.clone(),
                    title: lib.name.clone(),
                    standard: lib.standard,
                    kind: lib.kind,
                    writable: lib.writable(),
                    categories: cats
                        .into_iter()
                        .map(|(name, components)| LibCategory { name, components })
                        .collect(),
                }
            })
            .collect()
    }

    pub fn user_library_list(&self) -> Vec<UserLibraryInfo> {
        self.libraries
            .iter()
            .filter(|l| l.writable())
            .map(|l| UserLibraryInfo {
                stem: l.file_stem.clone(),
                title: l.name.clone(),
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibTreeNode {
    pub stem: String,
    pub title: String,
    pub standard: bool,
    pub kind: LibraryKind,
    pub writable: bool,
    pub categories: Vec<LibCategory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibCategory {
    pub name: String,
    pub components: Vec<LibComponentItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibComponentItem {
    pub key: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserLibraryInfo {
    pub stem: String,
    pub title: String,
}

pub fn component_full_name(stem: &str, key: &str) -> String {
    if stem == "stdlib" {
        key.to_string()
    } else {
        format!("{stem}.{key}")
    }
}
