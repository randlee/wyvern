//! L1: turbo-flow fixture — workspace canvas + wizard page routes.

mod support;
use support::http::{http_client, wait_for_url_file, wait_for_wizard_state};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{begin, HostOptions, ViewerMode};
use wyvern_schema::{
    validate, Command, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn turbo_flow_ui_root() -> PathBuf {
    workspace_root().join("examples/wizards/turbo-flow")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn load_turbo_flow_command() -> Command {
    let path = turbo_flow_ui_root().join("wizard.json");
    let raw = std::fs::read_to_string(&path).expect("read turbo-flow wizard.json");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse wizard.json");
    validate(&value).expect("validate turbo-flow wizard.json")
}

fn page(id: &str, title: &str, html: &str) -> WizardPageDescriptor {
    WizardPageDescriptor {
        id: WizardPageId::new(id),
        title: WizardPageTitle::new(title),
        html: WizardPageHtml::new(html),
        layout: None,
    }
}

fn host_options(url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: turbo_flow_ui_root(),
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    }
}

fn post_navigate(
    client: &reqwest::blocking::Client,
    base: &str,
    body: serde_json::Value,
) -> serde_json::Value {
    let resp = client
        .post(format!("{base}/api/wizard/navigate"))
        .json(&body)
        .send()
        .expect("navigate");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "navigate body={body}"
    );
    resp.json().expect("navigate json")
}

fn sample_canvas_data() -> serde_json::Value {
    serde_json::json!({
        "nodes": [
            {
                "id": "node-1",
                "type": "turbo",
                "position": { "x": 0, "y": 0 },
                "data": { "label": "Researcher", "subtitle": "analysis" }
            },
            {
                "id": "node-2",
                "type": "turbo",
                "position": { "x": 250, "y": 80 },
                "data": { "label": "Agent 2", "subtitle": "Connect & configure" }
            }
        ],
        "edges": [
            { "id": "edge-1-2", "source": "node-1", "target": "node-2", "type": "turbo" }
        ],
        "details": {
            "node-1": {
                "core": {
                    "node_id": "node-1",
                    "name": "Researcher",
                    "role": "analysis",
                    "description": "Collects sources"
                }
            }
        },
        "editing_node_id": null
    })
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
fn wizard_turbo_flow_workspace_canvas_and_detail_pages_load() {
    let client = http_client();
    let url_file = unique_path("wyvern-wizard-turbo-flow-url");
    let handle = begin(load_turbo_flow_command(), host_options(url_file.clone())).expect("begin");

    let dialog_url = wait_for_url_file(&url_file);
    assert!(
        dialog_url.contains("/wizard/pages/canvas.html"),
        "dialog_url={dialog_url}"
    );
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard path");

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "canvas");
    assert_eq!(state["page"]["layout"], "workspace");

    let canvas = wait_for_page(&client, &format!("{base}/wizard/pages/canvas.html"));
    assert!(
        canvas.contains("turbo-flow-canvas"),
        "canvas page should mount turbo-flow shell"
    );
    let bundle = wait_for_page(&client, &format!("{base}/wizard/dist/canvas.js"));
    assert!(!bundle.is_empty(), "built canvas bundle should be served");

    let detail = wait_for_page(&client, &format!("{base}/wizard/pages/detail.html"));
    assert!(detail.contains("node-detail-form"));

    let review = wait_for_page(&client, &format!("{base}/wizard/pages/review.html"));
    assert!(review.contains("data-wizard-terminal"));

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn wizard_turbo_flow_configure_and_finish_over_http() {
    let client = http_client();
    let url_file = unique_path("wyvern-wizard-turbo-flow-url");
    let handle = begin(load_turbo_flow_command(), host_options(url_file.clone())).expect("begin");

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard path");
    let _ = wait_for_wizard_state(&client, &base);

    let canvas_data = sample_canvas_data();
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": canvas_data,
            "next": page("review", "Review", "pages/review.html")
        }),
    );

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "review");
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);

    let mut full_stack = state["stack"].as_array().expect("prior stack").clone();
    full_stack.push(serde_json::json!({
        "page": state["page"],
        "data": state["page_data"],
    }));

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "finish",
            "data": state["page_data"],
            "stack": full_stack
        }))
        .send()
        .expect("finish");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().expect("finish json");
    assert_eq!(body["button"], "finish");
    assert_eq!(body["stack"].as_array().unwrap().len(), 2);
    assert_eq!(
        body["stack"][0]["data"]["details"]["node-1"]["core"]["name"],
        "Researcher"
    );

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "finish");
            assert_eq!(w.stack.len(), 2);
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
}
