//! Save policies: fold user libraries into the project, or explode them.

use std::collections::HashMap;

use super::expand::expand_primitive;
use super::rewrite::{rewrite_component_names_in_libs_map, rewrite_component_names_map};
use super::{component_full_name, ComponentDef, LibraryKind, LibrarySet, PROJECT_STEM};
use crate::document::Document;
use crate::primitive::{ComponentRef, Primitive};

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

struct UsedUserDef {
    stem: String,
    def: ComponentDef,
    spellings: Vec<String>,
}

fn push_spelling(spellings: &mut Vec<String>, name: &str) {
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    if spellings.iter().any(|s| s.eq_ignore_ascii_case(name)) {
        return;
    }
    spellings.push(name.to_string());
}

fn collect_used_user_defs(prims: &[Primitive], libs: &LibrarySet, out: &mut Vec<UsedUserDef>) {
    for p in prims {
        let Primitive::Component(c) = p else {
            continue;
        };
        let Some((lib, def)) = libs.lookup_raw(&c.name) else {
            continue;
        };
        if lib.kind == LibraryKind::Local {
            let spellings_from = |lib: &super::Library, def: &ComponentDef, name: &str| {
                let mut spellings = Vec::new();
                for prefix in lib.mc_prefixes() {
                    push_spelling(&mut spellings, &component_full_name(&prefix, &def.key));
                }
                push_spelling(&mut spellings, name);
                spellings
            };
            if let Some(existing) = out.iter_mut().find(|u| {
                u.stem.eq_ignore_ascii_case(&lib.file_stem)
                    && u.def.key.eq_ignore_ascii_case(&def.key)
            }) {
                for spelling in spellings_from(lib, def, &c.name) {
                    push_spelling(&mut existing.spellings, &spelling);
                }
            } else {
                let nested = def.primitives.clone();
                out.push(UsedUserDef {
                    stem: lib.file_stem.clone(),
                    def: def.clone(),
                    spellings: spellings_from(lib, def, &c.name),
                });
                collect_used_user_defs(&nested, libs, out);
            }
        } else if lib.kind == LibraryKind::Project {
            collect_used_user_defs(&def.primitives, libs, out);
        }
    }
}

fn collect_groups(groups: &[&[Primitive]], libs: &LibrarySet) -> Vec<UsedUserDef> {
    let mut used = Vec::new();
    for group in groups {
        collect_used_user_defs(group, libs, &mut used);
    }
    if let Some(project) = libs.project() {
        for def in &project.components {
            collect_used_user_defs(&def.primitives, libs, &mut used);
        }
    }
    used
}

fn install_folded_copies(libs: &mut LibrarySet, used: Vec<UsedUserDef>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for item in used {
        let dest_key = {
            let Some(project) = libs.project_mut() else {
                continue;
            };
            if let Some(existing) = project
                .find(&item.def.key)
                .filter(|existing| same_saved_component(existing, &item.def))
            {
                existing.key.clone()
            } else if project.find(&item.def.key).is_none() {
                let key = item.def.key.clone();
                let mut moved = item.def.clone();
                moved.key = key.clone();
                project.components.push(moved);
                key
            } else {
                let key = project.next_key();
                let mut moved = item.def.clone();
                moved.key = key.clone();
                project.components.push(moved);
                key
            }
        };
        let new_full = component_full_name(PROJECT_STEM, &dest_key);
        let new_key = new_full.to_ascii_lowercase();
        for spelling in &item.spellings {
            let from = spelling.to_ascii_lowercase();
            if from != new_key {
                map.entry(from).or_insert_with(|| new_full.clone());
            }
        }
    }
    map
}

fn fold_groups(groups: &mut [&mut [Primitive]], libs: &mut LibrarySet) {
    libs.ensure_user_libraries();
    let owned: Vec<Vec<Primitive>> = groups.iter().map(|g| g.to_vec()).collect();
    let slices: Vec<&[Primitive]> = owned.iter().map(|g| g.as_slice()).collect();
    let used = collect_groups(&slices, libs);
    let map = install_folded_copies(libs, used);
    for group in groups.iter_mut() {
        rewrite_component_names_map(group, &map);
    }
    rewrite_component_names_in_libs_map(libs, &map);
}

