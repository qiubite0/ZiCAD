//! The AI assistant: a floating chat window in the "CAD AI Studio" style. Platform presets
//! (or a local Ollama) fill the endpoint and model; models can be pulled live from the
//! platform's `/models`; the model answers with ZiCAD command lines in a ```cadcraft fence
//! and the panel runs them through the same command-line path a user types into.

use crate::CadApp;
use egui::ScrollArea;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Where models live, editable: an OpenAI-compatible /chat/completions endpoint.
const DEFAULT_BASE_URL: &str = "https://api.deepseek.com/v1";
const DEFAULT_MODEL: &str = "deepseek-flash";
/// DeepSeek is the default platform (index into [`PLATFORMS`]).
const DEFAULT_PLATFORM: usize = 2;
/// Images attached to one message at most (token sanity).
const MAX_IMAGES: usize = 4;

/// A provider with an official OpenAI-compatible endpoint, a starter model and its catalogue.
pub struct Preset {
    pub name: &'static str,
    pub base_url: &'static str,
    pub model: &'static str,
    pub models: &'static [&'static str],
    /// Local endpoints need the user to allow cross-origin calls (Ollama: OLLAMA_ORIGINS).
    pub local: bool,
}

pub const PLATFORMS: &[Preset] = &[
    Preset { name: "本地 Ollama", base_url: "http://127.0.0.1:11434/v1", model: "qwen3:14b", models: &["qwen3:14b"], local: true },
    Preset {
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        model: "gpt-6.1-sol",
        models: &["gpt-6.1-sol", "gpt-6-astra", "gpt-6-sol", "gpt-6-luna", "gpt-5.6-sol", "gpt-5-mini", "gpt-4.1", "gpt-4.1-mini"],
        local: false,
    },
    Preset {
        name: "DeepSeek 深度求索",
        base_url: "https://api.deepseek.com/v1",
        model: "deepseek-flash",
        models: &["deepseek-flash", "deepseek-v4-pro"],
        local: false,
    },
    Preset {
        name: "智谱 GLM",
        base_url: "https://open.bigmodel.cn/api/paas/v4",
        model: "glm-5.3",
        models: &["glm-5.3", "glm-5.2", "glm-5v-turbo", "glm-4-flash"],
        local: false,
    },
    Preset {
        name: "月之暗面 Kimi",
        base_url: "https://api.moonshot.cn/v1",
        model: "kimi-k3",
        models: &["kimi-k3", "kimi-latest", "kimi-k2-0905-preview"],
        local: false,
    },
    Preset {
        name: "阿里通义千问",
        base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
        model: "qwen3-max",
        models: &["qwen3-max", "qwen-max", "qwen-plus", "qwen-turbo"],
        local: false,
    },
    Preset {
        name: "字节豆包",
        base_url: "https://ark.cn-beijing.volces.com/api/v3",
        model: "doubao-seed-1-6-250615",
        models: &["doubao-seed-1-6-250615"],
        local: false,
    },
    Preset {
        name: "硅基流动",
        base_url: "https://api.siliconflow.cn/v1",
        model: "deepseek-ai/DeepSeek-V3.1",
        models: &["zai-org/GLM-5.3", "deepseek-ai/DeepSeek-V4-Flash", "Qwen/Qwen3-VL-32B-Instruct"],
        local: false,
    },
    Preset {
        name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        model: "openai/gpt-6.1-sol",
        models: &[
            "openai/gpt-6.1-sol", "openai/gpt-6-astra", "openai/gpt-6-luna", "openai/gpt-5.6-sol",
            "anthropic/claude-sonnet-5.5", "anthropic/claude-opus-5.5", "anthropic/claude-haiku-5.5", "anthropic/claude-fable-5.1",
            "google/gemini-4-argon", "google/gemini-3.8-flash", "x-ai/grok-4.7", "zai-org/glm-5.3",
        ],
        local: false,
    },
];
/// Beyond the presets: endpoint and model fully manual.
const CUSTOM: usize = PLATFORMS.len();

#[derive(Clone, Debug, PartialEq)]
pub struct Llm {
    /// Index into [`PLATFORMS`], or [`CUSTOM`].
    pub platform: usize,
    pub base_url: String,
    pub model: String,
    /// One key per platform name: switching platforms keeps the others.
    pub keys: BTreeMap<String, String>,
    /// Total completion tokens reported by the platforms, for the panel badge.
    pub tokens: u64,
}

