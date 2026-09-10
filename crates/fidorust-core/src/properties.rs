//! Object properties dialog: common attributes across a multi-selection (FidoCAD 0.96).

use crate::layers::LayerId;
use crate::primitive::{
    Bezier, ComponentRef, Connection, Ellipse, Line, PadStyle, PcbPad, PcbTrack, Poly, Primitive,
    Rect, Text, STYLE_BOLD, STYLE_ITALIC, STYLE_MIRRORED, STYLE_UNDERLINE,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

/// Editable property identifiers (FidoCAD internal names).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropField {
    UseComponentLayers,
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
    Choice { options: Vec<String> },
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
    /// When true the field is shown but cannot be applied.
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropPatch {
    #[serde(default)]
    pub use_component_layers: Option<bool>,
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

/// Per-variant property table used by the properties dialog.
pub trait PropSource {
    fn fields() -> &'static [PropField];
    fn read(&self, field: PropField) -> Option<PropFieldValue>;
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool;
}

fn read_layer(layer: LayerId, field: PropField) -> Option<PropFieldValue> {
    match field {
        PropField::Layer => Some(PropFieldValue::Layer {
            value: layer.0 as i32,
        }),
        _ => None,
    }
}

fn apply_layer(layer: &mut LayerId, field: PropField, value: &PropFieldValue) -> bool {
    match (field, value) {
        (PropField::Layer, PropFieldValue::Layer { value: v }) if (0..256).contains(v) => {
            *layer = LayerId(*v as u8);
            true
        }
        _ => false,
    }
}

fn read_filled(filled: bool, layer: LayerId, field: PropField) -> Option<PropFieldValue> {
    match field {
        PropField::Filled => Some(PropFieldValue::Bool { value: filled }),
        _ => read_layer(layer, field),
    }
}

fn apply_filled(
    filled: &mut bool,
    layer: &mut LayerId,
    field: PropField,
    value: &PropFieldValue,
) -> bool {
    match (field, value) {
        (PropField::Filled, PropFieldValue::Bool { value: v }) => {
            *filled = *v;
            true
        }
        _ => apply_layer(layer, field, value),
    }
}

impl PropSource for Line {
    fn fields() -> &'static [PropField] {
        &[]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        read_layer(self.layer, field)
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        apply_layer(&mut self.layer, field, value)
    }
}

impl PropSource for Bezier {
    fn fields() -> &'static [PropField] {
        &[]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        read_layer(self.layer, field)
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        apply_layer(&mut self.layer, field, value)
    }
}

impl PropSource for Connection {
    fn fields() -> &'static [PropField] {
        &[]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        read_layer(self.layer, field)
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        apply_layer(&mut self.layer, field, value)
    }
}

impl PropSource for Rect {
    fn fields() -> &'static [PropField] {
        &[PropField::Filled]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        read_filled(self.filled, self.layer, field)
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        apply_filled(&mut self.filled, &mut self.layer, field, value)
    }
}

impl PropSource for Poly {
    fn fields() -> &'static [PropField] {
        &[PropField::Filled]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        read_filled(self.filled, self.layer, field)
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        apply_filled(&mut self.filled, &mut self.layer, field, value)
    }
}

impl PropSource for Ellipse {
    fn fields() -> &'static [PropField] {
        &[PropField::Filled]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        read_filled(self.filled, self.layer, field)
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        apply_filled(&mut self.filled, &mut self.layer, field, value)
    }
}

impl PropSource for PcbTrack {
    fn fields() -> &'static [PropField] {
        &[PropField::Thickness]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        match field {
            PropField::Thickness => Some(PropFieldValue::Int { value: self.width }),
            _ => read_layer(self.layer, field),
        }
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        match (field, value) {
            (PropField::Thickness, PropFieldValue::Int { value: v }) => {
                self.width = (*v).clamp(1, 100);
                true
            }
            _ => apply_layer(&mut self.layer, field, value),
        }
    }
}

