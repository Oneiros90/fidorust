use std::path::PathBuf;

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn assert_snapshot(name: &str, actual: &str) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
    std::fs::create_dir_all(&dir).expect("create snapshots dir");
    let path = dir.join(name);
    if std::env::var("UPDATE_SNAPSHOTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, normalize_newlines(actual))
            .unwrap_or_else(|e| panic!("write {name}: {e}"));
        return;
    }
    let expected =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing snapshot {name}: {e}"));
    assert_eq!(
        normalize_newlines(&expected),
        normalize_newlines(actual),
        "snapshot mismatch: {name}"
    );
}