impl Default for Llm {
    fn default() -> Self {
        Llm { platform: DEFAULT_PLATFORM, base_url: DEFAULT_BASE_URL.into(), model: DEFAULT_MODEL.into(), keys: BTreeMap::new(), tokens: 0 }
    }
}

impl Llm {
    pub fn platform_name(&self) -> &str {
        PLATFORMS.get(self.platform).map(|p| p.name).unwrap_or("自定义")
    }

    fn set_platform(&mut self, i: usize) {
        self.platform = i;
        if let Some(p) = PLATFORMS.get(i) {
            self.base_url = p.base_url.into();
            self.model = p.model.into();
        }
    }

    pub fn current_key(&self) -> &str {
        self.keys.get(self.platform_name()).map(String::as_str).unwrap_or("")
    }

    fn to_prefs(&self) -> Value {
        json!({
            "platform": self.platform,
            "baseUrl": self.base_url,
            "model": self.model,
            "keys": self.keys,
            "tokens": self.tokens,
        })
    }

    fn from_prefs(v: &Value) -> Option<Self> {
        let a = v.get("ai")?;
        let mut keys = BTreeMap::new();
        if let Some(map) = a.get("keys").and_then(Value::as_object) {
            for (k, val) in map {
                if let Some(s) = val.as_str().filter(|s| !s.is_empty()) {
                    // Older builds saved empty entries just by rendering the key field.
                    keys.insert(k.clone(), s.to_string());
                }
            }
        }
        let platform = a.get("platform").and_then(Value::as_u64).unwrap_or(DEFAULT_PLATFORM as u64) as usize;
        Some(Llm {
            platform,
            base_url: a.get("baseUrl").and_then(Value::as_str).unwrap_or(DEFAULT_BASE_URL).into(),
            model: a.get("model").and_then(Value::as_str).unwrap_or(DEFAULT_MODEL).into(),
            keys,
            tokens: a.get("tokens").and_then(Value::as_u64).unwrap_or(0),
        })
    }
}

pub fn restore_prefs(app: &mut CadApp, v: &Value) {
    if let Some(cfg) = Llm::from_prefs(v) {
        app.ai.cfg = cfg;
    }
    let a = match v.get("ai") {
        Some(a) => a,
        None => return,
    };
    if let Some(t) = a.get("temperature").and_then(Value::as_f64) {
        app.ai.temperature = t as f32;
    }
    if let Some(n) = a.get("maxTurns").and_then(Value::as_u64) {
        app.ai.max_turns = n as u32;
    }
    if let Some(b) = a.get("autoExec").and_then(Value::as_bool) {
        app.ai.auto_exec = b;
    }
    if let Some(s) = a.get("extraPrompt").and_then(Value::as_str) {
        app.ai.extra_prompt = s.into();
    }
}

