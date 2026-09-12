//! Canvas colours used by tessellation and the WebGL renderer.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb(pub [f32; 3]);

impl Rgb {
    pub const SELECTION: Self = Self([0.85, 0.42, 0.22]);
    pub const PREVIEW_LIGHT: Self = Self([0.72, 0.42, 0.22]);
    pub const PREVIEW_DARK: Self = Self([0.85, 0.55, 0.32]);
    /// How much white to mix into a hovered primitive's layer colour (`0` = none, `1` = white).
    pub const HOVER_LIGHTEN: f32 = 0.15;

    pub fn from_rgba_u8(c: [u8; 4]) -> [f32; 4] {
        [
            c[0] as f32 / 255.0,
            c[1] as f32 / 255.0,
            c[2] as f32 / 255.0,
            c[3] as f32 / 255.0,
        ]
    }

    /// Mix `amount` of white into RGB (alpha unchanged).
    pub fn mix_white(rgba: [f32; 4], amount: f32) -> [f32; 4] {
        let t = amount.clamp(0.0, 1.0);
        [
            rgba[0] + (1.0 - rgba[0]) * t,
            rgba[1] + (1.0 - rgba[1]) * t,
            rgba[2] + (1.0 - rgba[2]) * t,
            rgba[3],
        ]
    }

    pub fn rgba(self, a: f32) -> [f32; 4] {
        [self.0[0], self.0[1], self.0[2], a]
    }

    pub fn to_svg(self) -> String {
        css_color(self.0[0], self.0[1], self.0[2], 1.0)
    }

    pub fn preview(dark: bool) -> Self {
        if dark {
            Self::PREVIEW_DARK
        } else {
            Self::PREVIEW_LIGHT
        }
    }
}

pub fn css_color(r: f32, g: f32, b: f32, a: f32) -> String {
    let ru = (r * 255.0).round().clamp(0.0, 255.0) as u8;
    let gu = (g * 255.0).round().clamp(0.0, 255.0) as u8;
    let bu = (b * 255.0).round().clamp(0.0, 255.0) as u8;
    if a >= 0.999 {
        format!("rgb({ru},{gu},{bu})")
    } else {
        let a = (a * 1000.0).round() / 1000.0;
        format!("rgba({ru},{gu},{bu},{a})")
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
