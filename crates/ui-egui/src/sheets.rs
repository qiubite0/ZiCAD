//! Batch plot and sheet index: both work over the frames that `frames::detect` finds.
//! Batch plot exports each checked frame as one page of a single PDF (io::pdf_items with a
//! window per frame); the sheet index draws a 序号/图名/图号/版别 table next to the frames
//! through ordinary engine commands, so it lands in the undo history like hand drawing.

use crate::frames::{self, Frame};
use crate::{CadApp, tl};
use cadcraft_doc::{Drawing, Space};
use cadcraft_io::pdf::{PdfOptions, pdf_items};
use serde_json::json;

/// Dialog state, shared by both dialogs: one detection, two consumers.
#[derive(Default)]
pub struct Sheets {
    pub batch_open: bool,
    pub index_open: bool,
    pub frames: Vec<Frame>,
    pub pick: Vec<bool>,
    /// Whether the cached frames still describe the active drawing.
    stale: bool,
}

impl Sheets {
    fn detect(&mut self, doc: Option<&Drawing>) {
        self.frames = doc.map(frames::detect).unwrap_or_default();
        self.pick = vec![true; self.frames.len()];
        self.stale = false;
    }

    fn ensure_fresh(&mut self, doc: Option<&Drawing>) {
        if self.stale {
            self.detect(doc);
        }
    }

    fn checked(&self) -> Vec<&Frame> {
        self.frames.iter().enumerate().filter(|(i, _)| *self.pick.get(*i).unwrap_or(&false)).map(|(_, f)| f).collect()
    }
}

/// Called from `dialogs::show` for "batchplot" / "sheetindex".
pub fn dialogs(app: &mut CadApp, ctx: &egui::Context, which: &str) {
    let doc = app.session.docs.get(app.session.active).map(|d| d.doc.clone());
    let opening = match which {
        "batchplot" => !app.sheets.batch_open,
        "sheetindex" => !app.sheets.index_open,
        _ => false,
    };
    if opening {
        app.sheets.detect(doc.as_deref());
    }
    app.sheets.ensure_fresh(doc.as_deref());
    match which {
        "batchplot" => {
            app.sheets.batch_open = true;
            batch_dialog(app, ctx);
        }
        "sheetindex" => {
            app.sheets.index_open = true;
            index_dialog(app, ctx);
        }
        _ => {}
    }
}

fn window_title(which: &str) -> egui::RichText {
    match which {
        "batchplot" => egui::RichText::new(tl!("Batch Plot...")),
        _ => egui::RichText::new(tl!("Sheet Index...")),
    }
}

fn batch_dialog(app: &mut CadApp, ctx: &egui::Context) {
    let mut open = app.sheets.batch_open;
    egui::Window::new(window_title("batchplot")).open(&mut open).default_width(640.0).resizable(true).show(ctx, |ui| {
        frame_table(app, ui);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.add_enabled(!app.sheets.checked().is_empty(), egui::Button::new(tl!("Export PDF"))).clicked() {
                let picked: Vec<Frame> = app.sheets.checked().iter().map(|f| (*f).clone()).collect();
                let name = format!("batch-plot-{}.pdf", now_stamp());
                match export_pdf(app, &picked, &name) {
                    Ok(()) => app.session.echo(format!("已导出 {name}（{} 页）", picked.len())),
                    Err(e) => app.session.echo(format!("导出失败: {e}")),
                }
            }
            ui.label(tl!("Each checked frame becomes one page of a single PDF."));
        });
    });
    app.sheets.batch_open = open;
    if !open && app.ui.dialog.as_deref() == Some("batchplot") {
        app.ui.dialog = None;
    }
}

fn index_dialog(app: &mut CadApp, ctx: &egui::Context) {
    let mut open = app.sheets.index_open;
    egui::Window::new(window_title("sheetindex")).open(&mut open).default_width(640.0).resizable(true).show(ctx, |ui| {
        frame_table(app, ui);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.add_enabled(!app.sheets.checked().is_empty(), egui::Button::new(tl!("Generate Index"))).clicked() {
                let picked: Vec<Frame> = app.sheets.checked().iter().map(|f| (*f).clone()).collect();
                let (ok, failed) = generate_index(app, &picked);
                app.session.echo(format!("图纸目录：{ok} 条成功，{failed} 条失败（目录绘制在图框右侧）"));
            }
            ui.label(tl!("Draws a sheet index table to the right of the frames."));
        });
    });
    app.sheets.index_open = open;
    if !open && app.ui.dialog.as_deref() == Some("sheetindex") {
        app.ui.dialog = None;
    }
}

/// The checkbox table: 序号 / 图名 / 图号 / 版别 / size.
fn frame_table(app: &mut CadApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button(tl!("Refresh")).clicked() {
            let doc = app.session.docs.get(app.session.active).map(|d| d.doc.clone());
            app.sheets.detect(doc.as_deref());
        }
        if ui.button(tl!("Select All")).clicked() {
            app.sheets.pick.iter_mut().for_each(|p| *p = true);
        }
        if ui.button(tl!("Uncheck All")).clicked() {
            app.sheets.pick.iter_mut().for_each(|p| *p = false);
        }
    });
    if app.sheets.frames.is_empty() {
        ui.add_space(4.0);
        ui.label(tl!("No drawing frames found. Draw closed rectangles of sheet size first."));
        return;
    }
    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
        egui::Grid::new("sheets_grid").num_columns(6).striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
            ui.strong(tl!("Sel."));
            ui.strong(tl!("Idx"));
            ui.strong(tl!("Sheet Name"));
            ui.strong(tl!("Sheet No."));
            ui.strong(tl!("Rev."));
            ui.strong(tl!("Size"));
            ui.end_row();
            for (i, f) in app.sheets.frames.clone().into_iter().enumerate() {
                let mut picked = app.sheets.pick[i];
                if ui.checkbox(&mut picked, "").changed() {
                    app.sheets.pick[i] = picked;
                }
                ui.label(format!("{}", i + 1));
                ui.label(truncate(&f.name, 28));
                ui.label(truncate(&f.number, 18));
                ui.label(truncate(&f.revision, 10));
                ui.label(format!("{:.0}×{:.0}", f.window.max.x - f.window.min.x, f.window.max.y - f.window.min.y));
                ui.end_row();
            }
        });
    });
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let t: String = s.chars().take(n).collect();
        format!("{t}…")
    }
}