pub fn to_prefs(app: &CadApp, prefs: &mut Value) {
    // Only once the user changed something: a default config would break the documented prefs.
    if !app.ai.is_stock() {
        prefs["ai"] = json!({
            "platform": app.ai.cfg.platform,
            "baseUrl": app.ai.cfg.base_url,
            "model": app.ai.cfg.model,
            "keys": app.ai.cfg.keys,
            "tokens": app.ai.cfg.tokens,
            "temperature": ((app.ai.temperature * 100.0).round() / 100.0),
            "maxTurns": app.ai.max_turns,
            "autoExec": app.ai.auto_exec,
            "extraPrompt": app.ai.extra_prompt,
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Msg {
    /// "user" | "assistant" | "error"
    pub role: &'static str,
    pub content: String,
    /// Attached images as data URLs (multimodal requests only).
    pub images: Vec<String>,
}

/// Chat window state. Replies come back on a channel: the HTTP callback runs off the UI path.
pub struct Ai {
    pub open: bool,
    pub settings_open: bool,
    pub cfg: Llm,
    /// The settings dialog edits a draft; 保存 writes it back, 取消 drops it.
    pub draft: Llm,
    pub temperature: f32,
    pub max_turns: u32,
    /// false: command fences wait for a click before running.
    pub auto_exec: bool,
    pub extra_prompt: String,
    /// Models pulled live from a platform's `/models`, keyed by endpoint.
    pub fetched: BTreeMap<String, Vec<String>>,
    /// Last connection test / pull outcome, shown in the settings dialog.
    pub conn_status: Option<(bool, String)>,
    pub msgs: Vec<Msg>,
    pub draft_text: String,
    pub busy: bool,
    pub started_ms: Option<f64>,
    pub last_cost_s: Option<f64>,
    /// Tool-result turns already chained for the current task.
    auto: u32,
    /// Command lines waiting for the user's 执行 click (`auto_exec` off).
    awaiting: Option<Vec<String>>,
    /// Images (data URLs) dragged in and waiting to be sent with the next message.
    pub pending: Vec<String>,
    /// Show the drag-images hint (toggled by 传截图).
    pub attach_hint: bool,
    /// Background `/models` fetch for the settings dialog.
    fetch_rx: Option<std::sync::mpsc::Receiver<(String, Result<Vec<String>, String>)>>,
    rx: Option<std::sync::mpsc::Receiver<Result<(String, Option<u64>), String>>>,
}

impl Default for Ai {
    fn default() -> Self {
        Ai {
            open: false,
            settings_open: false,
            cfg: Llm::default(),
            draft: Llm::default(),
            temperature: 0.3,
            max_turns: 20,
            auto_exec: true,
            extra_prompt: String::new(),
            fetched: BTreeMap::new(),
            conn_status: None,
            msgs: Vec::new(),
            draft_text: String::new(),
            busy: false,
            started_ms: None,
            last_cost_s: None,
            auto: 0,
            awaiting: None,
            pending: Vec::new(),
            attach_hint: false,
            fetch_rx: None,
            rx: None,
        }
    }
}

impl Ai {
    fn is_stock(&self) -> bool {
        self.cfg == Llm::default() && self.temperature == 0.3 && self.max_turns == 20 && self.auto_exec && self.extra_prompt.is_empty()
    }

    /// Attach one dropped image (data URL), up to [`MAX_IMAGES`] per message.
    pub fn attach_image(&mut self, name: &str, bytes: &[u8]) {
        if self.pending.len() >= MAX_IMAGES {
            return;
        }
        let mime = match name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
            Some("png") => "image/png",
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("webp") => "image/webp",
            Some("gif") => "image/gif",
            Some("bmp") => "image/bmp",
            _ => return,
        };
        self.pending.push(format!("data:{mime};base64,{}", base64(bytes)));
    }

    fn new_session(&mut self) {
        self.msgs.clear();
        self.awaiting = None;
        self.auto = 0;
    }
}

/// The floating chat window. Call once per frame, before the central panel so it docks right.
pub fn panel(app: &mut CadApp, ui: &mut egui::Ui) {
    if !app.ai.open {
        return;
    }
    poll(app);
    if app.ai.busy || app.ai.fetch_rx.is_some() {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
    }
    // Desktop drops carry their bytes right away; the web host reads images async and calls
    // [`Ai::attach_image`] itself.
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dropped: Vec<(String, Vec<u8>)> = ui.ctx().input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter(|f| is_image(&f.path().to_string_lossy()))
                .filter_map(|f| f.bytes().ok().map(|b| (f.path().to_string_lossy().into_owned(), b)))
                .collect()
        });
        for (name, bytes) in dropped {
            app.ai.attach_image(&name, &bytes);
        }
    }
    // On the web the host app reads images async (upload.rs) and calls attach_image itself.

    let mut open = app.ai.open;
    let screen = ui.input(|i| i.viewport_rect());
    // About two thirds of the viewport, capped for small windows.
    let def_w = (screen.width() * 0.62).min(480.0).max(320.0);
    let def_h = (screen.height() * 0.45).min(380.0).max(260.0);
    let max_w = (screen.width() - 24.0).max(320.0);
    let max_h = (screen.height() - 150.0).max(240.0);
    let win = egui::Window::new(crate::tl!("AI Assistant · Chat"))
        .default_size([def_w, def_h])
        .max_size([max_w, max_h])
        .default_pos(egui::pos2(screen.center().x - def_w / 2.0, screen.top() + 60.0))
        .movable(true)
        .resizable(true);
    win.show(ui.ctx(), |ui| {
        status_bar(app, &mut open, ui);
        ui.separator();
        // Reserve the composer before allocating the scroll area: otherwise its
        // infinite default height stretches the window and pushes Send to the bottom.
        let composer_height = 120.0;
        let history_height = (ui.available_height() - composer_height).max(40.0);
        ScrollArea::vertical().max_height(history_height).stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
            if app.ai.msgs.is_empty() {
                ui.small(crate::tl!("CAD 服务已就绪。描述要画或要改的内容，我会直接操作当前图纸。"));
                ui.add_space(4.0);
            }
            for m in &app.ai.msgs {
                let who = match m.role {
                    "user" => crate::tl!("你:"),
                    "error" => crate::tl!("错误:"),
                    _ => crate::tl!("AI:"),
                };
                let color = match m.role {
                    "user" => crate::theme::Tokens::get().accent,
                    "error" => egui::Color32::from_rgb(0xe0, 0x60, 0x50),
                    _ => crate::theme::Tokens::get().text,
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(who).strong().color(color));
                    ui.label(egui::RichText::new(&m.content).color(color));
                    if !m.images.is_empty() {
                        ui.label(format!("📎{}", m.images.len()));
                    }
                });
                ui.add_space(4.0);
            }
            if let Some(n) = app.ai.awaiting.as_ref().map(Vec::len) {
                ui.horizontal(|ui| {
                    ui.small(format!("{n} 条命令等待确认。"));
                    if ui.small_button(crate::tl!("执行")).clicked() {
                        let lines = app.ai.awaiting.take().unwrap_or_default();
                        let (ok, failed) = run_lines(app, &lines);
                        app.ai.auto += 1;
                        let report = format!("命令执行结果：{ok} 条成功，{failed} 条失败。请继续或总结。");
                        request(app, Some(report));
                    }
                });
            }
            if app.ai.busy {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new().size(14.0));
                    ui.label(crate::tl!("Thinking..."));
                });
            }
        });
        ui.separator();
        if let Some(secs) = app.ai.last_cost_s {
            ui.small(format!("合计 {secs:.1}s"));
        }
        input_ui(app, ui);
    });
    app.ai.open = open;
    settings_dialog(app, ui.ctx());
}

