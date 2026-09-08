//! Canvas colours used by tessellation and the WebGL renderer.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb(pub [f32; 3]);

impl Rgb {
    pub const SELECTION: Self = Self([0.85, 0.42, 0.22]);
    pub const PREVIEW_LIGHT: Self = Self([0.72, 0.42, 0.22]);
    pub const PREVIEW_DARK: Self = Self([0.85, 0.55, 0.32]);
    pub fn from_u8(c: [u8; 3]) -> Self {
        Self([
            c[0] as f32 / 255.0,
            c[1] as f32 / 255.0,
            c[2] as f32 / 255.0,
        ])
    }

    pub fn to_svg(self) -> String {
        format!(
            "rgb({},{},{})",
            (self.0[0] * 255.0) as u8,
            (self.0[1] * 255.0) as u8,
            (self.0[2] * 255.0) as u8
        )
    }

    pub fn preview(dark: bool) -> Self {
        if dark {
            Self::PREVIEW_DARK
        } else {
            Self::PREVIEW_LIGHT
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub bg: [f32; 3],
    pub grid: [f32; 3],
}

impl Theme {
    pub const LIGHT: Self = Self {
        bg: [0.97, 0.95, 0.92],
        grid: [0.78, 0.72, 0.66],
    };
    pub const DARK: Self = Self {
        bg: [0.10, 0.09, 0.08],
        grid: [0.32, 0.26, 0.22],
    };
}
