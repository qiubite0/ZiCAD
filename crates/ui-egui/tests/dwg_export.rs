use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services};
use serde_json::json;

#[test]
fn export_dwg_preserves_entities_and_does_not_mark_the_drawing_saved() {
    let mut app = CadApp::new(Session::new(), Services::default());
    app.run("circle", json!({"center":[10,20],"radius":5})).unwrap();
    app.ui.save_version = "2018".into();
    let path = std::env::temp_dir().join(format!("cadcraft-export-test-{}.dwg", std::process::id()));
    app.run("ui.export.dwg", json!({"path":path})).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes.get(..6), Some(b"AC1032".as_slice()));
    let drawing = cadcraft_io::read(&bytes, "roundtrip.dwg").unwrap();
    assert_eq!(drawing.model.len(), 1);
    assert!(app.session.state().unwrap().is_dirty());
    assert!(app.session.state().unwrap().path.is_none());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn exporting_without_a_drawing_reports_an_error() {
    let mut app = CadApp::new(Session::empty(), Services::default());
    assert!(app.run("ui.export.dwg", json!({"path":"unused.dwg"})).is_err());
}