/// The top strip: endpoint · model · key state · tokens · 新会话 · 工具入口.
fn status_bar(app: &mut CadApp, open: &mut bool, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        if ui.small_button(crate::tl!("Close")).clicked() {
            *open = false;
        }
        if ui.small_button(crate::tl!("Settings")).clicked() {
            app.ai.draft = app.ai.cfg.clone();
            app.ai.conn_status = None;
            app.ai.settings_open = true;
        }
        if ui.small_button(crate::tl!("New Session")).clicked() {
            app.ai.new_session();
        }
        if ui.small_button(crate::tl!("Batch Plot...")).clicked() {
            app.ui.dialog = Some("batchplot".into());
        }
        if ui.small_button(crate::tl!("Sheet Index...")).clicked() {
            app.ui.dialog = Some("sheetindex".into());
        }
        ui.separator();
        let key_state = if app.ai.cfg.current_key().is_empty() { "未配 Key" } else { "Key 已配" };
        ui.small(format!("{} · {} · {key_state}", app.ai.cfg.platform_name(), short_model(&app.ai.cfg.model),));
        tokens_badge(ui, app.ai.cfg.tokens);
    });
}

/// A round badge with the accumulated token count.
fn tokens_badge(ui: &mut egui::Ui, tokens: u64) {
    let t = crate::theme::Tokens::get();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 30.0), egui::Sense::hover());
    let c = rect.center();
    ui.painter().circle_filled(c, 14.0, t.control);
    ui.painter().circle_stroke(c, 14.0, egui::Stroke::new(1.5, t.accent));
    ui.painter().text(c, egui::Align2::CENTER_CENTER, short_count(tokens), egui::FontId::proportional(11.0), t.text);
}

