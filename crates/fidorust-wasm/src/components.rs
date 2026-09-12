//! Component and library helpers used by the WASM façade.

use fidorust_core::parse::parse_library;
use fidorust_core::serialize::serialize_library;
use fidorust_core::LibraryKind;
use fidorust_gpu::tessellate::{scene_to_thumb_svg, tessellate_primitives};
use wasm_bindgen::JsValue;

use crate::json::{
    to_json, ComponentCursorDto, CreatedComponentDto, UnresolvedComponentDto, UserLibBlob,
};
use crate::{to_js, App};

pub(crate) fn set_pending_component(app: &mut App, name: &str) {
    app.editor.set_pending_component(Some(name.to_string()));
    app.editor.adopt_component_tool();
    app.editor.clear_hover();
}

pub(crate) fn set_pending_follow(app: &mut App, on: bool) {
    app.editor.set_pending_follow(on);
}

pub(crate) fn place_component_at(app: &mut App, name: &str, sx: f32, sy: f32) {
    if app.editor.pending_component() != Some(name) {
        app.editor.set_pending_component(Some(name.to_string()));
    }
    app.editor.adopt_component_tool();
    let w = app.editor.screen_to_world(sx, sy);
    app.editor.place_dropped_component(w);
}

pub(crate) fn split_selected_components(app: &mut App) {
    app.editor.split_selected_components();
}

pub(crate) fn component_preview_svg(app: &App, name: &str) -> String {
    let scene = component_scene(app, name);
    scene_to_thumb_svg(&scene, 40.0)
}

pub(crate) fn component_cursor_json(app: &App, name: &str) -> String {
    use fidorust_core::COMPONENT_ORIGIN;
    use fidorust_gpu::scene_to_cursor_svg;
    let scene = component_scene(app, name);
    let cur = scene_to_cursor_svg(&scene, COMPONENT_ORIGIN);
    to_json(
        &ComponentCursorDto {
            svg: cur.svg,
            ox: cur.ox,
            oy: cur.oy,
            w: cur.w,
            h: cur.h,
        },
        "{}",
    )
}

pub(crate) fn unresolved_components_json(app: &App) -> String {
    let items: Vec<UnresolvedComponentDto> = app
        .editor
        .unresolved_components()
        .into_iter()
        .map(|(name, count)| UnresolvedComponentDto { name, count })
        .collect();
    to_json(&items, "[]")
}

pub(crate) fn library_json(app: &App) -> String {
    to_json(&app.editor.libs().tree(), "[]")
}

pub(crate) fn user_libraries_json(app: &App) -> String {
    to_json(&app.editor.libs().user_library_list(), "[]")
}

pub(crate) fn create_component_from_selection(
    app: &mut App,
    target: &str,
    display_name: &str,
) -> String {
    match app
        .editor
        .create_component_from_selection(target, display_name)
    {
        Some((stem, key)) => to_json(&CreatedComponentDto { stem, key }, "{}"),
        None => String::new(),
    }
}

pub(crate) fn enter_component_edit(app: &mut App, stem: &str, key: &str) -> bool {
    let ok = app.editor.enter_component_edit(stem, key);
    if ok {
        app.editor.fit_view(app.width, app.height);
    }
    ok
}

pub(crate) fn edit_selected_component(app: &mut App) -> bool {
    let Some((stem, key)) = app.editor.selected_editable_component() else {
        return false;
    };
    enter_component_edit(app, &stem, &key)
}

pub(crate) fn save_component_edit(app: &mut App) -> bool {
    app.editor.save_component_edit()
}

pub(crate) fn local_component_uses_nonzero_layers(app: &App, stem: &str, key: &str) -> bool {
    app.editor.local_component_uses_nonzero_layers(stem, key)
}

pub(crate) fn editing_local_component_uses_nonzero_layers(app: &App) -> bool {
    app.editor.editing_local_component_uses_nonzero_layers()
}