fn now_stamp() -> String {
    let secs = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{secs:x}")
}

/// One multi-page PDF with a page per frame window.
fn export_pdf(app: &mut CadApp, picked: &[Frame], name: &str) -> Result<(), String> {
    let doc = active_drawing(app)?;
    let items: Vec<(Space, PdfOptions)> =
        picked.iter().map(|f| (Space::Model, PdfOptions { window: Some(f.window), title: f.name.clone(), ..Default::default() })).collect();
    let global = PdfOptions { title: name.trim_end_matches(".pdf").to_string(), ..Default::default() };
    let bytes = pdf_items(&doc, &items, &global).map_err(|e| e.to_string())?;
    deliver(app, name, "application/pdf", &bytes)
}

/// Hand bytes to the host: the web build installed a deliver hook (download/bridge); on the
/// desktop write beside the drawing and echo where it went.
fn deliver(app: &mut CadApp, name: &str, mime: &str, bytes: &[u8]) -> Result<(), String> {
    if let Some(hook) = cadcraft_engine::cmd::file::deliver() {
        return hook(name, mime, bytes);
    }
    let dir = app
        .session
        .docs
        .get(app.session.active)
        .and_then(|d| d.path.clone())
        .and_then(|p| std::path::Path::new(&p).parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let path = dir.join(name);
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    app.session.echo(format!("已写出 {}", path.display()));
    Ok(())
}

fn active_drawing(app: &mut CadApp) -> Result<std::sync::Arc<Drawing>, String> {
    app.session.docs.get(app.session.active).map(|d| d.doc.clone()).ok_or_else(|| "没有打开的图纸".into())
}

/// Draw the index table (title row + 序号/图名/图号/版别 columns) to the right of the frames,
/// through ordinary commands so every line lands in the undo history.
fn generate_index(app: &mut CadApp, picked: &[Frame]) -> (usize, usize) {
    const H: f64 = 1000.0; // row height
    const PAD: f64 = 200.0;
    const W: [f64; 4] = [1500.0, 12_000.0, 5000.0, 2500.0];
    let total_w: f64 = W.iter().sum();
    let right = picked.iter().map(|f| f.window.max.x).fold(f64::NEG_INFINITY, f64::max);
    if !right.is_finite() {
        return (0, 0);
    }
    let top = picked.iter().map(|f| f.window.max.y).fold(f64::MIN, f64::max);
    let x0 = right + 8000.0;
    let title_h = 1600.0;
    let rows = picked.len();
    let table_h = title_h + H * (rows as f64 + 1.0);
    let y0 = top;
    let y1 = y0 - table_h;

    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut cmd = |app: &mut CadApp, what: &str, params: serde_json::Value| {
        if app.session.execute(what, &params).is_ok() {
            ok += 1;
        } else {
            failed += 1;
        }
    };

    // Border + title underline.
    cmd(app, "pline", json!({"vertices": [[x0, y0], [x0 + total_w, y0], [x0 + total_w, y1], [x0, y1]], "closed": true}));
    let title_y = y0 - title_h;
    cmd(app, "line", json!({"points": [[x0, title_y], [x0 + total_w, title_y]]}));
    // Header row line and column lines.
    let header_y = title_y - H;
    cmd(app, "line", json!({"points": [[x0, header_y], [x0 + total_w, header_y]]}));
    let mut xs = vec![x0];
    let mut acc = x0;
    for w in W {
        acc += w;
        xs.push(acc);
    }
    for &x in xs.iter().skip(1).take(W.len() - 1) {
        cmd(app, "line", json!({"points": [[x, y0], [x, y1]]}));
    }
    for i in 1..rows {
        let y = header_y - H * i as f64;
        cmd(app, "line", json!({"points": [[x0, y], [x0 + total_w, y]]}));
    }
    // Title + headers.
    cmd(app, "text", json!({"at": [x0 + total_w / 2.0 - 2000.0, y0 - title_h + 300.0], "height": 900.0, "text": "图纸目录"}));
    let headers = ["序号", "图名", "图号", "版别"];
    for (i, h) in headers.iter().enumerate() {
        cmd(app, "text", json!({"at": [xs[i] + PAD, header_y - H + 250.0], "height": 500.0, "text": h}));
    }
    // Rows.
    for (i, f) in picked.iter().enumerate() {
        let y = header_y - H * (i as f64 + 1.0) + 250.0;
        let cells = [
            format!("{}", i + 1),
            f.name.clone(),
            if f.number.is_empty() { "—".into() } else { f.number.clone() },
            if f.revision.is_empty() { "—".into() } else { f.revision.clone() },
        ];
        for (c, s) in cells.iter().enumerate() {
            let s = if c == 1 { truncate(s, 30) } else { truncate(s, 12) };
            cmd(app, "text", json!({"at": [xs[c] + PAD, y], "height": 450.0, "text": s}));
        }
    }
    (ok, failed)
}
