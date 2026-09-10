use fidocad_core::library::{LibraryKind, PROJECT_STEM};
use fidocad_core::parse::{
    builtin_libraries, parse_document, parse_document_with_project_library, parse_library,
};
use fidocad_core::serialize::{
    serialize_document, serialize_document_with_policy, serialize_library, serialize_primitive,
    SaveLibraryPolicy,
};
use fidocad_core::{Editor, LayerId, Line, Point, Primitive, Tool};

#[test]
fn create_from_selection_replaces_with_instance() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(10, 20),
        Point::new(30, 20),
        LayerId(0),
    ));
    ed.doc_mut().insert(Primitive::line(
        Point::new(10, 40),
        Point::new(30, 40),
        LayerId(0),
    ));
    ed.set_selected(vec![0, 1]);
    let created = ed
        .create_component_from_selection(PROJECT_STEM, "Nuovo componente")
        .expect("created");
    assert_eq!(created.0, PROJECT_STEM);
    assert_eq!(ed.doc().primitives.len(), 1);
    assert!(ed.doc().primitives[0].is_component());
    let lib = ed.libs().project().unwrap();
    let def = lib.find(&created.1).unwrap();
    assert_eq!(def.name, "Nuovo componente");
    assert_eq!(def.primitives.len(), 2);
}

#[test]
fn project_library_roundtrips_in_fcd() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::Line(Line {
        a: Point::new(5, 5),
        b: Point::new(15, 5),
        layer: LayerId(0),
    }));
    ed.set_selected(vec![0]);
    ed.create_component_from_selection(PROJECT_STEM, "Box")
        .unwrap();
    let text = serialize_document(ed.doc(), Some(ed.libs()));
    assert!(text.contains("[FIDOLIB project]"));
    assert!(text.contains("[C01 Box]"));
    let (doc, project) = parse_document_with_project_library(&text).unwrap();
    assert_eq!(doc.primitives.len(), 1);
    assert!(doc.primitives[0].is_component());
    let project = project.expect("project lib");
    assert_eq!(project.components.len(), 1);
    assert_eq!(project.components[0].name, "Box");
}

#[test]
fn save_keeps_project_refs() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(5, 5),
        Point::new(15, 5),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    ed.create_component_from_selection(PROJECT_STEM, "Box")
        .unwrap();
    let text = serialize_document(ed.doc(), Some(ed.libs()));
    assert!(text.contains("project.C01"), "{text}");
    assert!(text.contains("[FIDOLIB project]"));
}

#[test]
fn save_keeps_local_refs() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(5, 5),
        Point::new(15, 5),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let stem = ed.create_user_library("Mine");
    ed.create_component_from_selection(&stem, "Box").unwrap();
    let text = serialize_document(ed.doc(), Some(ed.libs()));
    assert!(text.contains(&format!("{stem}.C01")), "{text}");
    assert!(!text.contains("[FIDOLIB Mine]"), "{text}");
    assert!(!text.contains(&format!("[FIDOLIB {stem}]")), "{text}");
}

#[test]
fn parser_does_not_ingest_embedded_library_primitives() {
    let src = "\
[FIDOCAD]
LI 0 0 10 0
[FIDOLIB project]
[C01 Extra]
LI 100 100 120 100
";
    let doc = parse_document(src).unwrap();
    assert_eq!(doc.primitives.len(), 1);
    let (_, project) = parse_document_with_project_library(src).unwrap();
    let project = project.unwrap();
    assert_eq!(project.components.len(), 1);
    assert_eq!(project.components[0].primitives.len(), 1);
}

#[test]
fn delete_component_splits_instances() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(10, 0),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let (_, key) = ed
        .create_component_from_selection(PROJECT_STEM, "A")
        .unwrap();
    assert_eq!(ed.doc().primitives.len(), 1);
    assert!(ed.delete_component(PROJECT_STEM, &key));
    assert!(ed.libs().project().unwrap().find(&key).is_none());
    assert_eq!(ed.doc().primitives.len(), 1);
    assert!(!ed.doc().primitives[0].is_component());
}

