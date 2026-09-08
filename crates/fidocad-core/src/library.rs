//! FidoCAD library (.fcl) model and component expansion.

use crate::consts::COMPONENT_MAX_DEPTH;
use crate::geom::{Aabb, Point, Transform};
use crate::primitive::{ComponentRef, Primitive};
use crate::COMPONENT_ORIGIN;
use serde::{Deserialize, Serialize};

pub const PROJECT_STEM: &str = "project";
pub const LOCAL_STEM: &str = "local";

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

    pub fn stem(self) -> Option<&'static str> {
        match self {
            Self::Builtin => None,
            Self::Project => Some(PROJECT_STEM),
            Self::Local => Some(LOCAL_STEM),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserLibraryTarget {
    Project,
    Local,
}

impl UserLibraryTarget {
    pub fn stem(self) -> &'static str {
        match self {
            Self::Project => PROJECT_STEM,
            Self::Local => LOCAL_STEM,
        }
    }

    pub fn kind(self) -> LibraryKind {
        match self {
            Self::Project => LibraryKind::Project,
            Self::Local => LibraryKind::Local,
        }
    }

    pub fn from_stem(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case(PROJECT_STEM) {
            Some(Self::Project)
        } else if s.eq_ignore_ascii_case(LOCAL_STEM) || s.eq_ignore_ascii_case("device") {
            Some(Self::Local)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentDef {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
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
}

impl Default for Library {
    fn default() -> Self {
        Self {
            name: String::new(),
            file_stem: String::new(),
            standard: false,
            kind: LibraryKind::Builtin,
            components: Vec::new(),
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
        }
    }

    pub fn empty_local() -> Self {
        Self {
            name: "Local library".into(),
            file_stem: LOCAL_STEM.into(),
            standard: false,
            kind: LibraryKind::Local,
            components: Vec::new(),
        }
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
        if !self.has_stem(LOCAL_STEM) {
            self.add(Library::empty_local());
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

    pub fn lookup(&self, mc_name: &str) -> Option<(&Library, &ComponentDef)> {
        let name = mc_name.trim_start_matches('~');
        if let Some((lib_stem, key)) = name.split_once('.') {
            for lib in &self.libraries {
                if lib.file_stem.eq_ignore_ascii_case(lib_stem) {
                    if let Some(m) = lib.find(key) {
                        return Some((lib, m));
                    }
                }
            }
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
                        description: m.description.clone(),
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
    pub description: String,
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

pub fn expand_component(
    def: &ComponentDef,
    xf: Transform,
    libs: &LibrarySet,
    depth: u8,
) -> Vec<Primitive> {
    if depth > COMPONENT_MAX_DEPTH {
        return Vec::new();
    }
    let mut out = Vec::new();
    for p in &def.primitives {
        if let Primitive::Component(m) = p {
            if let Some((_, nested)) = libs.lookup(&m.name) {
                let mut nested_xf = xf;
                nested_xf.origin = xf.apply(m.pos, COMPONENT_ORIGIN);
                nested_xf.rotations = (xf.rotations + m.rotations) % 4;
                nested_xf.mirrored = xf.mirrored ^ m.mirrored;
                out.extend(expand_component(nested, nested_xf, libs, depth + 1));
            }
        } else {
            let mut q = p.clone();
            q.apply_transform(xf);
            out.push(q);
        }
    }
    out
}

pub fn expand_primitive(p: &Primitive, libs: &LibrarySet) -> Vec<Primitive> {
    if let Primitive::Component(m) = p {
        if let Some((_, def)) = libs.lookup(&m.name) {
            expand_component(
                def,
                Transform {
                    origin: m.pos,
                    rotations: m.rotations,
                    mirrored: m.mirrored,
                },
                libs,
                0,
            )
        } else {
            vec![p.clone()]
        }
    } else {
        vec![p.clone()]
    }
}

/// World AABB of a primitive after expanding nested components.
pub fn expanded_aabb(p: &Primitive, libs: &LibrarySet) -> Aabb {
    let mut bb = Aabb::empty();
    for q in expand_primitive(p, libs) {
        bb.include_aabb(&q.aabb());
    }
    bb
}

pub fn rewrite_component_names(prims: &mut [Primitive], from: &str, to: &str) {
    for p in prims {
        if let Primitive::Component(c) = p {
            if c.name.eq_ignore_ascii_case(from) {
                c.name = to.to_string();
            }
        }
    }
}

pub fn rewrite_component_names_in_libs(libs: &mut LibrarySet, from: &str, to: &str) {
    for lib in &mut libs.libraries {
        for def in &mut lib.components {
            rewrite_component_names(&mut def.primitives, from, to);
        }
    }
}

/// Replace every instance of `full_name` with its expanded primitives.
pub fn explode_named(prims: &mut Vec<Primitive>, full_name: &str, libs: &LibrarySet) {
    let mut out = Vec::with_capacity(prims.len());
    for p in prims.drain(..) {
        if let Primitive::Component(ComponentRef { name, .. }) = &p {
            if name.eq_ignore_ascii_case(full_name) {
                out.extend(expand_primitive(&p, libs));
                continue;
            }
        }
        out.push(p);
    }
    *prims = out;
}

pub fn explode_named_everywhere(
    doc_prims: &mut Vec<Primitive>,
    libs: &mut LibrarySet,
    full_name: &str,
) {
    let snapshot = libs.clone();
    for lib in &mut libs.libraries {
        for def in &mut lib.components {
            explode_named(&mut def.primitives, full_name, &snapshot);
        }
    }
    explode_named(doc_prims, full_name, &snapshot);
}

pub fn translate_primitives(prims: &mut [Primitive], dx: i32, dy: i32) {
    if dx == 0 && dy == 0 {
        return;
    }
    let delta = Point::new(dx, dy);
    for p in prims {
        p.transform(|q| q + delta);
    }
}