/// The "AI 设置" dialog, edited against a draft configuration.
fn settings_dialog(app: &mut CadApp, ctx: &egui::Context) {
    if !app.ai.settings_open {
        return;
    }
    let mut open = true;
    egui::Window::new(crate::tl!("AI Settings")).open(&mut open).default_size([540.0, 480.0]).resizable(true).show(ctx, |ui| {
        let t = crate::theme::Tokens::get();
        egui::Frame::NONE.fill(t.panel).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
            ui.strong(crate::tl!("AI Platform Access"));
            ui.add_space(4.0);
            let before = app.ai.draft.platform;
            egui::Grid::new("ai_cfg").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label(crate::tl!("AI Platform"));
                egui::ComboBox::from_id_salt("ai_platform").selected_text(app.ai.draft.platform_name()).width(200.0).show_ui(ui, |ui| {
                    for (i, p) in PLATFORMS.iter().enumerate() {
                        ui.selectable_value(&mut app.ai.draft.platform, i, p.name);
                    }
                    ui.selectable_value(&mut app.ai.draft.platform, CUSTOM, crate::tl!("Custom"));
                });
                ui.end_row();
                ui.label(crate::tl!("Endpoint"));
                ui.add(egui::TextEdit::singleline(&mut app.ai.draft.base_url).hint_text("https://…/v1").desired_width(f32::INFINITY));
                ui.end_row();
                ui.label(crate::tl!("API Key"));
                let name = app.ai.draft.platform_name().to_string();
                let key = app.ai.draft.keys.entry(name).or_default();
                ui.add(
                    egui::TextEdit::singleline(key)
                        .password(true)
                        .hint_text(crate::tl!("This platform's API Key; empty keeps the saved one"))
                        .desired_width(f32::INFINITY),
                );
                ui.end_row();
                ui.label(crate::tl!("Model"));
                ui.horizontal(|ui| {
                    let platform_models =
                        PLATFORMS.get(app.ai.draft.platform).map(|p| p.models.iter().map(|m| m.to_string()).collect::<Vec<_>>()).unwrap_or_default();
                    let mut listed = platform_models.clone();
                    if let Some(fetched) = app.ai.fetched.get(&app.ai.draft.base_url) {
                        listed.extend(fetched.iter().cloned());
                    }
                    listed.sort();
                    listed.dedup();
                    let current = app.ai.draft.model.clone();
                    egui::ComboBox::from_id_salt("ai_model").width(170.0).selected_text(short_model(&current)).show_ui(ui, |ui| {
                        if !listed.iter().any(|m| *m == current) && !current.is_empty() {
                            let c = current.clone();
                            ui.selectable_value(&mut app.ai.draft.model, c.clone(), c);
                        }
                        for m in &listed {
                            ui.selectable_value(&mut app.ai.draft.model, m.clone(), m.clone());
                        }
                    });
                    if ui.small_button(crate::tl!("拉取列表")).clicked() {
                        fetch_models(app);
                    }
                });
                ui.end_row();
            });
            if app.ai.draft.platform != before && app.ai.draft.platform != CUSTOM {
                let i = app.ai.draft.platform;
                app.ai.draft.set_platform(i);
                app.ai.fetched.remove(&app.ai.draft.base_url);
            }
            if let Some((ok, msg)) = &app.ai.conn_status {
                ui.small(egui::RichText::new(msg).color(if *ok {
                    egui::Color32::from_rgb(0x50, 0xc0, 0x60)
                } else {
                    egui::Color32::from_rgb(0xe0, 0x60, 0x50)
                }));
            }
            ui.small("GPT 选择 OpenAI；Claude / Gemini / Grok 选择 OpenRouter，并使用 OpenRouter API Key。");
            ui.add_space(4.0);
            ui.strong(crate::tl!("高级"));
            ui.horizontal(|ui| {
                ui.label(crate::tl!("温度"));
                ui.add(egui::DragValue::new(&mut app.ai.temperature).speed(0.01).range(0.0..=2.0));
                ui.label(crate::tl!("最大工具轮数"));
                ui.add(egui::DragValue::new(&mut app.ai.max_turns).speed(0.2).range(1..=50));
            });
            ui.checkbox(&mut app.ai.auto_exec, crate::tl!("修改模型无需确认（自动执行）"));
            ui.add(
                egui::TextEdit::multiline(&mut app.ai.extra_prompt)
                    .hint_text(crate::tl!("附加提示词（可选，追加到系统提示后面）"))
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            );
            ui.small(crate::tl!(
                "接口地址、API Key、模型必须是同一家的；换厂商要连 Key 和模型一起换，否则会报 401。看图需求请选视觉模型（如 glm-5v-turbo）。"
            ));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button(crate::tl!("测试连接")).clicked() {
                    test_connection(app);
                }
                if ui.button(crate::tl!("Save")).clicked() {
                    app.ai.cfg = app.ai.draft.clone();
                    app.ai.settings_open = false;
                }
                if ui.button(crate::tl!("Cancel")).clicked() {
                    app.ai.settings_open = false;
                }
            });
        });
    });
    if !open {
        app.ai.settings_open = false;
    }
}

