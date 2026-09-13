//! Compile-time hooks for an optional private module linked into the WASM binary.
//!
//! The public crate never depends on premium code. A sibling crate registers a
//! dispatch function via [`install`] before the editor starts (typically
//! `#[wasm_bindgen(start)]`).

use std::sync::OnceLock;

use crate::Editor;

/// `(name, json payload) -> json result`.
pub type ExtDispatch = fn(editor: &mut Editor, name: &str, payload: &str) -> Result<String, String>;
pub type ExtCaps = fn() -> &'static str;

static DISPATCH: OnceLock<ExtDispatch> = OnceLock::new();
static CAPS: OnceLock<ExtCaps> = OnceLock::new();

/// Register premium dispatch. First caller wins; later calls are ignored.
pub fn install(dispatch: ExtDispatch, caps: ExtCaps) {
    let _ = DISPATCH.set(dispatch);
    let _ = CAPS.set(caps);
}

pub fn dispatch(editor: &mut Editor, name: &str, payload: &str) -> Result<String, String> {
    match DISPATCH.get() {
        Some(f) => f(editor, name, payload),
        None => Err(format!("unknown extension command: {name}")),
    }
}

pub fn installed() -> bool {
    DISPATCH.get().is_some()
}

pub fn capabilities() -> String {
    match CAPS.get() {
        Some(f) => f().to_string(),
        None => "{\"pro\":false,\"commands\":[]}".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::LibrarySet;

    fn ping(ed: &mut Editor, name: &str, _payload: &str) -> Result<String, String> {
        if name != "ping" {
            return Err(format!("unknown: {name}"));
        }
        Ok(format!(
            "{{\"sum\":{},\"n\":{}}}",
            1 + 1,
            ed.doc().primitives.len()
        ))
    }

    fn caps() -> &'static str {
        r#"{"pro":true,"commands":["ping"]}"#
    }

    #[test]
    fn dispatch_before_and_after_install() {
        let mut ed = Editor::new(LibrarySet::new());
        if !installed() {
            let err = dispatch(&mut ed, "ping", "{}").unwrap_err();
            assert!(err.contains("unknown extension command"), "{err}");
            assert!(capabilities().contains("\"pro\":false"));
            install(ping, caps);
        }
        let out = dispatch(&mut ed, "ping", "{}").unwrap();
        assert!(out.contains("\"sum\":2"), "{out}");
        assert!(installed());
        assert!(capabilities().contains("\"pro\":true"));
    }
}
