//! L1: Agent DAG nav — pair → agent-1 → back → solo → finish (g.7 AC 1).

mod support;
use support::http::{http_client, wait_for_url_file, wait_for_wizard_state};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use wyvern_host::{begin, DialogHandle, HostOptions, ViewerMode};
use wyvern_schema::{
    validate, Command, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn agent_dag_ui_root() -> PathBuf {
    workspace_root().join("share/wyvern/examples/agent-dag")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn load_agent_dag_command() -> Command {
    let path = agent_dag_ui_root().join("wizard.json");
    let raw = std::fs::read_to_string(&path).expect("read agent-dag wizard.json");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse wizard.json");
    validate(&value).expect("validate agent-dag wizard.json")
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
        ui_root: agent_dag_ui_root(),
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    }
}

fn start_agent_dag(client: &reqwest::blocking::Client) -> (DialogHandle, String, PathBuf) {
    let url_file = unique_path("wyvern-wizard-agent-dag-nav-url");
    let handle = begin(load_agent_dag_command(), host_options(url_file.clone())).expect("begin");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard path");
    let _ = wait_for_wizard_state(client, &base);
    (handle, base, url_file)
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

/// Pair → configure agent-1 → back → revisit pair (restore) → solo → finish.
#[test]
fn wizard_agent_dag_pair_back_to_solo_finishes_one_node() {
    let client = http_client();
    let (handle, base, url_file) = start_agent_dag(&client);

    let pair_data = serde_json::json!({ "layout_id": "pair" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": pair_data,
            "next": page("agent-1", "Agent 1", "pages/agent.html")
        }),
    );

    let agent1_pair = serde_json::json!({ "name": "planner", "role": "plan" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "back",
            "data": agent1_pair
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "layout");
    assert_eq!(state["page_data"]["layout_id"], "pair");

    // Revisit pair — forward-same-page restores agent-1 fields.
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": pair_data,
            "next": page("agent-1", "Agent 1", "pages/agent.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "agent-1");
    assert_eq!(state["page_data"]["name"], "planner");
    assert_eq!(state["page_data"]["role"], "plan");

    post_navigate(
        &client,
        &base,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "layout");
    assert_eq!(state["page_data"]["layout_id"], "pair");

    // Switch to solo — layout blob replaced; pair extras stay out of prior stack.
    let solo_data = serde_json::json!({ "layout_id": "solo" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": solo_data,
            "next": page("agent-1", "Agent 1", "pages/agent.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "agent-1");
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["data"]["layout_id"], "solo");

    // New solo fields replace the restored pair blob on the way to review.
    let agent1_solo = serde_json::json!({ "name": "scout", "role": "explore" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": agent1_solo,
            "next": page("review", "Review", "pages/review.html")
        }),
    );

    let dag = serde_json::json!({
        "layout_id": "solo",
        "nodes": [
            { "id": "agent-1", "name": "scout", "role": "explore" }
        ],
        "edges": [
            ["layout-picker", "agent-1"],
            ["agent-1", "finish"]
        ]
    });
    let finish_data = serde_json::json!({ "dag": dag });
    let stack = serde_json::json!([
        {
            "page": page("layout", "Agent DAG", "pages/layout.html"),
            "data": solo_data
        },
        {
            "page": page("agent-1", "Agent 1", "pages/agent.html"),
            "data": agent1_solo
        },
        {
            "page": page("review", "Review", "pages/review.html"),
            "data": finish_data
        }
    ]);

    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "finish",
            "data": finish_data,
            "stack": stack
        }))
        .send()
        .expect("finish");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().expect("finish json");
    assert_eq!(body["data"]["dag"]["layout_id"], "solo");
    assert_eq!(body["data"]["dag"]["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"]["dag"]["nodes"][0]["id"], "agent-1");
    assert_eq!(body["data"]["dag"]["nodes"][0]["name"], "scout");
    assert_ne!(body["data"]["dag"]["nodes"][0]["name"], "planner");

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.data["dag"]["layout_id"], "solo");
            assert_eq!(w.data["dag"]["nodes"].as_array().unwrap().len(), 1);
            assert_eq!(w.stack.len(), 3);
            assert_eq!(w.stack[0].data["layout_id"], "solo");
            assert_eq!(w.stack[1].data["name"], "scout");
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
}