/// GET `{endpoint}/models` with the draft key: fills the catalogue and reports reachability.
fn fetch_models(app: &mut CadApp) {
    let (tx, rx) = std::sync::mpsc::channel();
    app.ai.fetch_rx = Some(rx);
    app.ai.conn_status = None;
    let url = ai_url(&app.ai.draft.base_url, "models");
    let mut req = ehttp::Request::get(url);
    req.headers.insert("Authorization", format!("Bearer {}", app.ai.draft.current_key()));
    let endpoint = app.ai.draft.base_url.clone();
    ehttp::fetch(req, move |result| {
        let outcome = result.map(|resp| String::from_utf8_lossy(&resp.bytes).into_owned()).and_then(|text| {
            let v: Value = serde_json::from_str(&text).map_err(|e| format!("响应解析失败: {e}"))?;
            let ids: Vec<String> = v
                .get("data")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(|m| m.get("id").and_then(Value::as_str)).map(str::to_string).collect())
                .unwrap_or_default();
            if ids.is_empty() { Err(format!("平台没有返回模型列表: {text}")) } else { Ok(ids) }
        });
        let _ = tx.send((endpoint, outcome.map_err(|e| e.to_string())));
    });
}

fn test_connection(app: &mut CadApp) {
    fetch_models(app);
}

/// Apply a pulled model list to the settings draft (called when the fetch lands).
fn apply_fetched(app: &mut CadApp, endpoint: String, ids: Vec<String>) {
    let n = ids.len();
    app.ai.fetched.insert(endpoint, ids);
    app.ai.conn_status = Some((true, format!("已连接，拉取到 {n} 个模型")));
}

/// Drain the settings' background fetches (models list / connection test).
fn poll_fetch(app: &mut CadApp) {
    let Some(rx) = &app.ai.fetch_rx else { return };
    let Ok((endpoint, outcome)) = rx.try_recv() else { return };
    app.ai.fetch_rx = None;
    match outcome {
        Ok(ids) => apply_fetched(app, endpoint, ids),
        Err(e) => app.ai.conn_status = Some((false, e)),
    }
}

fn input_ui(app: &mut CadApp, ui: &mut egui::Ui) {
    ui.add(
        egui::TextEdit::multiline(&mut app.ai.draft_text)
            .hint_text(crate::tl!("描述你要画的图或要改的内容…（Ctrl+Enter 发送，可拖入截图）"))
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    // Buttons on their own row: sitting in the window's bottom-right corner put them under
    // the resize handle's hot zone, swallowing clicks.
    ui.horizontal(|ui| {
        if ui.add_enabled(!app.ai.busy && !app.ai.draft_text.trim().is_empty(), egui::Button::new(crate::tl!("Send"))).clicked() {
            ask(app);
        }
        if ui.add_enabled(app.ai.busy, egui::Button::new(crate::tl!("Stop"))).clicked() {
            app.ai.rx = None;
            app.ai.busy = false;
            app.ai.last_cost_s = app.ai.started_ms.map(|s| (now_ms() - s) / 1000.0);
            app.ai.msgs.push(Msg { images: Vec::new(), role: "error", content: crate::tl!("Stopped.").to_string() });
        }
        if ui.button(crate::tl!("传截图")).clicked() {
            app.ai.attach_hint = !app.ai.attach_hint;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.small(crate::tl!("Ctrl+Enter 发送"));
        });
    });
    if app.ai.attach_hint {
        ui.small(crate::tl!("把图片文件直接拖到窗口里即可附加（最多 4 张，发给视觉模型）。"));
    }
    let ctrl_enter = ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl);
    if ctrl_enter && !app.ai.busy && !app.ai.draft_text.trim().is_empty() {
        ask(app);
    }
}

/// One user turn: remember it, then request a completion with the conversation so far.
fn ask(app: &mut CadApp) {
    let content = app.ai.draft_text.trim().to_string();
    if content.is_empty() {
        return;
    }
    if app.ai.cfg.current_key().is_empty() {
        app.ai.msgs.push(Msg { images: Vec::new(), role: "error", content: crate::tl!("No API Key configured for this platform yet.").to_string() });
        return;
    }
    app.ai.draft_text.clear();
    let images = std::mem::take(&mut app.ai.pending);
    app.ai.msgs.push(Msg { images, role: "user", content });
    app.ai.auto = 0;
    request(app, None);
}

