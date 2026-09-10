mod common;

use fidocad_wasm::App;

#[test]
fn snapshot_empty_status_layers_library() {
    let app = App::new();
    common::assert_snapshot("status.json", &pretty(&app.status_json()));
    common::assert_snapshot("layers.json", &pretty(&app.layers_json()));
    common::assert_snapshot("library.json", &pretty(&app.library_json()));
    common::assert_snapshot(
        "world_to_screen.json",
        &pretty(&app.world_to_screen_json(10.0, 20.0)),
    );
}

#[test]
fn snapshot_alimentatore_status_and_dblclick() {
    let mut app = App::new();
    app.load_fcd(include_str!("../../fidocad-core/tests/Alimentatore.fcd"))
        .unwrap();
    common::assert_snapshot("alimentatore_status.json", &pretty(&app.status_json()));
    common::assert_snapshot("alimentatore_layers.json", &pretty(&app.layers_json()));
    let click = app.dblclick(0.0, 0.0);
    common::assert_snapshot("alimentatore_dblclick.json", &click);
}

fn pretty(raw: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(raw).unwrap_or(serde_json::Value::Null);
    serde_json::to_string_pretty(&v).unwrap()
}

#[test]
fn set_view_roundtrip() {
    let mut app = App::new();
    app.set_view(8.0, 12.0, 34.0);
    let status: serde_json::Value = serde_json::from_str(&app.status_json()).unwrap();
    assert_eq!(status["zoom"].as_f64(), Some(8.0));
    assert_eq!(status["pan_x"].as_f64(), Some(12.0));
    assert_eq!(status["pan_y"].as_f64(), Some(34.0));
}

#[test]
fn registered_fonts_start_with_courier_prime() {
    let app = App::new();
    let fonts: Vec<String> = serde_json::from_str(&app.registered_fonts_json()).unwrap();
    assert_eq!(fonts.first().map(String::as_str), Some("Courier Prime"));
}
