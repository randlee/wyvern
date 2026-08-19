//! Host copies `next_wizard` from finish onto `WizardResult` (ADR-0024).

mod support;
use support::http::{http_client, wait_for_url_file, wait_for_wizard_state};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use wyvern_host::{begin, HostOptions, ViewerMode};
use wyvern_schema::{
    Command, NextWizard, WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId,
    WizardPageTitle, WizardResult,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn write_ui_root() -> PathBuf {
    let root = unique_path("wyvern-next-wizard-ui");
    let pages = root.join("pages");
    std::fs::create_dir_all(&pages).expect("mkdir");
    std::fs::write(
        pages.join("a.html"),
        "<!DOCTYPE html><title>a</title><h1>a</h1>",
    )
    .expect("write");
    root
}

fn wizard_command() -> Command {
    Command::Wizard(WizardCommand {
        page: WizardPageDescriptor {
            id: WizardPageId::new("a"),
            title: WizardPageTitle::new("A"),
            html: WizardPageHtml::new("pages/a.html"),
            layout: None,
        },
        config: serde_json::json!({}),
        width: None,
        height: None,
        workflow: None,
    })
}

#[test]
fn finish_copies_next_wizard_after_stack_validation() {
    let ui_root = write_ui_root();
    let url_file = unique_path("wyvern-next-wizard-url");
    let handle = begin(
        wizard_command(),
        HostOptions {
            bind: SocketAddr::from(([127, 0, 0, 1], 0)),
            ui_root: ui_root.clone(),
            shared_ui_root: workspace_root().join("ui"),
            viewer: ViewerMode::None,
            dialog_url_env: true,
            dialog_url_file: Some(url_file.clone()),
            allow_non_loopback: false,
            session_timeout: Duration::from_secs(30),
            mock_picker: None,
        },
    )
    .expect("begin");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("wizard path");
    let client = http_client();
    let state = wait_for_wizard_state(&client, &base);

    let next = NextWizard {
        path: "{wyvern_share}/testdata/workflow/b/wizard.json".into(),
        input: serde_json::json!({"from": "a"}),
        ui_root: None,
    };
    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "finish",
            "data": {},
            "stack": [{ "page": state["page"], "data": {} }],
            "next_wizard": next
        }))
        .send()
        .expect("finish");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: WizardResult = resp.json().expect("wizard result");
    let copied = body.next_wizard.expect("next_wizard copied");
    assert_eq!(copied.path, next.path);
    assert_eq!(copied.input, next.input);
    assert!(copied.ui_root.is_none());

    drop(handle);
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}
