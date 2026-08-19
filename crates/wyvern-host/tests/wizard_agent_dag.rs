//! L1: Agent DAG finish asserts `data.dag` wire-shape (g.7 AC 2).

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
    let url_file = unique_path("wyvern-wizard-agent-dag-url");
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

fn pair_dag() -> serde_json::Value {
    serde_json::json!({
        "layout_id": "pair",
        "nodes": [
            { "id": "agent-1", "name": "planner", "role": "plan" },
            { "id": "agent-2", "name": "reviewer", "role": "review" }
        ],
        "edges": [
            ["layout-picker", "agent-1"],
            ["agent-1", "agent-2"],
            ["agent-2", "finish"]
        ]
    })
}

fn assert_dag_wire_shape(dag: &serde_json::Value) {
    assert!(
        dag["layout_id"].is_string(),
        "data.dag.layout_id must be a string: {dag}"
    );
    let nodes = dag["nodes"].as_array().expect("data.dag.nodes array");
    assert!(!nodes.is_empty(), "data.dag.nodes must not be empty");
    for node in nodes {
        assert!(node["id"].is_string(), "node.id: {node}");
        assert!(node["name"].is_string(), "node.name: {node}");
        assert!(node["role"].is_string(), "node.role: {node}");
    }
    let edges = dag["edges"].as_array().expect("data.dag.edges array");
    assert!(!edges.is_empty(), "data.dag.edges must not be empty");
    for edge in edges {
        let pair = edge.as_array().expect("[from, to]");
        assert_eq!(pair.len(), 2, "edge pair: {edge}");
        assert!(pair[0].is_string() && pair[1].is_string(), "{edge}");
    }
}

/// Pair path finish includes the AC 2 `data.dag` wire-shape.
#[test]
fn wizard_agent_dag_finish_has_dag_wire_shape() {
    let client = http_client();
    let (handle, base, url_file) = start_agent_dag(&client);

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "layout");
    assert_eq!(state["config"]["layouts"].as_array().unwrap().len(), 3);

    let layout_data = serde_json::json!({ "layout_id": "pair" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": layout_data,
            "next": page("agent-1", "Agent 1", "pages/agent.html")
        }),
    );

    let agent1 = serde_json::json!({ "name": "planner", "role": "plan" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": agent1,
            "next": page("agent-2", "Agent 2", "pages/agent.html")
        }),
    );

    let agent2 = serde_json::json!({ "name": "reviewer", "role": "review" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": agent2,
            "next": page("review", "Review", "pages/review.html")
        }),
    );

    let dag = pair_dag();
    let finish_data = serde_json::json!({ "dag": dag });
    let stack = serde_json::json!([
        {
            "page": page("layout", "Agent DAG", "pages/layout.html"),
            "data": layout_data
        },
        {
            "page": page("agent-1", "Agent 1", "pages/agent.html"),
            "data": agent1
        },
        {
            "page": page("agent-2", "Agent 2", "pages/agent.html"),
            "data": agent2
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
    assert_eq!(body["button"], "finish");
    assert_eq!(body["data"]["dag"], dag);
    assert_dag_wire_shape(&body["data"]["dag"]);

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "finish");
            assert_eq!(w.data["dag"], dag);
            assert_dag_wire_shape(&w.data["dag"]);
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
}
