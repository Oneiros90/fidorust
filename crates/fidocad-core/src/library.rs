//! FidoCAD library (.fcl) model and macro expansion.

use crate::geom::Transform;
use crate::primitive::Primitive;
use crate::MACRO_ORIGIN;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MacroDef {
    pub key: String,
    pub name: String,
    pub category: String,
    pub primitives: Vec<Primitive>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub file_stem: String,
    pub standard: bool,
    pub macros: Vec<MacroDef>,
}

#[derive(Clone, Debug, Default)]
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

    pub fn lookup(&self, mc_name: &str) -> Option<(&Library, &MacroDef)> {
        let name = mc_name.trim_start_matches('~');
        if let Some((lib_stem, key)) = name.split_once('.') {
            for lib in &self.libraries {
                if lib.file_stem.eq_ignore_ascii_case(lib_stem) {
                    if let Some(m) = lib.macros.iter().find(|m| m.key.eq_ignore_ascii_case(key)) {
                        return Some((lib, m));
                    }
                }
            }
        }
        // Bare key: stdlib first, then others.
        for lib in &self.libraries {
            if lib.file_stem == "stdlib" {
                if let Some(m) = lib.macros.iter().find(|m| m.key.eq_ignore_ascii_case(name)) {
                    return Some((lib, m));
                }
            }
        }
        for lib in &self.libraries {
            if let Some(m) = lib.macros.iter().find(|m| m.key.eq_ignore_ascii_case(name)) {
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

    pub fn tree(&self) -> Vec<LibTreeNode> {
        self.libraries
            .iter()
            .map(|lib| {
                let mut cats: Vec<(String, Vec<(String, String)>)> = Vec::new();
                for m in &lib.macros {
                    let cat = if m.category.is_empty() {
                        lib.name.clone()
                    } else {
                        m.category.clone()
                    };
                    if let Some((_, items)) = cats.iter_mut().find(|(c, _)| c == &cat) {
                        items.push((m.key.clone(), m.name.clone()));
                    } else {
                        cats.push((cat, vec![(m.key.clone(), m.name.clone())]));
                    }
                }
                LibTreeNode {
                    stem: lib.file_stem.clone(),
                    title: lib.name.clone(),
                    standard: lib.standard,
                    categories: cats
                        .into_iter()
                        .map(|(name, macros)| LibCategory { name, macros })
                        .collect(),
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibTreeNode {
    pub stem: String,
    pub title: String,
    pub standard: bool,
    pub categories: Vec<LibCategory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibCategory {
    pub name: String,
    pub macros: Vec<(String, String)>,
}

pub fn expand_macro(def: &MacroDef, xf: Transform, libs: &LibrarySet, depth: u8) -> Vec<Primitive> {
    if depth > 8 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for p in &def.primitives {
        match p {
            Primitive::Macro { name, .. } => {
                if let Some((_, nested)) = libs.lookup(name) {
                    let mut nested_xf = xf;
                    if let Primitive::Macro {
                        pos,
                        rotations,
                        mirrored,
                        ..
                    } = p
                    {
                        nested_xf.origin = xf.apply(*pos, MACRO_ORIGIN);
                        nested_xf.rotations = (xf.rotations + rotations) % 4;
                        nested_xf.mirrored = xf.mirrored ^ mirrored;
                    }
                    out.extend(expand_macro(nested, nested_xf, libs, depth + 1));
                }
            }
            other => {
                let mut q = other.clone();
                q.apply_transform(xf);
                out.push(q);
            }
        }
    }
    out
}

pub fn expand_primitive(p: &Primitive, libs: &LibrarySet) -> Vec<Primitive> {
    match p {
        Primitive::Macro {
            pos,
            rotations,
            mirrored,
            name,
            ..
        } => {
            if let Some((_, def)) = libs.lookup(name) {
                expand_macro(
                    def,
                    Transform {
                        origin: *pos,
                        rotations: *rotations,
                        mirrored: *mirrored,
                    },
                    libs,
                    0,
                )
            } else {
                vec![p.clone()]
            }
        }
        _ => vec![p.clone()],
    }
}

/// Flatten every macro in a document (for drawing / hit-test).
pub fn flatten(prims: &[Primitive], libs: &LibrarySet) -> Vec<(usize, Primitive)> {
    let mut out = Vec::new();
    for (i, p) in prims.iter().enumerate() {
        match p {
            Primitive::Macro { .. } => {
                for q in expand_primitive(p, libs) {
                    out.push((i, q));
                }
            }
            _ => out.push((i, p.clone())),
        }
    }
    out
}
