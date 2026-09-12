//! Expand component instances and related queries.

use super::{ComponentDef, LibrarySet};
use crate::consts::COMPONENT_MAX_DEPTH;
use crate::geom::{Aabb, Transform};
use crate::layers::LayerId;
use crate::primitive::Primitive;
use crate::COMPONENT_ORIGIN;

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
                let mut nested_out = expand_component(nested, nested_xf, libs, depth + 1);
                if !m.use_component_layers {
                    paint_primitives(&mut nested_out, m.layer);
                }
                out.extend(nested_out);
            } else {
                let mut q = p.clone();
                q.apply_transform(xf);
                out.push(q);
            }
        } else {
            let mut q = p.clone();
            q.apply_transform(xf);
            out.push(q);
        }
    }
    out
}

/// Highest layer index actually drawn: the instance layer, or the definition
/// layers when the instance keeps them (`use_component_layers`).
pub fn max_used_layer_index(prims: &[Primitive], libs: &LibrarySet) -> usize {
    let mut max = 0;
    for p in prims {
        if p.uses_component_layers() {
            for q in expand_primitive(p, libs) {
                max = max.max(q.layer().index());
            }
        } else {
            max = max.max(p.layer().index());
        }
    }
    max
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
            if !m.use_component_layers {
                paint_primitives(&mut out, m.layer);
            }
            out
        } else {
            vec![p.clone()]
        }
    } else {
        vec![p.clone()]
    }
}

/// Top-level `MC` names that do not resolve, with instance counts (case-insensitive grouping).
pub fn unresolved_components(prims: &[Primitive], libs: &LibrarySet) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for p in prims {
        let Primitive::Component(c) = p else {
            continue;
        };
        if libs.lookup(&c.name).is_some() {
            continue;
        }
        if let Some((_, n)) = counts
            .iter_mut()
            .find(|(name, _)| name.eq_ignore_ascii_case(&c.name))
        {
            *n += 1;
        } else {
            counts.push((c.name.clone(), 1));
        }
    }
    counts.sort_by_key(|(name, _)| name.to_ascii_lowercase());
    counts
}

/// Number of top-level `MC` instances that do not resolve in `libs`.
pub fn unresolved_component_count(prims: &[Primitive], libs: &LibrarySet) -> usize {
    unresolved_components(prims, libs)
        .iter()
        .map(|(_, n)| n)
        .sum()
}

/// Assign every primitive (including nested component refs) to `layer`.
pub fn paint_primitives(prims: &mut [Primitive], layer: LayerId) {
    for p in prims {
        p.set_layer(layer);
    }
}

/// True if any primitive is assigned to a layer other than 0.
/// Instances with `use_component_layers` do not count (their stored layer is unused).
pub fn primitives_use_nonzero_layers(prims: &[Primitive]) -> bool {
    prims
        .iter()
        .any(|p| p.assigned_layer().is_some_and(|id| id.0 != 0))
}

/// World AABB of a primitive after expanding nested components.
pub fn expanded_aabb(p: &Primitive, libs: &LibrarySet) -> Aabb {
    let mut bb = Aabb::empty();
    for q in expand_primitive(p, libs) {
        bb.include_aabb(&q.aabb());
    }
    bb
}
