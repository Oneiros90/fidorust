//! Property dialog tests.

use fidocad_core::geom::Point;
use fidocad_core::layers::LayerId;
use fidocad_core::primitive::Primitive;
use fidocad_core::properties::{apply_selection_props, selection_props_form, PropField, PropPatch};

#[test]
fn integration_mixed_selection_layer_only() {
    let r = Primitive::Rect {
        a: Point::new(0, 0),
        b: Point::new(10, 10),
        filled: true,
        layer: LayerId(1),
    };
    let l = Primitive::Line {
        a: Point::new(0, 0),
        b: Point::new(5, 5),
        layer: LayerId(1),
    };
    let form = selection_props_form(&[&r, &l]);
    assert_eq!(form.len(), 1);
    assert_eq!(form[0].id, PropField::Layer);
}

#[test]
fn integration_apply_filled() {
    let mut r = Primitive::Rect {
        a: Point::new(0, 0),
        b: Point::new(10, 10),
        filled: false,
        layer: LayerId(0),
    };
    let patch = PropPatch {
        filled: Some(true),
        ..Default::default()
    };
    apply_selection_props(std::slice::from_mut(&mut r), &patch);
    assert!(matches!(
        r,
        Primitive::Rect {
            filled: true,
            ..
        }
    ));
}
