//! L1: workspace-hint fixture — wire shape for `page.layout` + opaque size.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{begin, HostOptions, ViewerMode};
use wyvern_schema::validate;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn workspace_hint_ui_root() -> PathBuf {
    workspace_root().join("examples/wizards/workspace-hint")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn load_workspace_hint_command() -> wyvern_schema::Command {
    let path = workspace_hint_ui_root().join("wizard.json");
    let raw = std::fs::read_to_string(&path).expect("read workspace-hint wizard.json");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse wizard.json");
    validate(&value).expect("validate workspace-hint wizard.json")
}

fn host_options(url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: workspace_hint_ui_root(),
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    }
}

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("http client")
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

fn wait_for_page(client: &reqwest::blocking::Client, url: &str) -> String {
    let start = std::time::Instant::now();
    loop {
        match client.get(url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.text().expect("page text");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

#[test]
fn wizard_workspace_hint_state_echoes_layout_and_estimated_size() {
    let client = http_client();
    let url_file = unique_path("wyvern-wizard-workspace-hint-url");
    let handle = begin(
        load_workspace_hint_command(),
        host_options(url_file.clone()),
    )
    .expect("begin");

    let dialog_url = wait_for_url_file(&url_file);
    assert!(
        dialog_url.contains("/wizard/pages/editor.html"),
        "dialog_url={dialog_url}"
    );
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard path");

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["type"], "wizard");
    assert_eq!(state["page"]["id"], "editor");
    assert_eq!(state["page"]["title"], "Canvas");
    assert_eq!(state["page"]["html"], "pages/editor.html");
    assert_eq!(state["page"]["layout"], "workspace");
    assert_eq!(state["config"]["estimated_size"]["width"], 960);
    assert_eq!(state["config"]["estimated_size"]["height"], 640);
    assert_eq!(state["page_data"], serde_json::json!({}));
    assert_eq!(state["stack"], serde_json::json!([]));

    let html = wait_for_page(&client, &format!("{base}/wizard/pages/editor.html"));
    assert!(
        html.contains("data-testid=\"workspace-canvas\"") || html.contains("Canvas placeholder"),
        "editor.html should load placeholder canvas"
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}
