//! Ephemeral canvas presentation. Not saved, not undoable, not part of the `.fcd`.

use crate::geom::Point;

/// Extra filled circle in world LU, drawn after document layers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayMarker {
    pub pos: Point,
    pub radius: f32,
    pub color: [u8; 4],
}

/// Canvas chrome (background / grid / selection) independent of the UI theme picker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPalette {
    pub bg: [f32; 3],
    pub grid: [f32; 3],
    pub selection: [f32; 3],
}

/// Non-document colour and marker override applied at tessellation time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewOverlay {
    /// Replace layer ink for primitives that are not accented or selected.
    pub ink: Option<[u8; 4]>,
    /// Top-level primitive indices painted with [`Self::accent`].
    pub accent_ids: Vec<usize>,
    pub accent: [u8; 4],
    pub markers: Vec<OverlayMarker>,
    pub canvas: Option<CanvasPalette>,
}

impl ViewOverlay {
    pub fn is_accent(&self, index: usize) -> bool {
        self.accent_ids.contains(&index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::LibrarySet;
    use crate::Editor;

    #[test]
    fn overlay_is_not_in_undo() {
        let mut ed = Editor::new(LibrarySet::new());
        ed.set_grid(10, 10);
        ed.set_view_overlay(Some(ViewOverlay {
            ink: Some([0, 0, 0, 255]),
            ..Default::default()
        }));
        assert!(ed.view_overlay().is_some());
        ed.undo();
        assert!(ed.view_overlay().is_some());
        assert_eq!(ed.view_overlay().unwrap().ink, Some([0, 0, 0, 255]));
    }
}
