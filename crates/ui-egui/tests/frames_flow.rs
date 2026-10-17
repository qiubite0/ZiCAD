//! The batch-plot / sheet-index flow against real drawings in the repo: frames detected with
//! their title-block fields, a plain plan without frames rejected, and a multi-page PDF
//! produced from the frame windows.

use cadcraft_io as io;

const MEP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/cadcraft-web/dev-host/mep-coordination.dxf");
const APARTMENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/cadcraft-web/dev-host/apartment.dxf");

#[test]
fn frames_detected_and_pdf_exported() {
    let bytes = std::fs::read(MEP).expect("sample drawing present in the repo");
    let d = io::read(&bytes, "mep-coordination.dxf").expect("reads");
    let frames = cadcraft_ui_egui::frames::detect(&d);
    assert_eq!(frames.len(), 1, "one title-block frame in the sample, got {:?}", frames.len());

    let f = &frames[0];
    assert!(f.name.contains("管线综合"), "图名 from the tallest text in the title strip: {}", f.name);
    let (w, h) = (f.window.max.x - f.window.min.x, f.window.max.y - f.window.min.y);
    assert!((18_000.0..=22_000.0).contains(&w) && (11_000.0..=15_000.0).contains(&h), "frame ≈20000×13000, got {w}×{h}");

    let page = cadcraft_io::pdf::PdfOptions { window: Some(f.window), title: f.name.clone(), ..Default::default() };
    let global = cadcraft_io::pdf::PdfOptions { title: "batch-plot".into(), ..Default::default() };
    let pdf = cadcraft_io::pdf::pdf_items(&d, &[(cadcraft_doc::Space::Model, page)], &global).expect("pdf builds");
    assert!(pdf.starts_with(b"%PDF"), "PDF magic, got {} bytes", pdf.len());
    assert!(pdf.len() > 10_000, "a plotted sheet is more than a header, got {} bytes", pdf.len());
}

#[test]
fn plain_plan_yields_at_most_building_outline_candidates() {
    // The apartment plan has no title block; its outer wall (12000×8000) still matches the
    // frame-size heuristic — the known false positive the dialogs let users untick.
    let bytes = std::fs::read(APARTMENT).expect("sample drawing present in the repo");
    let d = io::read(&bytes, "apartment.dxf").expect("reads");
    let frames = cadcraft_ui_egui::frames::detect(&d);
    assert_eq!(frames.len(), 1, "the outer wall reads as one frame candidate, got {frames:?}");
    // The title strip has no text here, so the name falls back to the longest text inside.
    assert!(!frames[0].name.is_empty(), "fields still extracted: {}", frames[0].name);
    assert_eq!(frames[0].number, "", "no 图号-like text in the apartment plan");
}

#[test]
fn engineering_dialogs_open_detect_frames_and_reopen_after_closing() {
    use cadcraft_engine::Session;
    use cadcraft_ui_egui::{CadApp, Services};
    use egui_kittest::{Harness, kittest::Queryable};
    let drawing = io::read(&std::fs::read(MEP).unwrap(), "mep-coordination.dxf").unwrap();
    let mut app = CadApp::new(Session::new(), Services::default());
    app.session.state_mut().unwrap().doc = std::sync::Arc::new(drawing);
    app.ui.in_window_menu = false;
    app.ui.start_tab = false;
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 900.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    for (command, action) in
        [("ui.dialog.batchplot", "Export PDF"), ("ui.dialog.sheetindex", "Generate Index"), ("ui.dialog.batchplot", "Export PDF")]
    {
        h.state_mut().run(command, serde_json::Value::Null).unwrap();
        h.run_steps(5);
        assert_eq!(h.state().sheets.frames.len(), 1);
        assert!(h.get_by_label(action).rect().is_positive());
        h.get_by_label("Close window").click();
        h.run_steps(5);
        assert!(h.state().ui.dialog.is_none());
    }
}
