#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use tauri::{Manager, State};

#[cfg(any(target_os = "macos", target_os = "ios"))]
use tauri::Emitter;

#[derive(Serialize)]
struct SystemMonoFont {
    family: String,
    data: String,
}

struct PendingOpens(Mutex<Vec<PathBuf>>);

#[derive(Serialize)]
struct OpenedFile {
    name: String,
    kind: String,
    data: Vec<u8>,
}

#[tauri::command]
fn system_mono_fonts() -> Vec<SystemMonoFont> {
    fidorust_gpu::font::load_system_monospace()
        .into_iter()
        .map(|(family, data)| SystemMonoFont {
            family,
            data: STANDARD.encode(data),
        })
        .collect()
}

fn file_kind(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()? {
        ext if ext.eq_ignore_ascii_case("fcd") => Some("fcd"),
        ext if ext.eq_ignore_ascii_case("fcl") => Some("fcl"),
        _ => None,
    }
}

fn parse_open_path(raw: &str) -> Option<PathBuf> {
    if raw.starts_with('-') {
        return None;
    }
    let path = PathBuf::from(raw);
    if file_kind(&path).is_some() && path.is_file() {
        Some(path)
    } else {
        None
    }
}

#[cfg(any(windows, target_os = "linux"))]
fn collect_launch_paths() -> Vec<PathBuf> {
    std::env::args()
        .skip(1)
        .filter_map(|arg| parse_open_path(&arg))
        .collect()
}

fn read_open_path(path: &Path) -> Option<OpenedFile> {
    let kind = file_kind(path)?.to_string();
    let name = path.file_name()?.to_string_lossy().into_owned();
    let data = std::fs::read(path).ok()?;
    Some(OpenedFile { name, kind, data })
}

fn drain_pending(pending: &PendingOpens) -> Vec<PathBuf> {
    pending
        .0
        .lock()
        .map(|mut guard| guard.drain(..).collect())
        .unwrap_or_default()
}

#[tauri::command]
fn take_pending_opens(pending: State<PendingOpens>) -> Vec<OpenedFile> {
    drain_pending(&pending)
        .iter()
        .filter_map(|path| read_open_path(path))
        .collect()
}

#[tauri::command]
fn read_open_files(paths: Vec<String>) -> Vec<OpenedFile> {
    paths
        .iter()
        .filter_map(|raw| parse_open_path(raw))
        .filter_map(|path| read_open_path(&path))
        .collect()
}

#[tauri::command]
async fn fetch_pro_module(url: String) -> Result<String, String> {
    if !pro_module_url_allowed(&url) {
        return Err("blocked module url".into());
    }
    let bytes = reqwest::get(&url)
        .await
        .and_then(|res| res.error_for_status())
        .map_err(|err| err.to_string())?
        .bytes()
        .await
        .map_err(|err| err.to_string())?;
    Ok(STANDARD.encode(bytes))
}

fn pro_module_url_allowed(url: &str) -> bool {
    const PREFIXES: &[&str] = &["https://oneiros90.github.io/fidorust/"];
    if PREFIXES.iter().any(|prefix| url.starts_with(prefix)) {
        return true;
    }
    cfg!(debug_assertions)
        && (url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:"))
        && url.contains("/pro/")
        && (url.ends_with("/fidorust-pro.bin") || url.ends_with("/fidorust-pro.dat"))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(PendingOpens(Mutex::new(Vec::new())))
        .invoke_handler(tauri::generate_handler![
            system_mono_fonts,
            take_pending_opens,
            read_open_files,
            fetch_pro_module
        ])
        .setup(|#[allow(unused_variables)] app| {
            #[cfg(any(windows, target_os = "linux"))]
            {
                let files = collect_launch_paths();
                if !files.is_empty() {
                    app.state::<PendingOpens>()
                        .0
                        .lock()
                        .expect("pending opens")
                        .extend(files);
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running FidoRust")
        .run(
            |#[allow(unused_variables)] app, #[allow(unused_variables)] event| {
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                if let tauri::RunEvent::Opened { urls } = event {
                    let files: Vec<PathBuf> = urls
                        .into_iter()
                        .filter_map(|url| url.to_file_path().ok())
                        .filter(|path| file_kind(path).is_some() && path.is_file())
                        .collect();
                    if files.is_empty() {
                        return;
                    }
                    app.state::<PendingOpens>()
                        .0
                        .lock()
                        .expect("pending opens")
                        .extend(files);
                    let _ = app.emit("open-files-ready", ());
                }
            },
        );
}
