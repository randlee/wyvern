//! L1: REQ-0024 bootstrap via HTTP — multi-step navigate + `GET /api/wizard/state`.
//!
//! Asserts `config`, `page`, `page_data`, and prior-only `stack` (no IPC).

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
    let root = unique_path("wyvern-wizard-stack-ui");
    let pages = root.join("pages");
    std::fs::create_dir_all(&pages).expect("mkdir pages");
    for name in ["a.html", "b.html", "c.html"] {
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

fn opaque_data(label: &str) -> serde_json::Value {
    serde_json::json!({
        "label": label,
        "nested": {"keep_me": true, "n": 7},
        "weird.key": "dot-name",
        "unicode_キー": "値"
    })
}

fn wizard_command() -> Command {
    Command::Wizard(WizardCommand {
        page: page("a", "pages/a.html"),
        config: serde_json::json!({
            "theme": "dark",
            "opaque_cfg": {"x": 1, "nested": {"y": "z"}}
        }),
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

fn start_wizard(client: &reqwest::blocking::Client) -> (DialogHandle, String, PathBuf, PathBuf) {
    let ui_root = write_ui_root();
    let url_file = unique_path("wyvern-wizard-stack-url");
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
    let _ = wait_for_wizard_state(client, &base);
    (handle, base, url_file, ui_root)
}

fn post_navigate(
    client: &reqwest::blocking::Client,
    navigate: &str,
    body: serde_json::Value,
) -> serde_json::Value {
    let resp = client.post(navigate).json(&body).send().expect("navigate");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "navigate body={body}"
    );
    resp.json().expect("navigate json")
}

fn assert_bootstrap_fields(state: &serde_json::Value) {
    assert_eq!(state["type"], "wizard");
    assert!(state.get("config").is_some(), "missing config");
    assert!(state.get("page").is_some(), "missing page");
    assert!(state.get("page_data").is_some(), "missing page_data");
    assert!(state.get("stack").is_some(), "missing stack");
    assert!(state["stack"].is_array(), "stack must be array");
}

/// Multi-step HTTP flow: state GET asserts config/page/page_data/prior-only stack.
#[test]
fn wizard_stack_http_state_asserts_bootstrap_fields() {
    let client = http_client();
    let (handle, base, url_file, ui_root) = start_wizard(&client);
    let navigate = format!("{base}/api/wizard/navigate");
    let a_data = opaque_data("a");
    let b_data = opaque_data("b");

    // First page — empty prior stack (REQ-0024).
    let state = wait_for_wizard_state(&client, &base);
    assert_bootstrap_fields(&state);
    assert_eq!(state["config"]["theme"], "dark");
    assert_eq!(state["config"]["opaque_cfg"]["nested"]["y"], "z");
    assert_eq!(state["page"]["id"], "a");
    assert_eq!(state["page"]["html"], "pages/a.html");
    assert_eq!(state["page_data"], serde_json::json!({}));
    assert_eq!(state["stack"], serde_json::json!([]));
    assert_eq!(state["width"], 640);
    assert_eq!(state["height"], 480);

    // A→B: prior stack has A only; current is B via page + page_data.
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": a_data,
            "next": page("b", "pages/b.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_bootstrap_fields(&state);
    assert_eq!(state["config"]["theme"], "dark");
    assert_eq!(state["page"]["id"], "b");
    assert_eq!(state["page_data"], serde_json::json!({}));
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["page"]["id"], "a");
    assert_eq!(state["stack"][0]["data"], a_data);
    assert_eq!(state["stack"][0]["data"]["unicode_キー"], "値");
    assert_ne!(state["stack"][0]["page"]["id"], state["page"]["id"]);

    // B→C
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": b_data,
            "next": page("c", "pages/c.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_bootstrap_fields(&state);
    assert_eq!(state["page"]["id"], "c");
    assert_eq!(state["stack"].as_array().unwrap().len(), 2);
    assert_eq!(state["stack"][1]["data"], b_data);
    assert_eq!(state["stack"][1]["data"]["weird.key"], "dot-name");

    // Back restores page_data; stack shrinks to prior-only.
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_bootstrap_fields(&state);
    assert_eq!(state["config"]["opaque_cfg"]["x"], 1);
    assert_eq!(state["page"]["id"], "b");
    assert_eq!(state["page_data"], b_data);
    assert_eq!(state["page_data"]["nested"]["keep_me"], true);
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["data"], a_data);

    // Forward restore keeps cached B page_data via GET state (not IPC).
    // Empty payload: destination restore only; current is whole-blob-replaced.
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": {},
            "next": page("b", "pages/b.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_bootstrap_fields(&state);
    assert_eq!(state["page"]["id"], "b");
    assert_eq!(state["page_data"], b_data);
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["page"]["id"], "a");
    assert_eq!(state["stack"][0]["data"], serde_json::json!({}));

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}
