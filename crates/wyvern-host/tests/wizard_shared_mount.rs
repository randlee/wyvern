//! L1: dual-mount — `/shared/wyvern-api.js` when `--ui-root` is an example dir.

mod support;
use support::http::{http_client, wait_for_url_file};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{begin, HostOptions, ViewerMode};
use wyvern_schema::{
    Command, WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
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
            layout: None,
        },
        config: serde_json::json!({}),
        width: None,
        height: None,
    })
}

/// Poll shared JS GET until HTTP 200 (URL file alone is not readiness).
fn wait_for_shared_js(client: &reqwest::blocking::Client, url: &str) -> String {
    let start = std::time::Instant::now();
    loop {
        match client.get(url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.text().expect("js text");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET shared JS at {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

#[test]
fn wizard_shared_mount_serves_wyvern_api_js_with_example_ui_root() {
    let url_file = unique_path("wyvern-wizard-shared-url");
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
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard path");

    let client = http_client();
    let js = wait_for_shared_js(&client, &format!("{base}/shared/wyvern-api.js"));
    assert!(
        js.contains("wyvern") || js.contains("fetch") || !js.is_empty(),
        "expected packaged wyvern-api.js content"
    );

    // d.7: shared wizard chrome must come from packaged ui/, not --ui-root.
    let nav = wait_for_shared_js(&client, &format!("{base}/shared/wizard-nav.js"));
    assert!(
        nav.contains("data-wizard-terminal") || nav.contains("WyvernWizardNav"),
        "expected packaged wizard-nav.js chrome helpers"
    );

    // Example ui-root must not contain shared/ — dual mount is the only source.
    assert!(
        !layout_picker_ui_root()
            .join("shared")
            .join("wyvern-api.js")
            .is_file(),
        "fixture must not ship shared/wyvern-api.js under --ui-root"
    );
    assert!(
        !layout_picker_ui_root()
            .join("shared")
            .join("wizard-nav.js")
            .is_file(),
        "fixture must not ship shared/wizard-nav.js under --ui-root"
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}