impl PropSource for PcbPad {
    fn fields() -> &'static [PropField] {
        &[
            PropField::SizeX,
            PropField::SizeY,
            PropField::IntDiam,
            PropField::PadStyle,
        ]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        match field {
            PropField::SizeX => Some(PropFieldValue::Int { value: self.dx }),
            PropField::SizeY => Some(PropFieldValue::Int { value: self.dy }),
            PropField::IntDiam => Some(PropFieldValue::Int { value: self.hole }),
            PropField::PadStyle => Some(PropFieldValue::PadStyle {
                value: self.style.to_string(),
            }),
            _ => read_layer(self.layer, field),
        }
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        match (field, value) {
            (PropField::SizeX, PropFieldValue::Int { value: v }) => {
                self.dx = (*v).clamp(2, 100);
                true
            }
            (PropField::SizeY, PropFieldValue::Int { value: v }) => {
                self.dy = (*v).clamp(2, 100);
                true
            }
            (PropField::IntDiam, PropFieldValue::Int { value: v }) => {
                self.hole = (*v).clamp(2, 100);
                true
            }
            (PropField::PadStyle, PropFieldValue::PadStyle { value: v }) => {
                if let Ok(s) = PadStyle::from_str(v) {
                    self.style = s;
                    true
                } else {
                    false
                }
            }
            _ => apply_layer(&mut self.layer, field, value),
        }
    }
}

impl PropSource for Text {
    fn fields() -> &'static [PropField] {
        &[
            PropField::Text,
            PropField::FontFace,
            PropField::FontHeight,
            PropField::FontWidth,
            PropField::RotationAngle,
            PropField::Bold,
            PropField::Italic,
            PropField::Mirrored,
            PropField::Underlined,
        ]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        match field {
            PropField::Text => Some(PropFieldValue::String {
                value: self.text.clone(),
            }),
            PropField::FontFace => Some(PropFieldValue::String {
                value: self.font.clone(),
            }),
            PropField::FontHeight => Some(PropFieldValue::Int { value: self.sy }),
            PropField::FontWidth => Some(PropFieldValue::Int { value: self.sx }),
            PropField::RotationAngle => Some(PropFieldValue::Int { value: self.angle }),
            PropField::Bold => Some(PropFieldValue::Bool {
                value: self.style & STYLE_BOLD != 0,
            }),
            PropField::Italic => Some(PropFieldValue::Bool {
                value: self.style & STYLE_ITALIC != 0,
            }),
            PropField::Mirrored => Some(PropFieldValue::Bool {
                value: self.style & STYLE_MIRRORED != 0,
            }),
            PropField::Underlined => Some(PropFieldValue::Bool {
                value: self.style & STYLE_UNDERLINE != 0,
            }),
            _ => read_layer(self.layer, field),
        }
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        match (field, value) {
            (PropField::Text, PropFieldValue::String { value: v }) => {
                self.text = v.clone();
                true
            }
            (PropField::FontFace, PropFieldValue::String { value: v }) => {
                self.font = v.clone();
                true
            }
            (PropField::FontHeight, PropFieldValue::Int { value: v }) => {
                self.sy = (*v).clamp(2, 100);
                true
            }
            (PropField::FontWidth, PropFieldValue::Int { value: v }) => {
                self.sx = (*v).clamp(2, 100);
                true
            }
            (PropField::RotationAngle, PropFieldValue::Int { value: v }) => {
                self.angle = (*v).clamp(0, 359);
                true
            }
            (PropField::Bold, PropFieldValue::Bool { value: v }) => {
                set_style_bit(&mut self.style, STYLE_BOLD, *v);
                true
            }
            (PropField::Italic, PropFieldValue::Bool { value: v }) => {
                set_style_bit(&mut self.style, STYLE_ITALIC, *v);
                true
            }
            (PropField::Mirrored, PropFieldValue::Bool { value: v }) => {
                set_style_bit(&mut self.style, STYLE_MIRRORED, *v);
                true
            }
            (PropField::Underlined, PropFieldValue::Bool { value: v }) => {
                set_style_bit(&mut self.style, STYLE_UNDERLINE, *v);
                true
            }
            _ => apply_layer(&mut self.layer, field, value),
        }
    }
}

impl PropSource for ComponentRef {
    fn fields() -> &'static [PropField] {
        &[PropField::UseComponentLayers]
    }
    fn read(&self, field: PropField) -> Option<PropFieldValue> {
        match field {
            PropField::UseComponentLayers => Some(PropFieldValue::Bool {
                value: self.use_component_layers,
            }),
            _ => read_layer(self.layer, field),
        }
    }
    fn apply(&mut self, field: PropField, value: &PropFieldValue) -> bool {
        match (field, value) {
            (PropField::UseComponentLayers, PropFieldValue::Bool { value: v }) => {
                if self.use_component_layers == *v {
                    return false;
                }
                self.use_component_layers = *v;
                true
            }
            (PropField::Layer, _) if self.use_component_layers => false,
            _ => apply_layer(&mut self.layer, field, value),
        }
    }
}

