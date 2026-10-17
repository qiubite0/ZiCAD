//! Live end-to-end against the real 智谱 endpoint: click Send in the chat window, the model's
//! reply lands in the conversation. Needs the dev proxy on 8771 and ZHIPU_KEY in the
//! environment; skipped otherwise (`cargo test -- --ignored` to run).

use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services};
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn harness() -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.ui.in_window_menu = false;
    app.ai.open = true;
    app.ai.cfg.platform = 3; // 智谱 GLM
    app.ai.cfg.base_url = "https://open.bigmodel.cn/api/paas/v4".into();
    app.ai.cfg.model = "glm-4.6".into();
    let key = std::env::var("ZHIPU_KEY").expect("ZHIPU_KEY not set");
    app.ai.cfg.keys.insert("智谱 GLM".into(), key);
    let mut harness = Harness::builder().with_size(vec2(1280.0, 760.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    harness.run_steps(5);
    harness
}

#[test]
#[ignore = "live: needs dev-proxy on 8771 and ZHIPU_KEY"]
fn live_send_gets_a_glm_reply_in_the_chat() {
    let mut h = harness();
    h.state_mut().ai.draft_text = "回复两个字：收到".into();
    h.run_steps(5);

    h.get_by_label("Send").click();
    // The round trip runs on real threads and a real network.
    for _ in 0..60 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        h.run_steps(10);
        if h
            .state()
            .ai
            .msgs
            .iter()
            .any(|m| m.role == "assistant" || m.role == "error")
        {
            break;
        }
    }

    let msgs = &h.state().ai.msgs;
    assert!(
        msgs.iter().any(|m| m.role == "user" && m.content == "回复两个字：收到"),
        "user message pushed, got {msgs:?}"
    );
    let assistant = msgs.iter().find(|m| m.role == "assistant");
    let error = msgs.iter().find(|m| m.role == "error");
    assert!(
        assistant.is_some(),
        "GLM reply lands in the chat; error instead: {error:?}"
    );
    assert!(
        assistant.unwrap().content.contains("收到"),
        "the reply echoes the request, got {:?}",
        assistant.unwrap().content
    );
}
