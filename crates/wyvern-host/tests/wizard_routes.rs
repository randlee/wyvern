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

    let start = std::time::Instant::now();
    let dialog_url = loop {
        if let Ok(url) = std::fs::read_to_string(&url_file) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                break url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for dialog URL");
        }
        thread::sleep(Duration::from_millis(20));
    };

    let client = reqwest::blocking::Client::new();
    let page = client
        .get(&dialog_url)
        .send()
        .expect("GET page")
        .error_for_status()
        .expect("page status");
    let html = page.text().expect("html");
    assert!(
        html.contains("data-testid=\"layout-picker\"") && html.contains("/shared/wyvern-api.js"),
        "expected layout-picker fixture HTML: {html}"
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}
