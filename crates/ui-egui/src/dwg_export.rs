//! Export the current drawing as DWG. Web conversion uses the local native service.
use crate::CadApp;
use serde_json::{Value, json};

#[derive(Default)]
pub struct State {
    pending: Option<(String, std::sync::mpsc::Receiver<Result<Vec<u8>, String>>)>,
}

pub fn start(app: &mut CadApp, params: &Value) -> Result<Value, String> {
    if app.dwg_export.pending.is_some() {
        return Err("DWG conversion is already running".into());
    }
    let st = app.session.state().map_err(|e| e.to_string())?;
    let suggested = std::path::Path::new(&st.title).file_stem().and_then(|s| s.to_str()).unwrap_or("Drawing");
    let name = params.get("path").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| format!("{suggested}.dwg"));
    let opts = cadcraft_io::SaveOptions { version: cadcraft_io::Version::parse(&app.ui.save_version), binary: false };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = if params.is_null() {
            let Some(path) = app.services.pick_save.as_ref().and_then(|pick| pick(&name)) else { return Ok(Value::Null) };
            path
        } else {
            name
        };
        let bytes = cadcraft_io::write_with(st.drawing(), "Drawing.dwg", &opts).map_err(|e| e.to_string())?;
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        app.set_status(format!("DWG: {path}"));
        Ok(json!({"path": path}))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = cadcraft_io::write_with(st.drawing(), "Drawing.dxf", &opts).map_err(|e| e.to_string())?;
        let (tx, rx) = std::sync::mpsc::channel();
        app.dwg_export.pending = Some((name.clone(), rx));
        let request = ehttp::Request::new(
            ehttp::Method::POST,
            format!("/convert/dxf-to-dwg?version={}", app.ui.save_version),
            ehttp::Headers::new(&[("Content-Type", "application/octet-stream")]),
        )
        .with_body(bytes);
        ehttp::fetch(request, move |response| {
            let result = response.and_then(|r| {
                if !r.ok {
                    let detail = serde_json::from_slice::<Value>(&r.bytes)
                        .ok()
                        .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_string))
                        .unwrap_or_else(|| String::from_utf8_lossy(&r.bytes).chars().take(500).collect());
                    return Err(format!("DWG conversion (HTTP {}): {detail}", r.status));
                }
                if !r.bytes.starts_with(b"AC10") {
                    return Err("The conversion service did not return a DWG file".into());
                }
                Ok(r.bytes)
            });
            let _ = tx.send(result);
        });
        app.set_status(crate::tl!("Converting to DWG...").to_string());
        Ok(json!({"pending": true, "name": name}))
    }
}

pub fn poll(app: &mut CadApp, ctx: &egui::Context) {
    let Some((_, rx)) = &app.dwg_export.pending else { return };
    let result = match rx.try_recv() {
        Ok(result) => result,
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            return;
        }
        Err(_) => Err("DWG conversion stopped before returning a file".into()),
    };
    let Some((name, _)) = app.dwg_export.pending.take() else { return };
    let result = result.and_then(|bytes| {
        let deliver = cadcraft_engine::cmd::file::deliver().ok_or("File download is unavailable")?;
        deliver(&name, "application/acad", &bytes)
    });
    let message = match result {
        Ok(()) => format!("{}: {name}", crate::tl!("DWG download started")),
        Err(e) => format!("DWG: {e}"),
    };
    app.session.echo(message.clone());
    app.set_status(message);
}
