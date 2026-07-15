//! L1: `POST /api/wizard/navigate` — next/back, cursor=0 → 400, cancel → 400.

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
    let root = unique_path("wyvern-wizard-nav-ui");
    let pages = root.join("pages");
    std::fs::create_dir_all(&pages).expect("mkdir pages");
    for name in ["a.html", "b.html", "c.html", "d.html"] {
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
        config: serde_json::json!({"theme": "dark"}),
        width: Some(640),
        height: Some(480),
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

fn start_wizard() -> (DialogHandle, String, PathBuf, PathBuf) {
    let ui_root = write_ui_root();
    let url_file = unique_path("wyvern-wizard-nav-url");
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
    (handle, base, url_file, ui_root)
}

#[test]
fn wizard_navigate_next_back_and_branch() {
    let (handle, base, url_file, ui_root) = start_wizard();
    let client = reqwest::blocking::Client::new();
    let navigate = format!("{base}/api/wizard/navigate");

    let resp = client
        .post(&navigate)
        .json(&serde_json::json!({
            "action": "next",
            "data": {"a": 1},
            "next": page("b", "pages/b.html")
        }))
        .send()
        .expect("next b");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().expect("json");
    assert_eq!(body["ok"], true);
    assert!(
        body["url"]
            .as_str()
            .unwrap()
            .ends_with("/wizard/pages/b.html"),
        "url={}",
        body["url"]
    );

    let resp = client
        .post(&navigate)
        .json(&serde_json::json!({
            "action": "next",
            "data": {"b": 2},
            "next": page("c", "pages/c.html")
        }))
        .send()
        .expect("next c");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    // Back preserves forward entries.
    let resp = client
        .post(&navigate)
        .json(&serde_json::json!({ "action": "back", "data": {} }))
        .send()
        .expect("back");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b");
    assert_eq!(state["page_data"], serde_json::json!({"b": 2}));
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);

    // Forward same page restores.
    let resp = client
        .post(&navigate)
        .json(&serde_json::json!({
            "action": "next",
            "data": {},
            "next": page("c", "pages/c.html")
        }))
        .send()
        .expect("restore c");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "c");
    assert_eq!(state["stack"].as_array().unwrap().len(), 2);

    // Branch truncates.
    let resp = client
        .post(&navigate)
        .json(&serde_json::json!({ "action": "back", "data": {} }))
        .send()
        .expect("back");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let resp = client
        .post(&navigate)
        .json(&serde_json::json!({ "action": "back", "data": {} }))
        .send()
        .expect("back");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let resp = client
        .post(&navigate)
        .json(&serde_json::json!({
            "action": "next",
            "data": {"a": 9},
            "next": page("d", "pages/d.html")
        }))
        .send()
        .expect("branch d");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "d");
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["data"], serde_json::json!({"a": 9}));

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_navigate_back_at_first_page_returns_400() {
    let (handle, base, url_file, ui_root) = start_wizard();
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{base}/api/wizard/navigate"))
        .json(&serde_json::json!({ "action": "back", "data": {} }))
        .send()
        .expect("back");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().expect("json");
    assert_eq!(body["ok"], false);
    assert!(
        body["message"]
            .as_str()
            .unwrap_or("")
            .contains("first wizard page"),
        "message={}",
        body["message"]
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_navigate_cancel_action_returns_400() {
    let (handle, base, url_file, ui_root) = start_wizard();
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{base}/api/wizard/navigate"))
        .json(&serde_json::json!({ "action": "cancel", "data": {} }))
        .send()
        .expect("cancel");
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_navigate_back_empty_preserves_current_data() {
    let (handle, base, url_file, ui_root) = start_wizard();
    let client = reqwest::blocking::Client::new();
    let navigate = format!("{base}/api/wizard/navigate");

    client
        .post(&navigate)
        .json(&serde_json::json!({
            "action": "next",
            "data": {"a": 1},
            "next": page("b", "pages/b.html")
        }))
        .send()
        .expect("next");

    // Seed B with meaningful data via next then back with empty (preserve).
    client
        .post(&navigate)
        .json(&serde_json::json!({
            "action": "next",
            "data": {"b": "keep"},
            "next": page("c", "pages/c.html")
        }))
        .send()
        .expect("next c");
    client
        .post(&navigate)
        .json(&serde_json::json!({ "action": "back", "data": {} }))
        .send()
        .expect("back to b");

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b");
    assert_eq!(state["page_data"], serde_json::json!({"b": "keep"}));

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}
