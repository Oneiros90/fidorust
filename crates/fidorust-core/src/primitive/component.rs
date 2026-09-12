//! Unexpanded component instance (FidoCAD `MC` line).

use serde::{Deserialize, Serialize};

use crate::consts::COMPONENT_HIT_R2;
use crate::geom::{Aabb, Point};
use crate::layers::LayerId;

use super::traits::{set_pos, Geometry, HitTest};
use crate::properties::{apply_layer, read_layer, PropField, PropFieldValue, PropSource};

/// Unexpanded component instance. Body is expanded at draw/hit time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentRef {
    pub pos: Point,
    pub rotations: u8,
    pub mirrored: bool,
    pub name: String,
    pub standard: bool,
    pub layer: LayerId,
    /// When true, expanded primitives keep the layers stored in the definition
    /// (classic FidoCAD). When false, the instance is painted onto [`Self::layer`].
    #[serde(default)]
    pub use_component_layers: bool,
}

impl Geometry for ComponentRef {
    fn layer(&self) -> LayerId {
        self.layer
    }
    fn set_layer(&mut self, layer: LayerId) {
        self.layer = layer;
    }
    fn aabb(&self) -> Aabb {
        Aabb {
            min: Point::new(self.pos.x - 10, self.pos.y - 10),
            max: Point::new(self.pos.x + 10, self.pos.y + 10),
        }
    }
    fn control_points(&self) -> Vec<Point> {
        vec![self.pos]
    }
    fn set_control_point(&mut self, index: usize, p: Point) {
        set_pos(&mut self.pos, index, p);
    }
    fn transform_points(&mut self, f: impl Fn(Point) -> Point) {
        self.pos = f(self.pos);
    }
}

impl HitTest for ComponentRef {
    fn body_hit(&self, x: f64, y: f64, tol2: f64) -> bool {
        self.pos.dist_sq_xy(x, y) <= COMPONENT_HIT_R2.max(tol2)
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