fn request(app: &mut CadApp, tool_result: Option<String>) {
    let (tx, rx) = std::sync::mpsc::channel();
    app.ai.rx = Some(rx);
    app.ai.busy = true;
    app.ai.started_ms = Some(now_ms());

    let mut body_msgs = vec![json!({"role": "system", "content": system_prompt(app)})];
    for m in &app.ai.msgs {
        let role = if m.role == "assistant" { "assistant" } else { "user" };
        body_msgs.push(json!({"role": role, "content": message_content(m)}));
    }
    if let Some(tool) = tool_result {
        body_msgs.push(json!({"role": "user", "content": tool}));
    }

    let url = ai_url(&app.ai.cfg.base_url, "chat/completions");
    // Built from scratch: ehttp's `post` ships a default text/plain Content-Type and
    // `Headers::insert` appends rather than replaces, so the platform would see both.
    let mut req =
        ehttp::Request::new(ehttp::Method::POST, url, ehttp::Headers::new(&[("Content-Type", "application/json"), ("Accept", "application/json")]))
            .with_body(
                json!({
                    "model": app.ai.cfg.model,
                    "messages": body_msgs,
                    "temperature": ((app.ai.temperature * 100.0).round() / 100.0),
                    "stream": false
                })
                .to_string()
                .into_bytes(),
            );
    req.headers.insert("Authorization", format!("Bearer {}", app.ai.cfg.current_key()));
    ehttp::fetch(req, move |result| {
        let reply = result.map(|resp| String::from_utf8_lossy(&resp.bytes).into_owned()).and_then(|text| {
            let v: Value = serde_json::from_str(&text).map_err(|e| format!("响应解析失败: {e}"))?;
            if let Some(err) = v.get("error").and_then(|e| e.get("message")).and_then(Value::as_str) {
                return Err(format!("平台返回错误: {err}"));
            }
            let tokens = v.get("usage").and_then(|u| u.get("total_tokens")).and_then(Value::as_u64);
            let content = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("响应里没有回答: {text}"))?;
            Ok((content, tokens))
        });
        let _ = tx.send(reply.map_err(|e| e.to_string()));
    });
}

/// Collect the assistant's reply and run any command fences, feeding results back.
fn poll(app: &mut CadApp) {
    poll_fetch(app);
    let Some(rx) = &app.ai.rx else { return };
    let Ok(reply) = rx.try_recv() else { return };
    app.ai.rx = None;
    app.ai.busy = false;
    if let Some(started) = app.ai.started_ms.take() {
        app.ai.last_cost_s = Some((now_ms() - started) / 1000.0);
    }
    match reply {
        Err(e) => app.ai.msgs.push(Msg { images: Vec::new(), role: "error", content: e }),
        Ok((content, tokens)) => {
            if let Some(n) = tokens {
                app.ai.cfg.tokens += n;
            }
            let (lines, rest) = commands_in(&content);
            if !rest.trim().is_empty() {
                app.ai.msgs.push(Msg { images: Vec::new(), role: "assistant", content: rest.trim().to_string() });
            }
            if lines.is_empty() {
                return;
            }
            if app.ai.auto_exec {
                run_and_report(app, lines);
            } else {
                app.ai.awaiting = Some(lines);
            }
        }
    }
}

/// Run a fence and chain a tool-result turn (within [`Ai::max_turns`]).
fn run_and_report(app: &mut CadApp, lines: Vec<String>) {
    let (ok, failed) = run_lines(app, &lines);
    app.ai.auto += 1;
    if app.ai.auto <= app.ai.max_turns {
        let mut report = format!("命令执行结果：{ok} 条成功，{failed} 条失败。");
        for l in &lines {
            report.push_str(&format!("\n- {l}"));
        }
        report.push_str("\n请根据结果继续修图，或向用户总结。没有更多要执行的命令时，直接用文字总结，不要再输出代码块。");
        app.ai.msgs.push(Msg { images: Vec::new(), role: "user", content: format!("〔执行结果〕{ok} 条成功，{failed} 条失败") });
        request(app, Some(report));
    } else {
        app.ai.msgs.push(Msg {
            images: Vec::new(),
            role: "error",
            content: crate::tl!("Stopped after several auto-continues; press Send to continue.").to_string(),
        });
    }
}

/// Run each `cmd {json}` (or plain command) line through the command line, like typing.
fn run_lines(app: &mut CadApp, lines: &[String]) -> (usize, usize) {
    let (mut ok, mut failed) = (0usize, 0usize);
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if app.cmdline(line).is_none() {
            ok += 1;
        } else {
            failed += 1;
        }
    }
    (ok, failed)
}

