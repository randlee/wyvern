//! L1: wizard dismissed finish + REQ-0097 host fallback (d.8).

mod support;
use support::http::{http_client, wait_for_url_file, wait_for_wizard_state};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use wyvern_host::{begin, DialogHandle, HostOptions, ViewerMode};
use wyvern_schema::{
    Command, WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn write_ui_root() -> PathBuf {
    let root = unique_path("wyvern-wizard-dismiss-ui");
    let pages = root.join("pages");
    std::fs::create_dir_all(&pages).expect("mkdir pages");
    for name in ["a.html", "b.html"] {
        std::fs::write(
            pages.join(name),
            format!("<!DOCTYPE html><title>{name}</title><h1>{name}</h1>"),
        )
        .expect("write page");
    }
    root
}

fn page(id: &str, html: &str) -> WizardPageDescriptor {
    WizardPageDescriptor {
        id: WizardPageId::new(id),
        title: WizardPageTitle::new(id),
        html: WizardPageHtml::new(html),
        layout: None,
    }
}

fn wizard_command() -> Command {
    Command::Wizard(WizardCommand {
        page: page("a", "pages/a.html"),
        config: serde_json::json!({}),
        width: None,
        height: None,
    })
}

fn host_options(ui_root: PathBuf, url_file: PathBuf, timeout: Duration) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: timeout,
        mock_picker: None,
    }
}

fn start_wizard(
    timeout: Duration,
) -> (
    DialogHandle,
    String,
    PathBuf,
    PathBuf,
    reqwest::blocking::Client,
) {
    let ui_root = write_ui_root();
    let url_file = unique_path("wyvern-wizard-dismiss-url");
    let handle = begin(
        wizard_command(),
        host_options(ui_root.clone(), url_file.clone(), timeout),
    )
    .expect("begin");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("wizard path");
    let client = http_client();
    let _ = wait_for_wizard_state(&client, &base);
    (handle, base, url_file, ui_root, client)
}

fn navigate_to_b(client: &reqwest::blocking::Client, base: &str) {
    let resp = client
        .post(format!("{base}/api/wizard/navigate"))
        .json(&serde_json::json!({
            "action": "next",
            "data": {"a": 1},
            "next": page("b", "pages/b.html")
        }))
        .send()
        .expect("next");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
}

/// Viewer algorithm: GET state → full visited stack → POST finish dismissed.
#[test]
fn wizard_dismiss_post_finish_includes_current_page() {
    let (handle, base, url_file, ui_root, client) = start_wizard(Duration::from_secs(30));
    navigate_to_b(&client, &base);

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b");
    let prior = state["stack"].as_array().expect("prior stack");
    assert_eq!(prior.len(), 1);

    let mut full_stack = prior.clone();
    full_stack.push(serde_json::json!({
        "page": state["page"],
        "data": state["page_data"],
    }));

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "data": state["page_data"],
            "stack": full_stack
        }))
        .send()
        .expect("dismissed finish");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().expect("json");
    assert_eq!(body["button"], "dismissed");
    assert_eq!(body["data"], serde_json::json!({}));
    let stack = body["stack"].as_array().expect("stack");
    assert_eq!(stack.len(), 2);
    assert_eq!(stack[0]["page"]["id"], "a");
    assert_eq!(stack[0]["data"], serde_json::json!({"a": 1}));
    assert_eq!(stack[1]["page"]["id"], "b");
    assert_eq!(stack[1]["data"], state["page_data"]);

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "dismissed");
            assert_eq!(w.data, serde_json::json!({}));
            assert_eq!(w.stack.len(), 2);
            assert_eq!(w.stack[0].page.id.as_str(), "a");
            assert_eq!(w.stack[1].page.id.as_str(), "b");
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_dismiss_stack_mismatch_returns_400() {
    let (handle, base, url_file, ui_root, client) = start_wizard(Duration::from_secs(30));
    navigate_to_b(&client, &base);

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "data": {},
            "stack": []
        }))
        .send()
        .expect("dismissed mismatch");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

/// CLI fallback: viewer exit without POST → full visited stack (not prior-only).
#[test]
fn wizard_dismiss_viewer_exit_uses_full_visited_stack() {
    let (handle, base, url_file, ui_root, client) = start_wizard(Duration::from_secs(30));
    navigate_to_b(&client, &base);
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b");

    let result = handle
        .viewer_exited_without_result()
        .expect("dismissed fallback");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "dismissed");
            assert_eq!(w.data, serde_json::json!({}));
            assert_eq!(
                w.stack.len(),
                2,
                "must include current page, not prior-only"
            );
            assert_eq!(w.stack[0].page.id.as_str(), "a");
            assert_eq!(w.stack[0].data, serde_json::json!({"a": 1}));
            assert_eq!(w.stack[1].page.id.as_str(), "b");
            assert_eq!(w.stack[1].data, state["page_data"]);
        }
        other => panic!("expected wizard dismissed, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

/// Session timeout fallback also derives the full visited stack.
///
/// RSH-003 / FTQ-001: use a 10s session_timeout so setup+navigate finish before
/// idle expiry, then sleep the remaining idle budget before `await_result`.
#[test]
fn wizard_dismiss_session_timeout_uses_full_visited_stack() {
    let session_timeout = Duration::from_secs(10);
    let started = std::time::Instant::now();
    let (handle, base, url_file, ui_root, client) = start_wizard(session_timeout);
    navigate_to_b(&client, &base);
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b");

    let remaining = session_timeout.saturating_sub(started.elapsed());
    std::thread::sleep(remaining + Duration::from_millis(500));

    let result = handle.await_result().expect("timeout dismissed");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "dismissed");
            assert_eq!(w.data, serde_json::json!({}));
            assert_eq!(w.stack.len(), 2);
            assert_eq!(w.stack[0].data, serde_json::json!({"a": 1}));
            assert_eq!(w.stack[1].page.id.as_str(), "b");
            assert_eq!(w.stack[1].data, state["page_data"]);
        }
        other => panic!("expected wizard dismissed, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}
