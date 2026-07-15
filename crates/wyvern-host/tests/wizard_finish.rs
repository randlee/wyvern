//! L1: `POST /api/wizard/finish` — stack validation, cancel, dismissed.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{begin, DialogHandle, HostOptions, ViewerMode};
use wyvern_schema::{Command, WizardCommand, WizardPageDescriptor};

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
    let root = unique_path("wyvern-wizard-finish-ui");
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
        id: id.into(),
        title: id.into(),
        html: html.into(),
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

fn host_options(ui_root: PathBuf, url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    }
}

fn wait_for_url_file(path: &std::path::Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for dialog URL file {}", path.display());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_wizard_state(client: &reqwest::blocking::Client, base: &str) -> serde_json::Value {
    let url = format!("{base}/api/wizard/state");
    let start = std::time::Instant::now();
    loop {
        match client.get(&url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.json().expect("state json");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET /api/wizard/state at {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn start_wizard() -> (
    DialogHandle,
    String,
    PathBuf,
    PathBuf,
    reqwest::blocking::Client,
) {
    let ui_root = write_ui_root();
    let url_file = unique_path("wyvern-wizard-finish-url");
    let handle = begin(
        wizard_command(),
        host_options(ui_root.clone(), url_file.clone()),
    )
    .expect("begin");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("wizard path");
    let client = reqwest::blocking::Client::new();
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

#[test]
fn wizard_finish_accepts_matching_stack() {
    let (handle, base, url_file, ui_root, client) = start_wizard();
    navigate_to_b(&client, &base);

    let finish_data = serde_json::json!({"b": 2});
    let stack = serde_json::json!([
        { "page": page("a", "pages/a.html"), "data": {"a": 1} },
        { "page": page("b", "pages/b.html"), "data": finish_data }
    ]);

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "finish",
            "data": finish_data,
            "stack": stack
        }))
        .send()
        .expect("finish");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().expect("json");
    assert_eq!(body["button"], "finish");
    assert_eq!(body["data"], finish_data);
    assert_eq!(body["stack"].as_array().unwrap().len(), 2);

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "finish");
            assert_eq!(w.data, finish_data);
            assert_eq!(w.stack.len(), 2);
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_finish_stack_mismatch_returns_400() {
    let (handle, base, url_file, ui_root, client) = start_wizard();
    navigate_to_b(&client, &base);

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "finish",
            "data": {"b": 2},
            "stack": []
        }))
        .send()
        .expect("finish");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().expect("json");
    assert!(
        body["message"]
            .as_str()
            .unwrap_or("")
            .contains("stack does not match"),
        "message={}",
        body["message"]
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_finish_cancel_clears_stack_and_data() {
    let (handle, base, url_file, ui_root, client) = start_wizard();
    navigate_to_b(&client, &base);

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "cancel",
            "data": {"ignored": true},
            "stack": [
                { "page": page("a", "pages/a.html"), "data": {"a": 1} }
            ]
        }))
        .send()
        .expect("cancel");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().expect("json");
    assert_eq!(body["button"], "cancel");
    assert_eq!(body["data"], serde_json::json!({}));
    assert_eq!(body["stack"], serde_json::json!([]));

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "cancel");
            assert!(w.stack.is_empty());
            assert_eq!(w.data, serde_json::json!({}));
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_finish_dismissed_full_stack_empty_data() {
    let (handle, base, url_file, ui_root, client) = start_wizard();
    navigate_to_b(&client, &base);

    let stack = serde_json::json!([
        { "page": page("a", "pages/a.html"), "data": {"a": 1} },
        { "page": page("b", "pages/b.html"), "data": {"b": 2} }
    ]);

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "data": {"b": 2},
            "stack": stack
        }))
        .send()
        .expect("dismissed");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().expect("json");
    assert_eq!(body["button"], "dismissed");
    assert_eq!(body["data"], serde_json::json!({}));
    assert_eq!(body["stack"].as_array().unwrap().len(), 2);

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "dismissed");
            assert_eq!(w.data, serde_json::json!({}));
            assert_eq!(w.stack.len(), 2);
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}