pub(crate) fn cancel_component_edit(app: &mut App) -> bool {
    app.editor.cancel_component_edit()
}

pub(crate) fn rename_component(app: &mut App, stem: &str, key: &str, name: &str) -> bool {
    app.editor.rename_component(stem, key, name)
}

pub(crate) fn rename_component_key(app: &mut App, stem: &str, key: &str, new_key: &str) -> bool {
    app.editor.rename_component_key(stem, key, new_key)
}

pub(crate) fn delete_component(app: &mut App, stem: &str, key: &str) -> bool {
    app.editor.delete_component(stem, key)
}

pub(crate) fn move_component(app: &mut App, stem: &str, key: &str, dest_stem: &str) -> String {
    app.editor
        .move_component(stem, key, dest_stem)
        .unwrap_or_default()
}

pub(crate) fn load_user_libraries(app: &mut App, json: &str) {
    let blobs: Vec<UserLibBlob> = serde_json::from_str(json).unwrap_or_default();
    let mut libs = Vec::new();
    for b in blobs {
        if b.fcl.trim().is_empty() {
            if !b.stem.is_empty() {
                let mut lib = fidorust_core::Library::empty_user(
                    b.stem,
                    if b.title.is_empty() {
                        "Library".into()
                    } else {
                        b.title
                    },
                );
                lib.aliases = b.aliases;
                libs.push(lib);
            }
            continue;
        }
        if let Ok(mut lib) = parse_library(&b.fcl) {
            if !b.stem.is_empty() {
                lib.file_stem = b.stem;
            }
            if !b.title.is_empty() {
                lib.name = b.title;
            }
            lib.aliases = b.aliases;
            lib.kind = LibraryKind::Local;
            lib.standard = false;
            libs.push(lib);
        }
    }
    app.editor.load_user_libraries(libs);
}

pub(crate) fn user_libraries_blob(app: &App) -> String {
    let blobs: Vec<UserLibBlob> = app
        .editor
        .libs()
        .user_libraries()
        .map(|lib| UserLibBlob {
            stem: lib.file_stem.clone(),
            title: lib.name.clone(),
            fcl: serialize_library(lib),
            aliases: lib.aliases.clone(),
        })
        .collect();
    to_json(&blobs, "[]")
}

pub(crate) fn create_user_library(app: &mut App, title: &str) -> String {
    app.editor.create_user_library(title)
}

pub(crate) fn import_library(app: &mut App, text: &str, filename: &str) -> Result<String, JsValue> {
    let mut lib = parse_library(text).map_err(to_js)?;
    lib.add_filename_alias(filename);
    Ok(app.editor.import_library(lib))
}

pub(crate) fn rename_library(app: &mut App, stem: &str, title: &str) -> String {
    app.editor.rename_library(stem, title).unwrap_or_default()
}

pub(crate) fn export_library_fcl(app: &App, stem: &str) -> String {
    app.editor
        .libs()
        .library(stem)
        .map(serialize_library)
        .unwrap_or_default()
}

pub(crate) fn clear_project_library(app: &mut App) -> bool {
    app.editor.clear_project_library()
}

pub(crate) fn remove_user_library(app: &mut App, stem: &str) -> bool {
    app.editor.remove_user_library(stem)
}

fn component_scene(app: &App, name: &str) -> fidorust_gpu::Scene {
    use fidorust_core::geom::Transform;
    use fidorust_core::library::expand_component;
    use fidorust_core::COMPONENT_ORIGIN;
    let Some((_, def)) = app.editor.libs().lookup(name) else {
        return fidorust_gpu::Scene::default();
    };
    let prims = expand_component(
        def,
        Transform {
            origin: COMPONENT_ORIGIN,
            rotations: 0,
            mirrored: false,
        },
        app.editor.libs(),
        0,
    );
    tessellate_primitives(&prims, &app.editor.doc().layers)
}
