//! Two-fixture `next_wizard` chain, depth 17, and stdout omitting `next_wizard`.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use serde_json::json;
use wyvern::{
    check_chain_depth, run_wizard_workflow_loop, Allowlist, WorkflowError, WorkflowRunner,
    NEXT_WIZARD_MAX_DEPTH, WORKFLOW_SCRIPT_TIMEOUT,
};
use wyvern_host::{HostOptions, ViewerMode};

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn workspace_share() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../share/wyvern")
}

fn workspace_ui() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn host_options(ui_root: PathBuf, url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui(),
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
        .expect("http")
}

fn wait_for_url(path: &Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for {}", path.display());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_url_change(path: &Path, previous: &str) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if !url.is_empty() && url != previous {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for next dialog URL");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_state(client: &reqwest::blocking::Client, base: &str) -> serde_json::Value {
    let url = format!("{base}/api/wizard/state");
    let start = std::time::Instant::now();
    loop {
        match client.get(&url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.json().expect("state");
            }
            _ => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn finish(
    client: &reqwest::blocking::Client,
    base: &str,
    state: &serde_json::Value,
    next_wizard: Option<serde_json::Value>,
) {
    let mut body = json!({
        "button": "finish",
        "data": {},
        "stack": [{ "page": state["page"], "data": {} }]
    });
    if let Some(next) = next_wizard {
        body["next_wizard"] = next;
    }
    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&body)
        .send()
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK, "{resp:?}");
}

#[test]
fn two_fixture_chain_omits_next_wizard_on_stdout() {
    let share = workspace_share();
    let a_root = workspace_root().join("share/wyvern/testdata/workflow/a");
    let command: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(a_root.join("wizard.json")).expect("a wizard.json"),
    )
    .expect("json");
    let url_file = unique_path("wyvern-g4-chain-url");
    let runner = WorkflowRunner {
        allowlist: Allowlist {
            share_root: share.clone(),
            cwd: workspace_root(),
            wizard_dir: a_root.clone(),
        },
        timeout: WORKFLOW_SCRIPT_TIMEOUT,
    };
    let host = host_options(a_root, url_file.clone());
    let handle = thread::spawn(move || run_wizard_workflow_loop(command, host, &runner, false));

    let url_a = wait_for_url(&url_file);
    let base_a = url_a
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("a url");
    let client = http_client();
    let state_a = wait_for_state(&client, &base_a);
    assert_eq!(state_a["page"]["id"], "a");

    finish(
        &client,
        &base_a,
        &state_a,
        Some(json!({
            "path": "{wyvern_share}/testdata/workflow/b/wizard.json",
            "input": { "from": "a" }
        })),
    );

    let url_b = wait_for_url_change(&url_file, &url_a);
    let base_b = url_b
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("b url");
    let state_b = wait_for_state(&client, &base_b);
    assert_eq!(state_b["page"]["id"], "b");
    assert_eq!(state_b["config"]["from"], "a");

    finish(&client, &base_b, &state_b, None);
    let stdout = handle.join().expect("join").expect("chain ok");
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("stdout json");
    assert_eq!(value["button"], "finish");
    assert!(
        value.get("next_wizard").is_none(),
        "final stdout must omit next_wizard: {value}"
    );
}

#[test]
fn seventeenth_hop_is_chain_depth() {
    let err = check_chain_depth(17).expect_err("17");
    assert!(matches!(
        err,
        WorkflowError::ChainDepth {
            max: NEXT_WIZARD_MAX_DEPTH
        }
    ));
    assert_eq!(NEXT_WIZARD_MAX_DEPTH, 16);
}

#[test]
fn seventeenth_hop_emits_workflow_error() {
    assert!(check_chain_depth(NEXT_WIZARD_MAX_DEPTH).is_ok());
    let err = check_chain_depth(NEXT_WIZARD_MAX_DEPTH + 1).unwrap_err();
    let stage = wyvern::emit_workflow_error(&err).expect("emit");
    let value: serde_json::Value = serde_json::from_str(&stage).unwrap();
    assert_eq!(value["code"], "WORKFLOW_ERROR");
    assert_eq!(value["error"], "workflow");
}
