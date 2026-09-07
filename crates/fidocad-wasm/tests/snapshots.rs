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
