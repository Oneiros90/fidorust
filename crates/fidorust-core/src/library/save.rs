//! Save policies: fold user libraries into the project, or explode them.

use super::expand::expand_primitive;
use super::rewrite::{rewrite_component_names, rewrite_component_names_in_libs};
use super::{component_full_name, ComponentDef, LibraryKind, LibrarySet, PROJECT_STEM};
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
pub fn fold_user_components_into_project(doc_prims: &mut [Primitive], libs: &mut LibrarySet) {
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
