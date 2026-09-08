use fidocad_core::library::{LibraryKind, PROJECT_STEM};
use fidocad_core::parse::{
    builtin_libraries, parse_document, parse_document_with_project_library, parse_library,
};
use fidocad_core::serialize::{
    serialize_document, serialize_document_with_policy, SaveLibraryPolicy,
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
    assert_eq!(b, "Mine2");
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
fn rename_user_library_rewrites_refs() {
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
    assert_eq!(renamed, "Other");
    assert!(ed.libs().library(&stem).is_none());
    match &ed.doc().primitives[0] {
        Primitive::Component(c) => assert_eq!(c.name, "Other.C01"),
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