#[test]
fn move_component_rewrites_refs() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(8, 0),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let (_, key) = ed
        .create_component_from_selection(PROJECT_STEM, "Moved")
        .unwrap();
    let dest = ed.create_user_library("Device");
    let dest_key = ed.move_component(PROJECT_STEM, &key, &dest).unwrap();
    assert!(ed.libs().project().unwrap().find(&key).is_none());
    assert!(ed.libs().library(&dest).unwrap().find(&dest_key).is_some());
    match &ed.doc().primitives[0] {
        Primitive::Component(c) => assert_eq!(c.name, format!("{dest}.{dest_key}")),
        other => panic!("expected component, got {other:?}"),
    }
}

#[test]
fn component_edit_save_and_cancel() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(10, 0),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let (_, key) = ed
        .create_component_from_selection(PROJECT_STEM, "Edit me")
        .unwrap();
    assert!(ed.enter_component_edit(PROJECT_STEM, &key));
    assert_eq!(ed.doc().primitives.len(), 1);
    ed.set_tool(Tool::Select);
    ed.set_selected(vec![0]);
    ed.delete_selected();
    assert!(ed.doc().primitives.is_empty());
    assert!(ed.cancel_component_edit());
    let def = ed.libs().project().unwrap().find(&key).unwrap();
    assert_eq!(def.primitives.len(), 1);

    assert!(ed.enter_component_edit(PROJECT_STEM, &key));
    ed.set_selected(vec![0]);
    ed.delete_selected();
    assert!(ed.save_component_edit());
    let def = ed.libs().project().unwrap().find(&key).unwrap();
    assert!(def.primitives.is_empty());
}

#[test]
fn description_roundtrip() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(1, 1),
        Point::new(2, 2),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let (_, key) = ed
        .create_component_from_selection(PROJECT_STEM, "Named")
        .unwrap();
    assert!(ed.set_component_description(PROJECT_STEM, &key, "A note"));
    let text = serialize_document(ed.doc(), Some(ed.libs()));
    assert!(text.contains("DS A note"));
    let (_, project) = parse_document_with_project_library(&text).unwrap();
    assert_eq!(project.unwrap().components[0].description, "A note");
}

#[test]
fn create_flattens_body_and_places_instance_on_current_layer() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(10, 0),
        LayerId(0),
    ));
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 10),
        Point::new(10, 10),
        LayerId(1),
    ));
    ed.set_layer(2);
    ed.set_selected(vec![0, 1]);
    ed.create_component_from_selection(PROJECT_STEM, "Mix")
        .unwrap();
    match &ed.doc().primitives[0] {
        Primitive::Component(c) => assert_eq!(c.layer.0, 2),
        _ => panic!("expected component instance"),
    }
    let def = ed.libs().project().unwrap().components.last().unwrap();
    assert!(
        def.primitives.iter().all(|p| p.layer().0 == 0),
        "definition must be flattened to layer 0"
    );
}

#[test]
fn instance_expansion_uses_instance_layer() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(10, 0),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    ed.create_component_from_selection(PROJECT_STEM, "A")
        .unwrap();
    ed.set_selected(vec![0]);
    ed.set_layer(1);
    assert_eq!(ed.doc().primitives[0].layer().0, 1);
    let flat = fidocad_core::library::expand_primitive(&ed.doc().primitives[0], ed.libs());
    assert!(!flat.is_empty());
    assert!(flat.iter().all(|p| p.layer().0 == 1));
}