/// Pull ```cadcraft (or bare ```) fences out of a reply; the rest is prose for the user.
fn commands_in(reply: &str) -> (Vec<String>, String) {
    let mut lines = Vec::new();
    let mut prose = String::new();
    let mut inside = false;
    for line in reply.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            inside = !inside;
            continue;
        }
        if inside {
            lines.push(line.to_string());
        } else {
            prose.push_str(line);
            prose.push('\n');
        }
    }
    (lines, prose)
}

fn now_ms() -> f64 {
    // std::time::SystemTime::now panics on wasm32-unknown-unknown. web-time
    // uses the browser clock there and the standard clock on native platforms.
    web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0)
}

fn system_prompt(app: &CadApp) -> String {
    let (title, count, extents, layers) = app
        .session
        .docs
        .get(app.session.active)
        .map(|d| {
            let ext = d.doc.extents(&cadcraft_doc::Space::Model);
            let layers: Vec<String> = d.doc.layers.iter().map(|l| l.name.clone()).take(20).collect();
            (d.title.clone(), d.doc.entity_count(), ext, layers)
        })
        .unwrap_or_default();
    let extra = if app.ai.extra_prompt.trim().is_empty() { String::new() } else { format!("\n用户附加要求：{}", app.ai.extra_prompt.trim()) };
    format!(
        "你是内嵌在 ZiCAD 网页版 CAD 里的绘图助手，直接操作用户当前图纸。\n\
        当前图纸：{title}，{count} 个实体，范围 {extents:?}，图层：{layers:?}。坐标单位与图纸一致（一般 mm），Y 轴向上。\n\
        要修改图纸时，把命令写进 ```cadcraft 代码块，每行一条，格式为「命令名 {{JSON参数}}」（这是 ZiCAD 的程序化命令行格式），例如：\n\
        ```cadcraft\n\
        line {{\"points\":[[0,0],[10000,0],[10000,5000]]}}\n\
        pline {{\"vertices\":[[0,0],[5000,0],[5000,3000],[0,3000]],\"closed\":true}}\n\
        circle {{\"center\":[2500,1500],\"radius\":600}}\n\
        text {{\"at\":[2500,3200],\"height\":400,\"text\":\"卧室\"}}\n\
        ```\n\
        已知命令参数：line{{points:[[x,y],…]}}、pline{{vertices:[[x,y],…],closed:bool}}、circle{{center:[x,y],radius:r}}、text{{at:[x,y],height:h,text:s}}。\n\
        规则：先算好坐标再写命令；文字高度通常 300–500；给用户的解释写在代码块外；\n\
        不确定的命令宁可不用；收到〔执行结果〕后按失败信息修正坐标或参数。{extra}"
    )
}

/// Plain text, or an OpenAI-style multipart array when the message carries images.
fn message_content(m: &Msg) -> Value {
    if m.images.is_empty() {
        json!(m.content)
    } else {
        let mut parts = vec![json!({"type": "text", "text": m.content})];
        for url in &m.images {
            parts.push(json!({"type": "image_url", "image_url": {"url": url}}));
        }
        json!(parts)
    }
}

/// A file name that looks like an image (drops are split: images go to the AI panel).
fn is_image(name: &str) -> bool {
    matches!(name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref(), Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"))
}

/// Standard base64 (no line breaks) — enough for data URLs without pulling a crate.
fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

fn short_model(m: &str) -> String {
    let width = m.chars().count();
    if width <= 18 {
        m.to_string()
    } else {
        let tail: String = m.chars().skip(width - 18).collect();
        format!("…{tail}")
    }
}

fn short_count(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}k", n as f64 / 1_000.0),
        _ => format!("{:.1}M", n as f64 / 1_000_000.0),
    }
}

/// Percent-encode for the dev-proxy query string.
fn urlencoded(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The chat/models endpoint. The web build has no way around CORS: requests go through the
/// same-origin dev proxy (`dev-proxy.py`, /llmi?u=<endpoint>); desktop calls directly.
fn ai_url(base_url: &str, path: &str) -> String {
    let endpoint = format!("{}/{path}", base_url.trim_end_matches('/'));
    #[cfg(target_arch = "wasm32")]
    {
        // 127.0.0.1 Ollama stays direct (same-origin policy still applies, but localhost-to-
        // localhost from a localhost page is same-site; the proxy would also loop).
        if base_url.starts_with("http://127.0.0.1") || base_url.starts_with("http://localhost") {
            return endpoint;
        }
        format!("/llmi?u={}", urlencoded(&endpoint))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        endpoint
    }
}
