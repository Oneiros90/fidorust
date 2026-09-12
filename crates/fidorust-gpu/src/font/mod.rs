//! Bundled Courier Prime (SIL OFL) plus optional system faces for schematic labels.
//!
//! [`install_hit_hooks`] injects glyph coverage into `fidorust-core` at runtime so
//! hit-testing can use tessellated outlines without a compile-time GPU dependency.

mod glyphs;
mod registry;
#[cfg(not(target_arch = "wasm32"))]
mod system;

#[cfg(not(target_arch = "wasm32"))]
pub use system::load_system_monospace;

pub use glyphs::{glyph_covers, glyph_triangles, install_hit_hooks};
pub use registry::{
    bundled_font_data, face_is_mono, font_file_bytes, register_font, registered_families,
};

#[cfg(test)]
pub(crate) use registry::best_mono_face;

#[cfg(test)]
pub fn reset_registry() {
    registry::clear_fonts();
    glyphs::clear_glyphs();
}

#[cfg(test)]
mod tests {
    use super::registry::FONT_DATA;
    use super::*;
    use fidorust_core::primitive::DEFAULT_FONT;
    use ttf_parser::Face;

    #[test]
    fn letter_a_is_filled() {
        let tris = glyph_triangles(DEFAULT_FONT, 'A');
        assert!(
            tris.len() >= 9,
            "expected tessellated triangles, got {}",
            tris.len()
        );
        assert_eq!(tris.len() % 3, 0);
    }

    #[test]
    fn letter_o_has_ink_and_a_hole() {
        let mut ink = false;
        let mut hole = false;
        for i in 0..21 {
            for j in 0..21 {
                let u = i as f32 / 20.0;
                let v = j as f32 / 20.0;
                if glyph_covers(DEFAULT_FONT, 'O', u, v) {
                    ink = true;
                } else if (0.3..0.7).contains(&u) && (0.3..0.7).contains(&v) {
                    hole = true;
                }
            }
        }
        assert!(ink, "expected tessellated ink for O");
        assert!(hole, "expected the counter of O to be empty");
    }

    #[test]
    fn space_has_no_geometry() {
        assert!(glyph_triangles(DEFAULT_FONT, ' ').is_empty());
    }

    #[test]
    fn empty_font_uses_courier_prime() {
        let a = glyph_triangles("", 'A');
        let b = glyph_triangles(DEFAULT_FONT, 'A');
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn unknown_font_falls_back_to_courier_prime() {
        let a = glyph_triangles("Missing Mono Face", 'Q');
        let b = glyph_triangles(DEFAULT_FONT, 'Q');
        assert_eq!(a, b);
    }

    #[test]
    fn bundled_prime_is_detected_mono() {
        let face = Face::parse(FONT_DATA, 0).unwrap();
        assert!(face_is_mono(&face));
        assert!(best_mono_face(FONT_DATA).is_some());
    }

    #[test]
    fn register_font_rejects_garbage() {
        assert!(!register_font("Nope", b"not a font"));
    }

    #[test]
    fn register_font_rejects_empty() {
        assert!(!register_font("Nope", b""));
        assert!(!register_font("", FONT_DATA));
    }

    #[test]
    fn register_font_accepts_courier_prime_under_alias() {
        reset_registry();
        assert!(register_font("Test Mono", FONT_DATA));
        let families = registered_families();
        assert_eq!(families[0], DEFAULT_FONT);
        assert!(families.iter().any(|n| n == "Test Mono"));
        let a = glyph_triangles("Test Mono", 'A');
        let b = glyph_triangles(DEFAULT_FONT, 'A');
        assert_eq!(a, b);
        reset_registry();
    }

    #[test]
    fn font_file_bytes_returns_bundled_prime_or_registered() {
        reset_registry();
        assert_eq!(font_file_bytes(""), bundled_font_data());
        assert_eq!(font_file_bytes("Courier Prime"), bundled_font_data());
        assert!(font_file_bytes("Missing Mono").is_empty());
        assert!(register_font("Test Mono", FONT_DATA));
        assert_eq!(font_file_bytes("Test Mono"), bundled_font_data());
        reset_registry();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn system_monospace_scan_finds_usable_faces() {
        let fonts = load_system_monospace();
        assert!(
            !fonts.is_empty(),
            "expected at least one OS monospace font (Consolas, DejaVu Sans Mono, …)"
        );
        for (name, data) in &fonts {
            assert!(!name.is_empty(), "empty family name");
            assert!(
                best_mono_face(data).is_some(),
                "{name} should parse as monospace"
            );
            assert!(
                !name.eq_ignore_ascii_case(DEFAULT_FONT),
                "bundled default should not be re-listed from disk"
            );
        }
        reset_registry();
        let mut ok = 0;
        for (name, data) in &fonts {
            if register_font(name, data) {
                ok += 1;
            }
        }
        assert_eq!(ok, fonts.len());
        let listed = registered_families();
        assert!(
            listed.len() > 1,
            "dropdown should include system families: {listed:?}"
        );
        let names: Vec<String> = fonts.iter().map(|(n, _)| n.to_ascii_lowercase()).collect();
        assert!(
            names.iter().any(|n| n.contains("mono")
                || n.contains("consolas")
                || n.contains("cascadia")
                || n.contains("dejavu")
                || n.contains("liberation")
                || n.contains("lucida")
                || n.contains("menlo")
                || n.contains("monaco")
                || n.contains("courier")),
            "scan missed well-known OS monospace families: {names:?}"
        );
        reset_registry();
    }

    #[cfg(windows)]
    #[test]
    fn windows_consolas_registers_and_differs_from_prime() {
        let fonts = load_system_monospace();
        let found = fonts
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("Consolas"));
        let Some((name, data)) = found else {
            panic!(
                "Consolas not recovered from Windows Fonts; got {:?}",
                fonts.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
            );
        };
        assert!(best_mono_face(data).is_some());
        reset_registry();
        assert!(register_font(name, data));
        let sys = glyph_triangles(name, 'A');
        let bundled = glyph_triangles(DEFAULT_FONT, 'A');
        assert!(!sys.is_empty());
        assert_ne!(
            sys, bundled,
            "registered Consolas must tessellate its own outlines"
        );
        reset_registry();
    }

    #[cfg(windows)]
    #[test]
    fn windows_arial_is_rejected() {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        let path = std::path::PathBuf::from(windir)
            .join("Fonts")
            .join("arial.ttf");
        if !path.exists() {
            return;
        }
        let data = std::fs::read(&path).expect("read Arial");
        assert!(
            best_mono_face(&data).is_none(),
            "Arial must not be treated as monospace"
        );
        reset_registry();
        assert!(!register_font("Arial", &data));
        reset_registry();
    }
}