#[test]
fn component_edit_locks_layers() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(10, 0),
        LayerId(1),
    ));
    ed.set_selected(vec![0]);
    let (_, key) = ed
        .create_component_from_selection(PROJECT_STEM, "A")
        .unwrap();
    assert!(ed.enter_component_edit(PROJECT_STEM, &key));
    assert_eq!(ed.doc().layers.len(), 1);
    assert_eq!(ed.layer().0, 0);
    assert!(ed.add_layer().is_none());
    ed.set_layer(3);
    assert_eq!(ed.layer().0, 0);
    ed.set_selected(vec![0]);
    assert!(ed
        .selection_props_form()
        .iter()
        .all(|f| f.id != fidocad_core::properties::PropField::Layer));
}

#[test]
fn builtin_set_has_no_default_local_library() {
    let libs = builtin_libraries();
    assert!(libs.project().is_some());
    assert!(libs.local().is_none());
    assert_eq!(libs.user_libraries().count(), 0);
}

#[test]
fn create_user_library_unique_stem() {
    let mut ed = Editor::new(builtin_libraries());
    let a = ed.create_user_library("Mine");
    let b = ed.create_user_library("Mine");
    assert_eq!(a, "Mine");
    assert_eq!(b, "Mine 2");
    assert!(ed.libs().library(&a).is_some());
    assert!(ed.libs().library(&b).is_some());
}

#[test]
fn import_library_from_fcl() {
    let mut ed = Editor::new(builtin_libraries());
    let fcl = "\
[FIDOLIB symbols]
[C01 Box]
LI 100 100 120 100
";
    let lib = parse_library(fcl).unwrap();
    let stem = ed.import_library(lib);
    assert_eq!(stem, "symbols");
    let lib = ed.libs().library(&stem).unwrap();
    assert_eq!(lib.kind, LibraryKind::Local);
    assert_eq!(lib.components.len(), 1);
    assert_eq!(lib.components[0].name, "Box");
}

#[test]
fn rename_user_library_keeps_fcd_prefix() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(8, 0),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let stem = ed.create_user_library("Mine");
    ed.create_component_from_selection(&stem, "Box").unwrap();
    let renamed = ed.rename_library(&stem, "Other").unwrap();
    assert_eq!(renamed, "Mine");
    let lib = ed.libs().library(&stem).unwrap();
    assert_eq!(lib.name, "Other");
    assert_eq!(lib.file_stem, "Mine");
    match &ed.doc().primitives[0] {
        Primitive::Component(c) => assert_eq!(c.name, "Mine.C01"),
        other => panic!("expected component, got {other:?}"),
    }
}

#[test]
fn remove_user_library_explodes_and_drops_slot() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(10, 0),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let stem = ed.create_user_library("Mine");
    ed.create_component_from_selection(&stem, "A").unwrap();
    assert!(ed.remove_user_library(&stem));
    assert!(ed.libs().library(&stem).is_none());
    assert_eq!(ed.doc().primitives.len(), 1);
    assert!(!ed.doc().primitives[0].is_component());
}

#[test]
fn clear_project_library_explodes_and_keeps_slot() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(0, 0),
        Point::new(10, 0),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    ed.create_component_from_selection(PROJECT_STEM, "A")
        .unwrap();
    assert!(ed.clear_project_library());
    assert!(ed.libs().project().unwrap().components.is_empty());
    assert_eq!(ed.doc().primitives.len(), 1);
    assert!(!ed.doc().primitives[0].is_component());
}

#[test]
fn save_policy_fold_copies_into_project() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(5, 5),
        Point::new(15, 5),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let stem = ed.create_user_library("Mine");
    ed.create_component_from_selection(&stem, "Box").unwrap();
    assert!(ed.uses_user_library_components());
    let text =
        serialize_document_with_policy(ed.doc(), ed.libs(), SaveLibraryPolicy::FoldIntoProject);
    assert!(text.contains("project.C01"), "{text}");
    assert!(text.contains("[FIDOLIB project]"), "{text}");
    assert!(!text.contains(&format!("{stem}.C01")), "{text}");
    match &ed.doc().primitives[0] {
        Primitive::Component(c) => assert_eq!(c.name, format!("{stem}.C01")),
        other => panic!("editor must stay local, got {other:?}"),
    }
    assert!(ed.libs().project().unwrap().components.is_empty());
}

