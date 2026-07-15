//! L1: ADR-0005 history matrix via `POST /api/wizard/navigate` + `GET /api/wizard/state`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
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
    let root = unique_path("wyvern-wizard-hist-ui");
    let pages = root.join("pages");
    std::fs::create_dir_all(&pages).expect("mkdir pages");
    for name in ["a.html", "b.html", "c.html", "d.html", "shared.html"] {
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
    let url_file = unique_path("wyvern-wizard-hist-url");
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

/// Full ADR-0005 matrix: push, back-without-truncation, restore, branch truncate,
/// and same-html/different-id truncate — asserted via `GET /api/wizard/state`.
#[test]
fn wizard_history_adr0005_matrix_via_state() {
    let (handle, base, url_file, ui_root) = start_wizard();
    let client = reqwest::blocking::Client::new();
    let navigate = format!("{base}/api/wizard/navigate");

    // --- forward_push_advances_cursor: A→B→C ---
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": {"a": 1},
            "next": page("b", "pages/b.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b");
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["data"], serde_json::json!({"a": 1}));

    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": {"b": "cached"},
            "next": page("c", "pages/c.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "c");
    assert_eq!(state["stack"].as_array().unwrap().len(), 2);
    assert_eq!(
        state["stack"][1]["data"],
        serde_json::json!({"b": "cached"})
    );

    // --- back_moves_cursor_without_truncation ---
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b");
    assert_eq!(state["page_data"], serde_json::json!({"b": "cached"}));
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);

    post_navigate(
        &client,
        &navigate,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "a");
    assert_eq!(state["stack"].as_array().unwrap().len(), 0);

    // --- forward_same_page_restores_data (non-meaningful payloads) ---
    for restore_payload in [
        serde_json::Value::Null,
        serde_json::json!({}),
        serde_json::json!([]),
        serde_json::json!(""),
    ] {
        // Ensure we are on A with forward B/C intact before each restore attempt.
        // After the first restore we may be on B — walk back to A first.
        let state = wait_for_wizard_state(&client, &base);
        if state["page"]["id"] == "b" {
            post_navigate(
                &client,
                &navigate,
                serde_json::json!({ "action": "back", "data": {} }),
            );
        }
        post_navigate(
            &client,
            &navigate,
            serde_json::json!({
                "action": "next",
                "data": restore_payload,
                "next": page("b", "pages/b.html")
            }),
        );
        let state = wait_for_wizard_state(&client, &base);
        assert_eq!(state["page"]["id"], "b");
        assert_eq!(
            state["page_data"],
            serde_json::json!({"b": "cached"}),
            "non-meaningful restore must keep cached B data"
        );
        assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    }

    // Walk back to A for branch tests.
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({ "action": "back", "data": {} }),
    );

    // --- forward_different_page_truncates ---
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": {"a": 9},
            "next": page("d", "pages/d.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "d");
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["data"], serde_json::json!({"a": 9}));

    // Rebuild A→B(shared)→C so same-html/different-id can truncate.
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
            "data": {"a": 1},
            "next": page("b", "pages/shared.html")
        }),
    );
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": {"b": "shared-cached"},
            "next": page("c", "pages/c.html")
        }),
    );
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({ "action": "back", "data": {} }),
    );

    // --- forward_same_html_different_id_truncates ---
    post_navigate(
        &client,
        &navigate,
        serde_json::json!({
            "action": "next",
            "data": {"a": 2},
            "next": page("b-alt", "pages/shared.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "b-alt");
    assert_eq!(state["page"]["html"], "pages/shared.html");
    assert_eq!(state["page_data"], serde_json::json!({}));
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["data"], serde_json::json!({"a": 2}));

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}
