//! Object properties dialog: common attributes across a multi-selection (FidoCAD 0.96).

use crate::layers::LayerId;
use crate::primitive::{PadStyle, Primitive};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Editable property identifiers (FidoCAD internal names).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropField {
    Filled,
    Layer,
    Thickness,
    SizeX,
    SizeY,
    IntDiam,
    PadStyle,
    Text,
    FontFace,
    FontHeight,
    FontWidth,
    RotationAngle,
    Bold,
    Italic,
    Mirrored,
    Underlined,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropFieldKind {
    Bool,
    Int { min: i32, max: i32 },
    String,
    Layer,
    PadStyle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum PropFieldValue {
    Unset,
    Bool { value: bool },
    Int { value: i32 },
    String { value: String },
    Layer { value: i32 },
    PadStyle { value: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropFormField {
    pub id: PropField,
    pub kind: PropFieldKind,
    pub value: PropFieldValue,
    /// When true the field is shown but cannot be applied (e.g. layer on macros).
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropPatch {
    #[serde(default)]
    pub filled: Option<bool>,
    #[serde(default)]
    pub layer: Option<i32>,
    #[serde(default)]
    pub thickness: Option<i32>,
    #[serde(default)]
    pub size_x: Option<i32>,
    #[serde(default)]
    pub size_y: Option<i32>,
    #[serde(default)]
    pub int_diam: Option<i32>,
    #[serde(default)]
    pub pad_style: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub font_face: Option<String>,
    #[serde(default)]
    pub font_height: Option<i32>,
    #[serde(default)]
    pub font_width: Option<i32>,
    #[serde(default)]
    pub rotation_angle: Option<i32>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default)]
    pub mirrored: Option<bool>,
    #[serde(default)]
    pub underlined: Option<bool>,
}

const STYLE_BOLD: u32 = 1;
const STYLE_ITALIC: u32 = 2;
const STYLE_MIRRORED: u32 = 4;
const STYLE_UNDERLINE: u32 = 8;

fn attrib_order(first: &Primitive) -> Vec<PropField> {
    match first {
        Primitive::Rect { .. } | Primitive::Poly { .. } | Primitive::Ellipse { .. } => {
            vec![PropField::Filled]
        }
        Primitive::PcbTrack { .. } => vec![PropField::Thickness],
        Primitive::PcbPad { .. } => {
            vec![
                PropField::SizeX,
                PropField::SizeY,
                PropField::IntDiam,
                PropField::PadStyle,
            ]
        }
        Primitive::Text { .. } => vec![
            PropField::Text,
            PropField::FontFace,
            PropField::FontHeight,
            PropField::FontWidth,
            PropField::RotationAngle,
            PropField::Bold,
            PropField::Italic,
            PropField::Mirrored,
            PropField::Underlined,
        ],
        Primitive::Line { .. }
        | Primitive::Bezier { .. }
        | Primitive::Connection { .. }
        | Primitive::Macro { .. } => vec![],
    }
}

fn field_kind(field: PropField) -> PropFieldKind {
    match field {
        PropField::Filled
        | PropField::Bold
        | PropField::Italic
        | PropField::Mirrored
        | PropField::Underlined => PropFieldKind::Bool,
        PropField::Thickness => PropFieldKind::Int { min: 1, max: 100 },
        PropField::SizeX | PropField::SizeY | PropField::IntDiam => {
            PropFieldKind::Int { min: 2, max: 100 }
        }
        PropField::FontHeight | PropField::FontWidth => PropFieldKind::Int { min: 2, max: 100 },
        PropField::RotationAngle => PropFieldKind::Int { min: 0, max: 359 },
        PropField::Text | PropField::FontFace => PropFieldKind::String,
        PropField::Layer => PropFieldKind::Layer,
        PropField::PadStyle => PropFieldKind::PadStyle,
    }
}

fn read_field(p: &Primitive, field: PropField) -> Option<PropFieldValue> {
    match field {
        PropField::Filled => match p {
            Primitive::Rect { filled, .. }
            | Primitive::Poly { filled, .. }
            | Primitive::Ellipse { filled, .. } => {
                Some(PropFieldValue::Bool { value: *filled })
            }
            _ => None,
        },
        PropField::Layer => match p {
            Primitive::Macro { .. } => Some(PropFieldValue::Layer { value: 0 }),
            _ => Some(PropFieldValue::Layer {
                value: p.layer().0 as i32,
            }),
        },
        PropField::Thickness => match p {
            Primitive::PcbTrack { width, .. } => Some(PropFieldValue::Int { value: *width }),
            _ => None,
        },
        PropField::SizeX => match p {
            Primitive::PcbPad { dx, .. } => Some(PropFieldValue::Int { value: *dx }),
            _ => None,
        },
        PropField::SizeY => match p {
            Primitive::PcbPad { dy, .. } => Some(PropFieldValue::Int { value: *dy }),
            _ => None,
        },
        PropField::IntDiam => match p {
            Primitive::PcbPad { hole, .. } => Some(PropFieldValue::Int { value: *hole }),
            _ => None,
        },
        PropField::PadStyle => match p {
            Primitive::PcbPad { style, .. } => Some(PropFieldValue::PadStyle {
                value: pad_style_name(*style).into(),
            }),
            _ => None,
        },
        PropField::Text => match p {
            Primitive::Text { text, .. } => Some(PropFieldValue::String {
                value: text.clone(),
            }),
            _ => None,
        },
        PropField::FontFace => match p {
            Primitive::Text { font, .. } => Some(PropFieldValue::String {
                value: font.clone(),
            }),
            _ => None,
        },
        PropField::FontHeight => match p {
            Primitive::Text { sy, .. } => Some(PropFieldValue::Int { value: *sy }),
            _ => None,
        },
        PropField::FontWidth => match p {
            Primitive::Text { sx, .. } => Some(PropFieldValue::Int { value: *sx }),
            _ => None,
        },
        PropField::RotationAngle => match p {
            Primitive::Text { angle, .. } => Some(PropFieldValue::Int { value: *angle }),
            _ => None,
        },
        PropField::Bold => match p {
            Primitive::Text { style, .. } => Some(PropFieldValue::Bool {
                value: style & STYLE_BOLD != 0,
            }),
            _ => None,
        },
        PropField::Italic => match p {
            Primitive::Text { style, .. } => Some(PropFieldValue::Bool {
                value: style & STYLE_ITALIC != 0,
            }),
            _ => None,
        },
        PropField::Mirrored => match p {
            Primitive::Text { style, .. } => Some(PropFieldValue::Bool {
                value: style & STYLE_MIRRORED != 0,
            }),
            _ => None,
        },
        PropField::Underlined => match p {
            Primitive::Text { style, .. } => Some(PropFieldValue::Bool {
                value: style & STYLE_UNDERLINE != 0,
            }),
            _ => None,
        },
    }
}

fn values_equal(a: &PropFieldValue, b: &PropFieldValue) -> bool {
    match (a, b) {
        (PropFieldValue::Bool { value: va }, PropFieldValue::Bool { value: vb }) => va == vb,
        (PropFieldValue::Int { value: va }, PropFieldValue::Int { value: vb }) => va == vb,
        (PropFieldValue::String { value: va }, PropFieldValue::String { value: vb }) => va == vb,
        (PropFieldValue::Layer { value: va }, PropFieldValue::Layer { value: vb }) => va == vb,
        (PropFieldValue::PadStyle { value: va }, PropFieldValue::PadStyle { value: vb }) => {
            va == vb
        }
        _ => false,
    }
}

fn pad_style_name(style: PadStyle) -> &'static str {
    match style {
        PadStyle::Oval => "Round",
        PadStyle::Rectangular => "Square",
        PadStyle::RoundedRect => "SquareRounded",
    }
}

fn parse_pad_style(name: &str) -> Option<PadStyle> {
    match name {
        "Square" => Some(PadStyle::Rectangular),
        "SquareRounded" => Some(PadStyle::RoundedRect),
        "Round" => Some(PadStyle::Oval),
        _ => None,
    }
}

fn set_style_bit(style: &mut u32, mask: u32, on: bool) {
    if on {
        *style |= mask;
    } else {
        *style &= !mask;
    }
}

fn clamp_int(v: i32, min: i32, max: i32) -> i32 {
    v.clamp(min, max)
}

/// Build the properties form for the given selection (FidoCAD intersection rules).
pub fn selection_props_form(primitives: &[&Primitive]) -> Vec<PropFormField> {
    if primitives.is_empty() {
        return Vec::new();
    }

    let first = primitives[0];
    let mut fields = Vec::new();

    for field in attrib_order(first) {
        let Some(first_val) = read_field(first, field) else {
            continue;
        };
        let mut common = first_val;
        let mut all_support = true;
        for p in &primitives[1..] {
            match read_field(p, field) {
                Some(v) => {
                    if !values_equal(&common, &v) {
                        common = PropFieldValue::Unset;
                    }
                }
                None => {
                    all_support = false;
                    break;
                }
            }
        }
        if all_support {
            fields.push(PropFormField {
                id: field,
                kind: field_kind(field),
                value: common,
                read_only: false,
            });
        }
    }

    // Layer always appended (FidoCAD).
    let mut layer_val = read_field(first, PropField::Layer).unwrap_or(PropFieldValue::Unset);
    let all_macro = primitives.iter().all(|p| matches!(p, Primitive::Macro { .. }));
    for p in &primitives[1..] {
        if let Some(v) = read_field(p, PropField::Layer) {
            if !values_equal(&layer_val, &v) {
                layer_val = PropFieldValue::Unset;
            }
        }
    }
    fields.push(PropFormField {
        id: PropField::Layer,
        kind: PropFieldKind::Layer,
        value: layer_val,
        read_only: all_macro,
    });

    fields
}

fn apply_field(p: &mut Primitive, field: PropField, value: &PropFieldValue) -> bool {
    match (field, value) {
        (PropField::Filled, PropFieldValue::Bool { value: v }) => match p {
            Primitive::Rect { filled, .. }
            | Primitive::Poly { filled, .. }
            | Primitive::Ellipse { filled, .. } => {
                *filled = *v;
                true
            }
            _ => false,
        },
        (PropField::Layer, PropFieldValue::Layer { value: v }) => {
            if matches!(p, Primitive::Macro { .. }) {
                return false;
            }
            if (0..16).contains(v) {
                p.set_layer(LayerId(*v as u8));
                true
            } else {
                false
            }
        }
        (PropField::Thickness, PropFieldValue::Int { value: v }) => match p {
            Primitive::PcbTrack { width, .. } => {
                *width = clamp_int(*v, 1, 100);
                true
            }
            _ => false,
        },
        (PropField::SizeX, PropFieldValue::Int { value: v }) => match p {
            Primitive::PcbPad { dx, .. } => {
                *dx = clamp_int(*v, 2, 100);
                true
            }
            _ => false,
        },
        (PropField::SizeY, PropFieldValue::Int { value: v }) => match p {
            Primitive::PcbPad { dy, .. } => {
                *dy = clamp_int(*v, 2, 100);
                true
            }
            _ => false,
        },
        (PropField::IntDiam, PropFieldValue::Int { value: v }) => match p {
            Primitive::PcbPad { hole, .. } => {
                *hole = clamp_int(*v, 2, 100);
                true
            }
            _ => false,
        },
        (PropField::PadStyle, PropFieldValue::PadStyle { value: v }) => match p {
            Primitive::PcbPad { style, .. } => {
                if let Some(s) = parse_pad_style(v) {
                    *style = s;
                    true
                } else {
                    false
                }
            }
            _ => false,
        },
        (PropField::Text, PropFieldValue::String { value: v }) => match p {
            Primitive::Text { text, .. } => {
                *text = v.clone();
                true
            }
            _ => false,
        },
        (PropField::FontFace, PropFieldValue::String { value: v }) => match p {
            Primitive::Text { font, .. } => {
                *font = v.clone();
                true
            }
            _ => false,
        },
        (PropField::FontHeight, PropFieldValue::Int { value: v }) => match p {
            Primitive::Text { sy, .. } => {
                *sy = clamp_int(*v, 2, 100);
                true
            }
            _ => false,
        },
        (PropField::FontWidth, PropFieldValue::Int { value: v }) => match p {
            Primitive::Text { sx, .. } => {
                *sx = clamp_int(*v, 2, 100);
                true
            }
            _ => false,
        },
        (PropField::RotationAngle, PropFieldValue::Int { value: v }) => match p {
            Primitive::Text { angle, .. } => {
                *angle = clamp_int(*v, 0, 359);
                true
            }
            _ => false,
        },
        (PropField::Bold, PropFieldValue::Bool { value: v }) => match p {
            Primitive::Text { style, .. } => {
                set_style_bit(style, STYLE_BOLD, *v);
                true
            }
            _ => false,
        },
        (PropField::Italic, PropFieldValue::Bool { value: v }) => match p {
            Primitive::Text { style, .. } => {
                set_style_bit(style, STYLE_ITALIC, *v);
                true
            }
            _ => false,
        },
        (PropField::Mirrored, PropFieldValue::Bool { value: v }) => match p {
            Primitive::Text { style, .. } => {
                set_style_bit(style, STYLE_MIRRORED, *v);
                true
            }
            _ => false,
        },
        (PropField::Underlined, PropFieldValue::Bool { value: v }) => match p {
            Primitive::Text { style, .. } => {
                set_style_bit(style, STYLE_UNDERLINE, *v);
                true
            }
            _ => false,
        },
        _ => false,
    }
}

fn patch_to_fields(patch: &PropPatch) -> HashMap<PropField, PropFieldValue> {
    let mut m = HashMap::new();
    if let Some(v) = patch.filled {
        m.insert(PropField::Filled, PropFieldValue::Bool { value: v });
    }
    if let Some(v) = patch.layer {
        m.insert(PropField::Layer, PropFieldValue::Layer { value: v });
    }
    if let Some(v) = patch.thickness {
        m.insert(PropField::Thickness, PropFieldValue::Int { value: v });
    }
    if let Some(v) = patch.size_x {
        m.insert(PropField::SizeX, PropFieldValue::Int { value: v });
    }
    if let Some(v) = patch.size_y {
        m.insert(PropField::SizeY, PropFieldValue::Int { value: v });
    }
    if let Some(v) = patch.int_diam {
        m.insert(PropField::IntDiam, PropFieldValue::Int { value: v });
    }
    if let Some(v) = &patch.pad_style {
        m.insert(
            PropField::PadStyle,
            PropFieldValue::PadStyle { value: v.clone() },
        );
    }
    if let Some(v) = &patch.text {
        m.insert(
            PropField::Text,
            PropFieldValue::String { value: v.clone() },
        );
    }
    if let Some(v) = &patch.font_face {
        m.insert(
            PropField::FontFace,
            PropFieldValue::String { value: v.clone() },
        );
    }
    if let Some(v) = patch.font_height {
        m.insert(PropField::FontHeight, PropFieldValue::Int { value: v });
    }
    if let Some(v) = patch.font_width {
        m.insert(PropField::FontWidth, PropFieldValue::Int { value: v });
    }
    if let Some(v) = patch.rotation_angle {
        m.insert(PropField::RotationAngle, PropFieldValue::Int { value: v });
    }
    if let Some(v) = patch.bold {
        m.insert(PropField::Bold, PropFieldValue::Bool { value: v });
    }
    if let Some(v) = patch.italic {
        m.insert(PropField::Italic, PropFieldValue::Bool { value: v });
    }
    if let Some(v) = patch.mirrored {
        m.insert(PropField::Mirrored, PropFieldValue::Bool { value: v });
    }
    if let Some(v) = patch.underlined {
        m.insert(PropField::Underlined, PropFieldValue::Bool { value: v });
    }
    m
}

/// Apply a partial property patch to the given primitives (only set fields).
pub fn apply_selection_props(primitives: &mut [Primitive], patch: &PropPatch) -> bool {
    let fields = patch_to_fields(patch);
    if fields.is_empty() {
        return false;
    }
    let mut changed = false;
    for p in primitives.iter_mut() {
        for (field, value) in &fields {
            if apply_field(p, *field, value) {
                changed = true;
            }
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Point;
    use crate::layers::LayerId;

    fn rect(filled: bool, layer: u8) -> Primitive {
        Primitive::Rect {
            a: Point::new(0, 0),
            b: Point::new(10, 10),
            filled,
            layer: LayerId(layer),
        }
    }

    fn line(layer: u8) -> Primitive {
        Primitive::Line {
            a: Point::new(0, 0),
            b: Point::new(5, 5),
            layer: LayerId(layer),
        }
    }

    #[test]
    fn homogeneous_rects_show_filled_and_layer() {
        let r1 = rect(true, 1);
        let r2 = rect(true, 1);
        let r3 = rect(true, 1);
        let form = selection_props_form(&[&r1, &r2, &r3]);
        assert_eq!(form.len(), 2);
        assert_eq!(form[0].id, PropField::Filled);
        assert_eq!(
            form[0].value,
            PropFieldValue::Bool { value: true }
        );
        assert_eq!(form[1].id, PropField::Layer);
    }

    #[test]
    fn mixed_rect_line_only_layer() {
        let r = rect(false, 0);
        let l = line(0);
        let form = selection_props_form(&[&r, &l]);
        assert_eq!(form.len(), 1);
        assert_eq!(form[0].id, PropField::Layer);
    }

    #[test]
    fn mixed_filled_values_unset() {
        let r1 = rect(true, 0);
        let r2 = rect(false, 0);
        let form = selection_props_form(&[&r1, &r2]);
        assert_eq!(form[0].value, PropFieldValue::Unset);
    }

    #[test]
    fn apply_partial_layer_only() {
        let mut prims = [rect(true, 0), rect(false, 0)];
        let patch = PropPatch {
            layer: Some(3),
            ..Default::default()
        };
        assert!(apply_selection_props(&mut prims, &patch));
        assert_eq!(prims[0].layer(), LayerId(3));
        assert_eq!(prims[1].layer(), LayerId(3));
        assert!(matches!(
            prims[0],
            Primitive::Rect {
                filled: true,
                ..
            }
        ));
        assert!(matches!(
            prims[1],
            Primitive::Rect {
                filled: false,
                ..
            }
        ));
    }
}