#[test]
fn save_policy_explode_writes_primitives() {
    let mut ed = Editor::new(builtin_libraries());
    ed.doc_mut().insert(Primitive::line(
        Point::new(5, 5),
        Point::new(15, 5),
        LayerId(0),
    ));
    ed.set_selected(vec![0]);
    let stem = ed.create_user_library("Mine");
    ed.create_component_from_selection(&stem, "Box").unwrap();
    let text = serialize_document_with_policy(ed.doc(), ed.libs(), SaveLibraryPolicy::ExplodeUser);
    assert!(!text.contains("MC "), "{text}");
    assert!(text.contains("LI "), "{text}");
    match &ed.doc().primitives[0] {
        Primitive::Component(_) => {}
        other => panic!("editor must stay as instance, got {other:?}"),
    }
}

const SOLID_STATE_FCL: &str = "\
[FIDOLIB Componenti stato solido]
[CS11 Operazionale]
LI 108 100 110 100
PV 97 93 97 107 108 100
";

#[test]
fn import_preserves_spaced_fidolib_title_as_stem() {
    let mut ed = Editor::new(builtin_libraries());
    let lib = parse_library(SOLID_STATE_FCL).unwrap();
    let stem = ed.import_library(lib);
    assert_eq!(stem, "Componenti stato solido");
    let lib = ed.libs().library(&stem).unwrap();
    assert_eq!(lib.name, "Componenti stato solido");
    assert_eq!(lib.file_stem, "Componenti stato solido");
    let fcl = serialize_library(lib);
    assert!(fcl.contains("[FIDOLIB Componenti stato solido]"), "{fcl}");
}

#[test]
fn lookup_resolves_spaced_mc_prefix_from_imported_fcl() {
    let mut ed = Editor::new(builtin_libraries());
    let lib = parse_library(SOLID_STATE_FCL).unwrap();
    ed.import_library(lib);
    let found = ed
        .libs()
        .lookup("Componenti stato solido.CS11")
        .expect("CS11 should resolve");
    assert_eq!(found.1.key, "CS11");
    assert_eq!(found.1.name, "Operazionale");

    ed.load_text("[FIDOCAD]\nMC 145 80 2 1 Componenti stato solido.CS11\n")
        .unwrap();
    let expanded = fidocad_core::library::expand_primitive(&ed.doc().primitives[0], ed.libs());
    assert!(
        expanded.iter().all(|p| !p.is_component()),
        "CS11 must expand to drawing primitives"
    );
    assert!(expanded.len() >= 2, "got {} primitives", expanded.len());
    assert_eq!(ed.unresolved_component_count(), 0);
}

#[test]
fn lookup_resolves_spaced_prefix_against_already_sanitized_stem() {
    let mut ed = Editor::new(builtin_libraries());
    let mut lib = parse_library(SOLID_STATE_FCL).unwrap();
    lib.file_stem = fidocad_core::library::sanitize_stem("Componenti stato solido");
    assert_eq!(lib.file_stem, "Componentistatosolido");
    ed.import_library(lib);
    let found = ed
        .libs()
        .lookup("Componenti stato solido.CS11")
        .expect("normalized stem match");
    assert_eq!(found.0.file_stem, "Componentistatosolido");
    assert_eq!(found.1.key, "CS11");
}

#[test]
fn lookup_uses_fcl_filename_alias_when_it_differs_from_title() {
    let mut ed = Editor::new(builtin_libraries());
    let mut lib = parse_library(SOLID_STATE_FCL).unwrap();
    lib.add_filename_alias("ihjh.fcl");
    let stem = ed.import_library(lib);
    assert_eq!(stem, "Componenti stato solido");
    assert!(ed.libs().lookup("ihjh.CS11").is_some());
    assert!(ed.libs().lookup("Componenti stato solido.CS11").is_some());
}

