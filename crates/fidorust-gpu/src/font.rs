//! Bundled Courier Prime (SIL OFL) plus optional system faces for schematic labels.

use std::cell::RefCell;
use std::collections::HashMap;

use fidorust_core::primitive::DEFAULT_FONT;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use ttf_parser::{Face, OutlineBuilder, Style};

const FONT_DATA: &[u8] = include_bytes!("../fonts/CourierPrime-Regular.ttf");

struct RegisteredFont {
    display: String,
    data: Vec<u8>,
    face_index: u32,
    score: i32,
}

type GlyphCache = HashMap<(String, char), Vec<[f32; 2]>>;

thread_local! {
    static FONTS: RefCell<HashMap<String, RegisteredFont>> = RefCell::new(HashMap::new());
    static GLYPHS: RefCell<GlyphCache> = RefCell::new(HashMap::new());
}

fn cache_key(font: &str) -> String {
    let t = font.trim();
    if t.is_empty() {
        DEFAULT_FONT.to_ascii_lowercase()
    } else {
        t.to_ascii_lowercase()
    }
}

fn regularity_score(face: &Face<'_>) -> i32 {
    let mut score = 0;
    if !face.is_italic() {
        score += 100;
    }
    if face.style() == Style::Normal {
        score += 50;
    }
    let w = face.weight().to_number() as i32;
    score -= (w - 400).abs();
    score
}

fn advance(face: &Face<'_>, ch: char) -> Option<u16> {
    face.glyph_index(ch).and_then(|g| face.glyph_hor_advance(g))
}

/// True if the face is tagged monospace or typical letters share the same advance.
pub fn face_is_mono(face: &Face<'_>) -> bool {
    if face.is_monospaced() {
        return true;
    }
    let samples = ['i', 'M', 'W', '.', '0'];
    let mut widths = samples.iter().filter_map(|&ch| advance(face, ch));
    let Some(first) = widths.next() else {
        return false;
    };
    first > 0 && widths.all(|w| w == first)
}

fn collection_len(data: &[u8]) -> u32 {
    ttf_parser::fonts_in_collection(data).unwrap_or(1)
}

fn best_mono_face(data: &[u8]) -> Option<(u32, i32)> {
    let n = collection_len(data);
    let mut best: Option<(u32, i32)> = None;
    for index in 0..n {
        let Ok(face) = Face::parse(data, index) else {
            continue;
        };
        if !face_is_mono(&face) {
            continue;
        }
        let score = regularity_score(&face);
        match best {
            Some((_, s)) if s >= score => {}
            _ => best = Some((index, score)),
        }
    }
    best
}

#[cfg(not(target_arch = "wasm32"))]
fn family_name(face: &Face<'_>) -> Option<String> {
    let mut fallback = None;
    for name in face.names() {
        if !name.is_unicode() {
            continue;
        }
        let Some(s) = name.to_string() else {
            continue;
        };
        if s.is_empty() {
            continue;
        }
        if name.name_id == ttf_parser::name_id::TYPOGRAPHIC_FAMILY {
            return Some(s);
        }
        if name.name_id == ttf_parser::name_id::FAMILY {
            fallback = Some(s);
        }
    }
    fallback
}

/// Register a TTF/OTF/TTC face. Keeps monospace families; Regular is preferred.
pub fn register_font(name: &str, data: &[u8]) -> bool {
    let name = name.trim();
    if name.is_empty() || data.is_empty() {
        return false;
    }
    let Some((face_index, score)) = best_mono_face(data) else {
        return false;
    };
    let key = name.to_ascii_lowercase();
    FONTS.with(|fonts| {
        let mut fonts = fonts.borrow_mut();
        if let Some(existing) = fonts.get(&key) {
            if existing.score >= score {
                return true;
            }
        }
        fonts.insert(
            key,
            RegisteredFont {
                display: name.to_string(),
                data: data.to_vec(),
                face_index,
                score,
            },
        );
        true
    })
}

/// Courier Prime first, then registered system families (no duplicates).
pub fn registered_families() -> Vec<String> {
    let mut names = vec![DEFAULT_FONT.to_string()];
    FONTS.with(|fonts| {
        let mut extra: Vec<String> = fonts
            .borrow()
            .values()
            .map(|f| f.display.clone())
            .filter(|n| !n.eq_ignore_ascii_case(DEFAULT_FONT))
            .collect();
        extra.sort_by_key(|a| a.to_ascii_lowercase());
        names.extend(extra);
    });
    names
}

/// Triangle vertices in a unit cell: `x` in `[0, 1]` (advance), `y` in `[0, 1]`
/// (`0` = top of the em box). Descenders may exceed `1`.
/// Unknown or empty `font` uses bundled Courier Prime.
pub fn glyph_triangles(font: &str, ch: char) -> Vec<[f32; 2]> {
    if ch.is_whitespace() {
        return Vec::new();
    }
    let key = cache_key(font);
    GLYPHS.with(|cache| {
        {
            let cache = cache.borrow();
            if let Some(v) = cache.get(&(key.clone(), ch)) {
                return v.clone();
            }
        }
        let tris = FONTS.with(|fonts| {
            let fonts = fonts.borrow();
            match fonts.get(&key) {
                Some(e) => tessellate_char(&e.data, e.face_index, ch),
                None => tessellate_char(FONT_DATA, 0, ch),
            }
        });
        cache.borrow_mut().insert((key, ch), tris.clone());
        tris
    })
}

