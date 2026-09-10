#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;

#[derive(Serialize)]
struct SystemMonoFont {
    family: String,
    data: String,
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![system_mono_fonts])
        .run(tauri::generate_context!())
        .expect("error while running FidoRust");
}