#[test]
fn nested_mc_with_spaced_library_prefix_expands() {
    let mut ed = Editor::new(builtin_libraries());
    let dots = parse_library(
        "\
[FIDOLIB Cerchietti]
[M01 Dot]
LI 100 100 102 100
",
    )
    .unwrap();
    ed.import_library(dots);
    let relay = parse_library(
        "\
[FIDOLIB Componenti elettromeccanici]
[Ce1 Relè]
MC 100 100 0 0 Cerchietti.M01
",
    )
    .unwrap();
    ed.import_library(relay);
    let found = ed
        .libs()
        .lookup("Componenti elettromeccanici.Ce1")
        .expect("relay");
    let expanded = fidocad_core::library::expand_component(
        found.1,
        fidocad_core::Transform {
            origin: Point::new(50, 50),
            rotations: 0,
            mirrored: false,
        },
        ed.libs(),
        0,
    );
    assert_eq!(expanded.len(), 1);
    assert!(!expanded[0].is_component());
}

#[test]
fn unresolved_mc_is_counted_and_kept_as_instance() {
    let mut ed = Editor::new(builtin_libraries());
    ed.load_text("[FIDOCAD]\nMC 145 80 2 1 Componenti stato solido.CS11\nMC 170 95 2 1 Componenti stato solido.CS11\n")
        .unwrap();
    assert_eq!(ed.unresolved_component_count(), 2);
    assert_eq!(
        ed.unresolved_components(),
        vec![("Componenti stato solido.CS11".into(), 2)]
    );
    let expanded = fidocad_core::library::expand_primitive(&ed.doc().primitives[0], ed.libs());
    assert_eq!(expanded.len(), 1);
    assert!(expanded[0].is_component());
}

#[test]
fn create_user_library_keeps_spaces_in_stem() {
    let mut ed = Editor::new(builtin_libraries());
    let stem = ed.create_user_library("My Library");
    assert_eq!(stem, "My Library");
}

fn plus_terminal_component() -> Editor {
    let mut ed = Editor::new(builtin_libraries());
    let src =
        "[FIDOCAD]\nLI 320 60 315 60 1\nPP 320 60 318 61 318 59 1\nTY 323 62 4 2 180 1 1 * +\n";
    let doc = parse_document(src).unwrap();
    for p in doc.primitives {
        ed.doc_mut().insert(p);
    }
    ed.set_selected(vec![0, 1, 2]);
    ed.create_component_from_selection(PROJECT_STEM, "Plus")
        .expect("created");
    ed
}

fn dump_doc_primitives(ed: &Editor) -> String {
    ed.doc()
        .primitives
        .iter()
        .map(serialize_primitive)
        .collect()
}

#[test]
fn rotating_a_component_matches_rotating_its_parts() {
    let mut as_component = plus_terminal_component();
    as_component.rotate_selected();
    as_component.split_selected_components();

    let mut as_parts = plus_terminal_component();
    as_parts.split_selected_components();
    as_parts.select_all();
    as_parts.rotate_selected();

    assert_eq!(
        dump_doc_primitives(&as_component),
        dump_doc_primitives(&as_parts)
    );
}

#[test]
fn rotating_a_component_twice_matches_rotating_its_parts_twice() {
    let mut as_component = plus_terminal_component();
    as_component.rotate_selected();
    as_component.rotate_selected();
    as_component.split_selected_components();

    let mut as_parts = plus_terminal_component();
    as_parts.split_selected_components();
    as_parts.select_all();
    as_parts.rotate_selected();
    as_parts.rotate_selected();

    assert_eq!(
        dump_doc_primitives(&as_component),
        dump_doc_primitives(&as_parts)
    );
}
