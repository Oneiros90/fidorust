//! FidoCAD library (.fcl) model and component expansion.

use crate::consts::COMPONENT_MAX_DEPTH;
use crate::geom::{Aabb, Point, Transform};
use crate::layers::LayerId;
use crate::primitive::{ComponentRef, Primitive};
use crate::COMPONENT_ORIGIN;
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

    pub fn empty_user(stem: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            file_stem: stem.into(),
            standard: false,
            kind: LibraryKind::Local,
            components: Vec::new(),
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
        let base = sanitize_stem(desired);
        if !self.stem_taken(&base, allow) {
            return base;
        }
        let mut n = 2u32;
        loop {
            let candidate = format!("{base}{n}");
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
            sanitize_stem(&lib.name)
        } else {
            lib.file_stem.clone()
        };
        let stem = self.unique_stem(&old_stem);
        if stem != old_stem {
            rewrite_component_names_in_lib(&mut lib, &old_stem, &stem);
        }
        lib.file_stem = stem.clone();
        self.add(lib);
        stem
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
            let mut out = expand_component(
                def,
                Transform {
                    origin: m.pos,
                    rotations: m.rotations,
                    mirrored: m.mirrored,
                },
                libs,
                0,
            );
            paint_primitives(&mut out, m.layer);
            out
        } else {
            vec![p.clone()]
        }
    } else {
        vec![p.clone()]
    }
}

/// Assign every primitive (including nested component refs) to `layer`.
pub fn paint_primitives(prims: &mut [Primitive], layer: LayerId) {
    for p in prims {
        p.set_layer(layer);
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

fn rewrite_component_names_in_lib(lib: &mut Library, old_stem: &str, new_stem: &str) {
    let keys: Vec<String> = lib.components.iter().map(|c| c.key.clone()).collect();
    for key in keys {
        let from = component_full_name(old_stem, &key);
        let to = component_full_name(new_stem, &key);
        rewrite_component_names_in_defs(&mut lib.components, &from, &to);
    }
}

fn rewrite_component_names_in_defs(defs: &mut [ComponentDef], from: &str, to: &str) {
    for def in defs {
        rewrite_component_names(&mut def.primitives, from, to);
    }
}

pub fn rewrite_library_stem(
    doc_prims: &mut [Primitive],
    libs: &mut LibrarySet,
    old_stem: &str,
    new_stem: &str,
) {
    let keys: Vec<String> = libs
        .library(old_stem)
        .map(|l| l.components.iter().map(|c| c.key.clone()).collect())
        .unwrap_or_default();
    for key in keys {
        let from = component_full_name(old_stem, &key);
        let to = component_full_name(new_stem, &key);
        rewrite_component_names(doc_prims, &from, &to);
        rewrite_component_names_in_libs(libs, &from, &to);
    }
}

pub fn explode_library_instances(
    doc_prims: &mut Vec<Primitive>,
    libs: &mut LibrarySet,
    stem: &str,
) {
    let keys: Vec<String> = libs
        .library(stem)
        .map(|l| l.components.iter().map(|c| c.key.clone()).collect())
        .unwrap_or_default();
    for key in keys {
        let full = component_full_name(stem, &key);
        explode_named_everywhere(doc_prims, libs, &full);
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

fn primitives_use_user_components(prims: &[Primitive], libs: &LibrarySet) -> bool {
    prims.iter().any(|p| {
        let Primitive::Component(c) = p else {
            return false;
        };
        libs.lookup(&c.name)
            .is_some_and(|(lib, _)| lib.kind == LibraryKind::Local)
    })
}

/// True if the drawing or project-library defs reference a user (non-project) library.
pub fn drawing_uses_user_library_components(prims: &[Primitive], libs: &LibrarySet) -> bool {
    if primitives_use_user_components(prims, libs) {
        return true;
    }
    if let Some(project) = libs.project() {
        for def in &project.components {
            if primitives_use_user_components(&def.primitives, libs) {
                return true;
            }
        }
    }
    false
}

fn collect_used_user_defs(
    prims: &[Primitive],
    libs: &LibrarySet,
    out: &mut Vec<(String, ComponentDef)>,
) {
    for p in prims {
        let Primitive::Component(c) = p else {
            continue;
        };
        let Some((lib, def)) = libs.lookup(&c.name) else {
            continue;
        };
        if lib.kind == LibraryKind::Local {
            let already = out
                .iter()
                .any(|(stem, d)| stem.eq_ignore_ascii_case(&lib.file_stem) && d.key == def.key);
            if !already {
                out.push((lib.file_stem.clone(), def.clone()));
                collect_used_user_defs(&def.primitives, libs, out);
            }
        } else if lib.kind == LibraryKind::Project {
            collect_used_user_defs(&def.primitives, libs, out);
        }
    }
}

/// Copy used user-library defs into the project library (clone). Rewrites MC names.
pub fn fold_user_components_into_project(doc_prims: &mut Vec<Primitive>, libs: &mut LibrarySet) {
    libs.ensure_user_libraries();
    let mut used = Vec::new();
    collect_used_user_defs(doc_prims, libs, &mut used);
    if let Some(project) = libs.project() {
        for def in &project.components {
            collect_used_user_defs(&def.primitives, libs, &mut used);
        }
    }
    let mut mapping: Vec<(String, String)> = Vec::new();
    for (stem, def) in used {
        let old_full = component_full_name(&stem, &def.key);
        let dest_key = {
            let project = match libs.project_mut() {
                Some(p) => p,
                None => continue,
            };
            if project.find(&def.key).is_none() {
                def.key.clone()
            } else {
                project.next_key()
            }
        };
        let new_full = component_full_name(PROJECT_STEM, &dest_key);
        if let Some(project) = libs.project_mut() {
            let mut moved = def;
            moved.key = dest_key;
            project.components.push(moved);
        }
        if old_full != new_full {
            mapping.push((old_full, new_full));
        }
    }
    for (from, to) in mapping {
        rewrite_component_names(doc_prims, &from, &to);
        rewrite_component_names_in_libs(libs, &from, &to);
    }
}

fn explode_local_refs(prims: &mut Vec<Primitive>, libs: &LibrarySet) {
    let mut out = Vec::with_capacity(prims.len());
    for p in prims.drain(..) {
        if let Primitive::Component(ComponentRef { name, .. }) = &p {
            if libs
                .lookup(name)
                .is_some_and(|(lib, _)| lib.kind == LibraryKind::Local)
            {
                out.extend(expand_primitive(&p, libs));
                continue;
            }
        }
        out.push(p);
    }
    *prims = out;
}

/// Expand every user-library instance in the drawing and project defs (clone).
pub fn explode_user_components_for_save(doc_prims: &mut Vec<Primitive>, libs: &mut LibrarySet) {
    let snapshot = libs.clone();
    if let Some(project) = libs.project_mut() {
        for def in &mut project.components {
            explode_local_refs(&mut def.primitives, &snapshot);
        }
    }
    explode_local_refs(doc_prims, &snapshot);
}
