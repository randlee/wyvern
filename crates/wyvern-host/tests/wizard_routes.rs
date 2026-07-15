//! L1: wizard page HTML served at `/wizard/**` from `--ui-root`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{begin, HostOptions, ViewerMode};
use wyvern_schema::{Command, WizardCommand, WizardPageDescriptor};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn layout_picker_ui_root() -> PathBuf {
    workspace_root().join("examples/wizards/layout-picker")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn wizard_command() -> Command {
    Command::Wizard(WizardCommand {
        page: WizardPageDescriptor {
            id: "layout-picker".into(),
            title: "Layout picker".into(),
            html: "pages/layout-picker.html".into(),
            layout: None,
        },
        config: serde_json::json!({}),
        width: None,
        height: None,
    })
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

/// Poll wizard page GET until HTTP 200 (URL file alone is not readiness).
fn wait_for_wizard_page(client: &reqwest::blocking::Client, url: &str) -> String {
    let start = std::time::Instant::now();
    loop {
        match client.get(url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.text().expect("html");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET wizard page at {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

#[test]
fn wizard_routes_serve_page_html_under_wizard_mount() {
    let url_file = unique_path("wyvern-wizard-routes-url");
    let options = HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: layout_picker_ui_root(),
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file.clone()),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    };
    let handle = begin(wizard_command(), options).expect("begin");

    let dialog_url = wait_for_url_file(&url_file);
    let client = reqwest::blocking::Client::new();
    let html = wait_for_wizard_page(&client, &dialog_url);
    assert!(
        html.contains("data-testid=\"layout-picker\"") && html.contains("/shared/wyvern-api.js"),
        "expected layout-picker fixture HTML: {html}"
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}
