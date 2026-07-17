//! L1: `GET /api/wizard/state` returns full `WizardStateResponse` wire shape.

mod support;
use support::http::{http_client, wait_for_url_file, wait_for_wizard_state};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use wyvern_host::{begin, HostOptions, ViewerMode};
use wyvern_schema::{
    Command, WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageLayout,
    WizardPageTitle,
};

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
            id: WizardPageId::new("layout-picker"),
            title: WizardPageTitle::new("Layout picker"),
            html: WizardPageHtml::new("pages/layout-picker.html"),
            layout: Some(WizardPageLayout::Dialog),
        },
        config: serde_json::json!({"theme": "dark"}),
        width: Some(640),
        height: Some(480),
    })
}

fn host_options(url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: layout_picker_ui_root(),
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    }
}

#[test]
fn wizard_state_returns_full_wire_shape_on_first_page() {
    let url_file = unique_path("wyvern-wizard-state-url");
    let options = host_options(url_file.clone());
    let handle = begin(wizard_command(), options).expect("begin");

    let dialog_url = wait_for_url_file(&url_file);
    assert!(
        dialog_url.contains("/wizard/pages/layout-picker.html"),
        "dialog_url={dialog_url}"
    );
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard path in url");

    let client = http_client();
    let state = wait_for_wizard_state(&client, &base);

    assert_eq!(state["type"], "wizard");
    assert_eq!(state["config"]["theme"], "dark");
    assert_eq!(state["page"]["id"], "layout-picker");
    assert_eq!(state["page"]["title"], "Layout picker");
    assert_eq!(state["page"]["html"], "pages/layout-picker.html");
    assert_eq!(state["page"]["layout"], "dialog");
    assert_eq!(state["page_data"], serde_json::json!({}));
    assert_eq!(state["stack"], serde_json::json!([]));
    assert_eq!(state["width"], 640);
    assert_eq!(state["height"], 480);

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}