/// Copy used user-library defs into the project library (clone). Rewrites MC names.
pub fn fold_user_components_into_project(doc_prims: &mut [Primitive], libs: &mut LibrarySet) {
    fold_groups(&mut [doc_prims], libs);
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

pub fn document_uses_user_library_components(doc: &Document, libs: &LibrarySet) -> bool {
    doc.sheets
        .iter()
        .any(|s| primitives_use_user_components(&s.primitives, libs))
        || drawing_uses_user_library_components(&[], libs)
}

pub fn fold_user_components_in_document(doc: &mut Document, libs: &mut LibrarySet) {
    libs.ensure_user_libraries();
    let owned: Vec<Vec<Primitive>> = doc.sheets.iter().map(|s| s.primitives.clone()).collect();
    let slices: Vec<&[Primitive]> = owned.iter().map(|p| p.as_slice()).collect();
    let used = collect_groups(&slices, libs);
    let map = install_folded_copies(libs, used);
    for sheet in &mut doc.sheets {
        rewrite_component_names_map(&mut sheet.primitives, &map);
    }
    rewrite_component_names_in_libs_map(libs, &map);
}

/// Point drawing and project-library refs at `project.KEY` when lookup already
/// uses an equivalent project copy. Makes a skipped save-prompt actually portable.
pub fn rewrite_equivalent_user_refs_to_project(doc: &mut Document, libs: &mut LibrarySet) -> bool {
    let map = equivalent_user_to_project_map(doc, libs);
    if map.is_empty() {
        return false;
    }
    for sheet in &mut doc.sheets {
        rewrite_component_names_map(&mut sheet.primitives, &map);
    }
    if let Some(project) = libs.project_mut() {
        for def in &mut project.components {
            rewrite_component_names_map(&mut def.primitives, &map);
        }
    }
    true
}

pub(crate) fn has_equivalent_user_refs_to_rewrite(doc: &Document, libs: &LibrarySet) -> bool {
    !equivalent_user_to_project_map(doc, libs).is_empty()
}

fn equivalent_user_to_project_map(doc: &Document, libs: &LibrarySet) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut consider = |prims: &[Primitive]| {
        for p in prims {
            let Primitive::Component(c) = p else {
                continue;
            };
            let Some(to) = canonical_project_mc_name(&c.name, libs) else {
                continue;
            };
            map.entry(c.name.to_ascii_lowercase()).or_insert(to);
        }
    };
    for sheet in &doc.sheets {
        consider(&sheet.primitives);
    }
    if let Some(project) = libs.project() {
        for def in &project.components {
            consider(&def.primitives);
        }
    }
    map
}

fn canonical_project_mc_name(mc_name: &str, libs: &LibrarySet) -> Option<String> {
    let (lib, def) = libs.lookup(mc_name)?;
    if lib.kind != LibraryKind::Project {
        return None;
    }
    let canonical = component_full_name(PROJECT_STEM, &def.key);
    if mc_name.eq_ignore_ascii_case(&canonical) {
        None
    } else {
        Some(canonical)
    }
}

/// True when `a` and `b` are the same saved component (key, name, and body).
/// Nested `MC` names compare by macro code, so `Lib.C01` and `project.C01` match.
pub(super) fn same_saved_component(a: &ComponentDef, b: &ComponentDef) -> bool {
    a.key.eq_ignore_ascii_case(&b.key)
        && a.name == b.name
        && primitives_match(&a.primitives, &b.primitives)
}

fn primitives_match(a: &[Primitive], b: &[Primitive]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| primitive_match(p, q))
}

fn primitive_match(a: &Primitive, b: &Primitive) -> bool {
    match (a, b) {
        (Primitive::Component(left), Primitive::Component(right)) => {
            component_refs_match(left, right)
        }
        _ => a == b,
    }
}

fn component_refs_match(a: &ComponentRef, b: &ComponentRef) -> bool {
    a.pos == b.pos
        && a.rotations == b.rotations
        && a.mirrored == b.mirrored
        && a.layer == b.layer
        && a.use_component_layers == b.use_component_layers
        && a.standard == b.standard
        && macro_code(&a.name).eq_ignore_ascii_case(macro_code(&b.name))
}

fn macro_code(name: &str) -> &str {
    name.rsplit_once('.').map(|(_, key)| key).unwrap_or(name)
}

pub fn explode_user_components_in_document(doc: &mut Document, libs: &mut LibrarySet) {
    let snapshot = libs.clone();
    if let Some(project) = libs.project_mut() {
        for def in &mut project.components {
            explode_local_refs(&mut def.primitives, &snapshot);
        }
    }
    for sheet in &mut doc.sheets {
        explode_local_refs(&mut sheet.primitives, &snapshot);
    }
}
