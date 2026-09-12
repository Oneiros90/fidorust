//! Rename, explode, and translate component instances.

use super::expand::expand_primitive;
use super::{component_full_name, ComponentDef, Library, LibrarySet};
use crate::geom::Point;
use crate::primitive::{ComponentRef, Primitive};

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

pub(crate) fn rewrite_component_names_in_lib(lib: &mut Library, old_stem: &str, new_stem: &str) {
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
