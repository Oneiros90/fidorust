//! OS font-directory scan (desktop tests and Tauri).

use std::collections::HashMap;

use fidorust_core::primitive::DEFAULT_FONT;
use ttf_parser::Face;

use super::registry::best_mono_face;
use super::registry::family_name;

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
            let home = std::path::PathBuf::from(home);
            dirs.push(home.join(".local/share/fonts"));
            dirs.push(home.join(".fonts"));
        }
    }
    dirs
}