fn attrib_order(first: &Primitive) -> &'static [PropField] {
    match first {
        Primitive::Line(_) => Line::fields(),
        Primitive::Rect(_) => Rect::fields(),
        Primitive::Poly(_) => Poly::fields(),
        Primitive::Ellipse(_) => Ellipse::fields(),
        Primitive::Bezier(_) => Bezier::fields(),
        Primitive::Text(_) => Text::fields(),
        Primitive::Connection(_) => Connection::fields(),
        Primitive::PcbTrack(_) => PcbTrack::fields(),
        Primitive::PcbPad(_) => PcbPad::fields(),
        Primitive::Component(_) => ComponentRef::fields(),
    }
}

fn field_kind(field: PropField) -> PropFieldKind {
    match field {
        PropField::UseComponentLayers
        | PropField::Filled
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
    crate::dispatch_primitive!(p, |inner| inner.read(field))
}

fn set_style_bit(style: &mut u32, mask: u32, on: bool) {
    if on {
        *style |= mask;
    } else {
        *style &= !mask;
    }
}

/// Build the properties form for the given selection (FidoCAD intersection rules).
pub fn selection_props_form(primitives: &[&Primitive]) -> Vec<PropFormField> {
    if primitives.is_empty() {
        return Vec::new();
    }

    let first = primitives[0];
    let mut fields = Vec::new();

    for &field in attrib_order(first) {
        let Some(first_val) = read_field(first, field) else {
            continue;
        };
        let mut common = first_val;
        let mut all_support = true;
        for p in &primitives[1..] {
            match read_field(p, field) {
                Some(v) => {
                    if common != v {
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
    for p in &primitives[1..] {
        if let Some(v) = read_field(p, PropField::Layer) {
            if layer_val != v {
                layer_val = PropFieldValue::Unset;
            }
        }
    }
    let layer_read_only = primitives
        .iter()
        .all(|p| matches!(p, Primitive::Component(c) if c.use_component_layers));
    fields.push(PropFormField {
        id: PropField::Layer,
        kind: PropFieldKind::Layer,
        value: layer_val,
        read_only: layer_read_only,
    });

    fields
}

fn apply_field(p: &mut Primitive, field: PropField, value: &PropFieldValue) -> bool {
    crate::dispatch_primitive!(p, |inner| inner.apply(field, value))
}

fn patch_to_fields(patch: &PropPatch) -> BTreeMap<PropField, PropFieldValue> {
    let mut m = BTreeMap::new();
    if let Some(v) = patch.use_component_layers {
        m.insert(
            PropField::UseComponentLayers,
            PropFieldValue::Bool { value: v },
        );
    }
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
        m.insert(PropField::Text, PropFieldValue::String { value: v.clone() });
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
    let mut fields = patch_to_fields(patch);
    if fields.is_empty() {
        return false;
    }
    // Apply the flag first so a following layer assign sees the new mode.
    let flag = fields.remove(&PropField::UseComponentLayers);
    let mut changed = false;
    for p in primitives.iter_mut() {
        if let Some(value) = &flag {
            if apply_field(p, PropField::UseComponentLayers, value) {
                changed = true;
            }
        }
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
        Primitive::Rect(Rect {
            a: Point::new(0, 0),
            b: Point::new(10, 10),
            filled,
            layer: LayerId(layer),
        })
    }

    fn line(layer: u8) -> Primitive {
        Primitive::Line(Line {
            a: Point::new(0, 0),
            b: Point::new(5, 5),
            layer: LayerId(layer),
        })
    }

    #[test]
    fn homogeneous_rects_show_filled_and_layer() {
        let r1 = rect(true, 1);
        let r2 = rect(true, 1);
        let r3 = rect(true, 1);
        let form = selection_props_form(&[&r1, &r2, &r3]);
        assert_eq!(form.len(), 2);
        assert_eq!(form[0].id, PropField::Filled);
        assert_eq!(form[0].value, PropFieldValue::Bool { value: true });
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
            Primitive::Rect(Rect { filled: true, .. })
        ));
        assert!(matches!(
            prims[1],
            Primitive::Rect(Rect { filled: false, .. })
        ));
    }
}