#[cfg(not(target_arch = "wasm32"))]
struct FoundFont {
    family: String,
    data: Vec<u8>,
    score: i32,
}

/// Load monospace families from OS font directories (desktop tests and Tauri).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_system_monospace() -> Vec<(String, Vec<u8>)> {
    let mut by_key = HashMap::<String, FoundFont>::new();
    let mut paths = Vec::new();
    for dir in system_font_dirs() {
        collect_font_files(&dir, 6, &mut paths);
    }
    for path in paths {
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() || meta.len() == 0 || meta.len() > 12 * 1024 * 1024 {
            continue;
        }
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Some((index, score)) = best_mono_face(&data) else {
            continue;
        };
        let Ok(face) = Face::parse(&data, index) else {
            continue;
        };
        let Some(family) = family_name(&face) else {
            continue;
        };
        let key = family.to_ascii_lowercase();
        if key == DEFAULT_FONT.to_ascii_lowercase() {
            continue;
        }
        if let Some(prev) = by_key.get(&key) {
            if prev.score >= score {
                continue;
            }
        }
        by_key.insert(
            key,
            FoundFont {
                family,
                data,
                score,
            },
        );
    }
    let mut out: Vec<(String, Vec<u8>)> =
        by_key.into_values().map(|f| (f.family, f.data)).collect();
    out.sort_by_key(|a| a.0.to_ascii_lowercase());
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_font_files(dir: &std::path::Path, depth: u8, out: &mut Vec<std::path::PathBuf>) {
    if depth == 0 || out.len() >= 4000 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= 4000 {
            return;
        }
        let path = entry.path();
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_dir() {
            collect_font_files(&path, depth.saturating_sub(1), out);
            continue;
        }
        if !ft.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        if let Some("ttf" | "otf" | "ttc" | "otc") = ext.as_deref() {
            out.push(path);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn system_font_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(windows)]
    {
        if let Ok(root) = std::env::var("WINDIR") {
            dirs.push(std::path::PathBuf::from(root).join("Fonts"));
        } else {
            dirs.push(std::path::PathBuf::from(r"C:\Windows\Fonts"));
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(std::path::PathBuf::from(local).join("Microsoft\\Windows\\Fonts"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        dirs.push(std::path::PathBuf::from("/System/Library/Fonts"));
        dirs.push(std::path::PathBuf::from("/Library/Fonts"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(std::path::PathBuf::from(home).join("Library/Fonts"));
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        dirs.push(std::path::PathBuf::from("/usr/share/fonts"));
        dirs.push(std::path::PathBuf::from("/usr/local/share/fonts"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(std::path::PathBuf::from(home).join(".local/share/fonts"));
            dirs.push(std::path::PathBuf::from(home).join(".fonts"));
        }
    }
    dirs
}

struct Metrics {
    units: f32,
    ascent: f32,
    advance: f32,
}

fn metrics(face: &Face<'_>) -> Metrics {
    let units = f32::from(face.units_per_em());
    let ascent = f32::from(face.ascender());
    let gid = face.glyph_index('M').or_else(|| face.glyph_index('0'));
    let advance = gid
        .and_then(|g| face.glyph_hor_advance(g))
        .map(f32::from)
        .unwrap_or(units);
    Metrics {
        units,
        ascent,
        advance,
    }
}

struct LyonOutline {
    builder: lyon::path::Builder,
    started: bool,
}

impl OutlineBuilder for LyonOutline {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.started {
            self.builder.end(false);
            self.started = false;
        }
        self.builder.begin(point(x, y));
        self.started = true;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder.quadratic_bezier_to(point(x1, y1), point(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder
            .cubic_bezier_to(point(x1, y1), point(x2, y2), point(x, y));
    }

    fn close(&mut self) {
        if self.started {
            self.builder.close();
            self.started = false;
        }
    }
}

fn tessellate_char(data: &[u8], face_index: u32, ch: char) -> Vec<[f32; 2]> {
    let Ok(face) = Face::parse(data, face_index) else {
        return Vec::new();
    };
    let m = metrics(&face);
    let gid = face
        .glyph_index(ch)
        .or_else(|| face.glyph_index('?'))
        .unwrap_or(ttf_parser::GlyphId(0));

    let mut outline = LyonOutline {
        builder: Path::builder(),
        started: false,
    };
    if face.outline_glyph(gid, &mut outline).is_none() {
        return Vec::new();
    }
    if outline.started {
        outline.builder.end(false);
    }
    let path = outline.builder.build();
    if path.iter().next().is_none() {
        return Vec::new();
    }

    let mut buffers: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let opts = FillOptions::tolerance((m.units * 0.012).max(0.5));
    let _ = tess.tessellate_path(
        &path,
        &opts,
        &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| {
            let p = v.position();
            [p.x, p.y]
        }),
    );

    let adv = m.advance.max(1.0);
    let mut out = Vec::with_capacity(buffers.indices.len());
    for tri in buffers.indices.chunks(3) {
        if tri.len() != 3 {
            continue;
        }
        for &i in tri {
            if let Some(&[x, y]) = buffers.vertices.get(i as usize) {
                out.push([x / adv, (m.ascent - y) / m.units]);
            }
        }
    }
    out
}

#[cfg(test)]
pub fn reset_registry() {
    FONTS.with(|f| f.borrow_mut().clear());
    GLYPHS.with(|g| g.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;

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
