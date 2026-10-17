//! Browser uploads: File > Open… on the web, through web-sys only. There is no file system to pick
//! a path from, so the picker is a hidden `<input type="file">`; the chosen file (or one dropped on
//! the page) is read in the background and handed to the app on its next frame, which opens it
//! with OPEN `{data, name}` ([`CadApp::open_bytes`]).

use std::cell::Cell;
use std::rc::Rc;

use cadcraft_ui_egui::CadApp;
use wasm_bindgen::JsCast as _;
use web_sys::js_sys;

/// The hidden file input (one at a time; a cancelled pick leaves it until the next one).
const INPUT_ID: &str = "cadcraft_open_input";
/// What the picker offers: the formats OPEN reads.
const ACCEPT: &str = ".dxf,.dwg";

/// A file read in the browser (name, contents), or why it couldn't be read.
type Upload = Result<(String, Vec<u8>), String>;

/// Files read in the background, waiting for the app's next frame.
#[derive(Clone)]
pub struct Inbox {
    files: Rc<Cell<Vec<Upload>>>,
    /// Dropped images read for the AI panel (name, bytes), waiting for the next frame.
    images: Rc<std::cell::RefCell<Vec<(String, Vec<u8>)>>>,
    ctx: egui::Context,
}

impl Inbox {
    pub fn new(ctx: &egui::Context) -> Self {
        Inbox { files: Rc::default(), images: Rc::default(), ctx: ctx.clone() }
    }

    /// Show the browser's file picker and return at once. A chosen file arrives through
    /// [`Self::drain`]; cancelling the picker does nothing.
    pub fn pick(&self) {
        if let Err(e) = self.show_picker() {
            self.push(Err(format!("Open: the browser's file picker is not available ({e})")));
        }
    }

    fn show_picker(&self) -> Result<(), String> {
        let document = web_sys::window().and_then(|w| w.document()).ok_or("no browser document")?;
        let body = document.body().ok_or("the page has no body")?;
        if let Some(old) = document.get_element_by_id(INPUT_ID) {
            old.remove();
        }
        let input = document
            .create_element("input")
            .map_err(js_error)?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| "could not create a file input".to_string())?;
        input.set_type("file");
        input.set_accept(ACCEPT);
        input.set_id(INPUT_ID);
        input.set_attribute("style", "display:none").map_err(js_error)?;
        let (inbox, chosen) = (self.clone(), input.clone());
        // Called once, when a file was chosen (browsers don't fire `change` on cancel).
        let on_change = wasm_bindgen::closure::Closure::once_into_js(move || {
            if let Some(file) = chosen.files().and_then(|f| f.get(0)) {
                inbox.read(file);
            }
            chosen.remove();
        });
        input.set_onchange(Some(on_change.unchecked_ref()));
        body.append_child(&input).map_err(js_error)?;
        input.click();
        Ok(())
    }

    /// Read the files dropped on the page this frame (eframe hands them over unread on the web).
    /// Image files don't open as drawings: they are read here and returned for the AI panel.
    pub fn take_dropped(&self, ctx: &egui::Context) {
        for f in ctx.input(|i| i.raw.dropped_files.clone()) {
            let image = is_image(&f.path().to_string_lossy());
            if image {
                if let Some(file) = f.web_file() {
                    self.read_image(file.clone());
                }
            } else if let Some(file) = f.web_file() {
                self.read(file.clone());
            }
        }
    }

    /// Read an image in the background; it arrives through [`Self::drain_images`].
    fn read_image(&self, file: web_sys::File) {
        let name = file.name();
        let inbox = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await {
                Ok(buf) => {
                    let bytes = js_sys::Uint8Array::new(&buf).to_vec();
                    let mut imgs = inbox.images.borrow_mut();
                    if imgs.len() < 8 {
                        imgs.push((name, bytes));
                    }
                    drop(imgs);
                    inbox.ctx.request_repaint();
                }
                Err(e) => inbox.push(Err(format!("AI: could not read image: {}", js_error(e)))),
            }
        });
    }

    /// Images read since the last frame, for the AI panel's attachments.
    pub fn drain_images(&self) -> Vec<(String, Vec<u8>)> {
        { let imgs = std::mem::take(&mut *self.images.borrow_mut()); imgs }
    }

    /// Read `file` in the background; it arrives through [`Self::drain`]. A file too large to
    /// open is refused before it is read.
    fn read(&self, file: web_sys::File) {
        let name = file.name();
        // `File.size` is a whole number of bytes; anything else counts as too large.
        let size = file.size();
        let len = if size.is_finite() && size >= 0.0 { size as u64 } else { u64::MAX };
        if let Some(e) = CadApp::open_size_error(&name, len) {
            return self.push(Err(e));
        }
        let inbox = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let r = match wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await {
                Ok(buf) => Ok((name, js_sys::Uint8Array::new(&buf).to_vec())),
                Err(e) => Err(format!("Open: could not read {name}: {}", js_error(e))),
            };
            inbox.push(r);
        });
    }

    /// A file read elsewhere (sent by the page embedding ZiCAD): opened on the next frame like
    /// a picked file, or refused when too large to open.
    pub fn receive(&self, name: String, bytes: Vec<u8>) {
        match CadApp::open_size_error(&name, bytes.len() as u64) {
            Some(e) => self.push(Err(e)),
            None => self.push(Ok((name, bytes))),
        }
    }

    /// A file from elsewhere refused before it was read (`error` says why).
    pub fn refuse(&self, error: String) {
        self.push(Err(error));
    }

    fn push(&self, upload: Upload) {
        let mut files = self.files.take();
        files.push(upload);
        self.files.set(files);
        self.ctx.request_repaint();
    }

    /// Open what arrived since the last frame (or report why it couldn't be read).
    pub fn drain(&self, app: &mut CadApp) {
        for upload in self.files.take() {
            match upload {
                Ok((name, bytes)) => app.open_bytes(&name, &bytes),
                Err(e) => app.open_failed(e),
            }
        }
    }
}

fn js_error(e: wasm_bindgen::JsValue) -> String {
    e.as_string().unwrap_or_else(|| format!("{e:?}"))
}


/// A file name that looks like an image: drops of these go to the AI panel, not to OPEN.
pub fn is_image(name: &str) -> bool {
    matches!(
        name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp")
    )
}
