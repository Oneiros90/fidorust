use fidocad_core::library::{UserLibraryTarget, PROJECT_STEM};
use fidocad_core::parse::{builtin_libraries, parse_document, parse_document_with_project_library};
use fidocad_core::serialize::serialize_document;
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
        .create_component_from_selection(UserLibraryTarget::Project, "Nuovo componente")
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
    ed.create_component_from_selection(UserLibraryTarget::Project, "Box")
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
    ed.create_component_from_selection(UserLibraryTarget::Project, "Box")
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
    ed.create_component_from_selection(UserLibraryTarget::Local, "Box")
        .unwrap();
    let text = serialize_document(ed.doc(), Some(ed.libs()));
    assert!(text.contains("local.C01"), "{text}");
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
        .create_component_from_selection(UserLibraryTarget::Project, "A")
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
        .create_component_from_selection(UserLibraryTarget::Project, "Moved")
        .unwrap();
    let dest = ed.move_component(PROJECT_STEM, &key, "local").unwrap();
    assert!(ed.libs().project().unwrap().find(&key).is_none());
    assert!(ed.libs().local().unwrap().find(&dest).is_some());
    match &ed.doc().primitives[0] {
        Primitive::Component(c) => assert_eq!(c.name, format!("local.{dest}")),
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
        .create_component_from_selection(UserLibraryTarget::Project, "Edit me")
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
        .create_component_from_selection(UserLibraryTarget::Project, "Named")
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
    ed.create_component_from_selection(UserLibraryTarget::Project, "Mix")
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
    ed.create_component_from_selection(UserLibraryTarget::Project, "A")
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
        .create_component_from_selection(UserLibraryTarget::Project, "A")
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
