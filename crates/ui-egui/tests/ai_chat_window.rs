//! The AI chat window's send flow: the send button is clickable inside the floating window and
//! pushes the user's message (with attachments) onto the conversation, then fires the request.

use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services};
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn harness() -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.ui.in_window_menu = false;
    app.integrated_titlebar = true;
    app.ai.open = true;
    // A key so ask() doesn't take the "no key" early-exit (that branch is asserted separately).
    app.ai.cfg.keys.insert("DeepSeek 深度求索".into(), "test-key".into());
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
fn the_chat_window_shows_and_send_pushes_the_message() {
    let mut h = harness();
    h.state_mut().ai.draft_text = "你好".into();
    h.run_steps(5);

    let send = h.get_by_label("Send");
    send.click();
    h.run_steps(10);

    let msgs = &h.state_mut().ai.msgs;
    assert_eq!(msgs.len(), 1, "the user message is pushed, got {msgs:?}");
    assert_eq!(msgs[0].role, "user");
    assert_eq!(msgs[0].content, "你好");
    assert!(h.state_mut().ai.draft_text.is_empty(), "the draft is cleared after sending");
}

#[test]
fn sending_without_a_key_reports_instead_of_firing() {
    let mut h = harness();
    h.state_mut().ai.cfg.keys.clear();
    h.state_mut().ai.draft_text = "你好".into();
    h.run_steps(5);

    h.get_by_label("Send").click();
    h.run_steps(5);

    let msgs = &h.state_mut().ai.msgs;
    assert_eq!(msgs.len(), 1, "exactly the local error, got {msgs:?}");
    assert_eq!(msgs[0].role, "error", "no request fires without a key");
    assert!(h.state_mut().ai.draft_text.contains("你好"), "the draft survives so nothing is lost");
}

#[test]
fn the_close_toggle_hides_the_window() {
    let mut h = harness();
    // The window header's close button.
    let close = h.get_by_label("Close");
    close.click();
    h.run_steps(1);
    h.run_steps(5);
    assert!(!h.state_mut().ai.open, "the ✕ closes the chat window");
}

#[test]
fn a_failed_request_surfaces_an_error_in_the_chat() {
    // Point at a port that refuses connections: the full loop (click -> ask -> ehttp ->
    // poll) must come back as a visible error line in the conversation.
    let mut h = harness();
    h.state_mut().ai.cfg.base_url = "http://127.0.0.1:1".into();
    h.state_mut().ai.draft_text = "hi".into();
    h.run_steps(5);

    h.get_by_label("Send").click();
    // The refused-connection failure lands on a real thread; give it wall-clock time.
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(300));
        h.run_steps(10);
        if h.state_mut().ai.msgs.iter().any(|m| m.role == "error") {
            break;
        }
    }

    let msgs = &h.state_mut().ai.msgs;
    assert!(msgs.iter().any(|m| m.role == "user"), "user message pushed, got {msgs:?}");
    assert!(msgs.iter().any(|m| m.role == "error"), "network failure surfaces as a chat error, got {msgs:?}");
}

#[test]
fn send_stays_inside_a_short_viewport_with_long_history() {
    let mut h = harness();
    h.state_mut().run("new", serde_json::json!({})).unwrap();
    h.state_mut().ui.start_tab = false;
    h.set_size(vec2(900.0, 540.0));
    h.state_mut().ai.msgs.push(cadcraft_ui_egui::ai::Msg { role: "assistant", content: "A long reply\n".repeat(100), images: Vec::new() });
    h.state_mut().ai.draft_text = "hello".into();
    h.run_steps(10);
    let send = h.get_by_label("Send");
    assert!(send.rect().bottom() < 510.0, "send must stay above the status bar: {:?}", send.rect());
    send.click();
    h.run_steps(5);
    assert!(h.state().ai.msgs.iter().any(|m| m.role == "user" && m.content == "hello"));
}

#[test]
fn send_works_over_an_open_drawing() {
    let mut h = harness();
    h.state_mut().run("new", serde_json::json!({})).unwrap();
    h.state_mut().ui.start_tab = false;
    h.state_mut().ai.draft_text = "hello".into();
    h.run_steps(10);
    let send = h.get_by_label("Send");
    assert!(send.rect().bottom() < 500.0, "composer stays in the default chat window: {:?}", send.rect());
    send.click();
    h.run_steps(5);
    assert!(h.state().ai.msgs.iter().any(|m| m.role == "user" && m.content == "hello"));
}

#[test]
fn ctrl_enter_sends_while_the_composer_is_focused() {
    let mut h = harness();
    h.state_mut().ai.draft_text = "hello".into();
    h.run_steps(5);
    h.get_by_role(egui::accesskit::Role::MultilineTextInput).click();
    h.run_steps(5);
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::Enter);
    h.run_steps(5);
    assert!(h.state().ai.msgs.iter().any(|m| m.role == "user" && m.content == "hello"));
    assert!(h.state().ai.draft_text.is_empty());
}

#[test]
fn sending_receives_a_reply_and_records_elapsed_time() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let n = stream.read(&mut buffer).unwrap();
            assert!(n > 0, "client must finish its request");
            request.extend_from_slice(&buffer[..n]);
            assert!(request.len() < 256 * 1024);
            if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..end]);
                let length: usize = headers
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().unwrap())
                    })
                    .unwrap();
                if request.len() >= end + 4 + length {
                    break;
                }
            }
        }
        let body = r#"{"choices":[{"message":{"content":"Received"}}],"usage":{"total_tokens":12}}"#;
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
            .unwrap();
    });
    let mut h = harness();
    h.state_mut().ai.cfg.base_url = format!("http://{address}/v1");
    h.state_mut().ai.draft_text = "hello".into();
    h.run_steps(5);
    h.get_by_label("Send").click();
    for _ in 0..50 {
        h.run_steps(5);
        if h.state().ai.msgs.iter().any(|m| m.role == "assistant") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(h.state().ai.msgs.iter().any(|m| m.role == "assistant" && m.content == "Received"));
    assert!(!h.state().ai.busy);
    assert_eq!(h.state().ai.cfg.tokens, 12);
    assert!(h.state().ai.last_cost_s.is_some_and(|s| s.is_finite() && s >= 0.0));
    server.join().unwrap();
}
