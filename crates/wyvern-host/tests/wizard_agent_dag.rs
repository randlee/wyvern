//! L1: Agent DAG finish asserts `data.dag` wire-shape (g.7 AC 2).
//! Canvas workspace + node configure replace the old layout/agent HTML flow.

mod support;
use support::http::{http_client, wait_for_url_file, wait_for_wizard_state};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use wyvern_host::{begin, DialogHandle, HostOptions, ViewerMode};
use wyvern_schema::{
    validate, Command, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageLayout,
    WizardPageTitle,
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
        layout: if id == "canvas" {
            Some(WizardPageLayout::Workspace)
        } else {
            None
        },
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

fn pair_canvas_data() -> serde_json::Value {
    serde_json::json!({
        "nodes": [
            {
                "id": "node-1",
                "type": "turbo",
                "position": { "x": 0, "y": 0 },
                "data": { "label": "planner", "subtitle": "plan" }
            },
            {
                "id": "node-2",
                "type": "turbo",
                "position": { "x": 250, "y": 80 },
                "data": { "label": "reviewer", "subtitle": "review" }
            }
        ],
        "edges": [
            { "id": "edge-1-2", "source": "node-1", "target": "node-2", "type": "turbo" }
        ],
        "details": {
            "node-1": {
                "core": {
                    "node_id": "node-1",
                    "name": "planner",
                    "role": "plan"
                }
            },
            "node-2": {
                "core": {
                    "node_id": "node-2",
                    "name": "reviewer",
                    "role": "review"
                }
            }
        },
        "editing_node_id": "node-1"
    })
}

fn pair_dag() -> serde_json::Value {
    serde_json::json!({
        "layout_id": "pair",
        "nodes": [
            { "id": "node-1", "name": "planner", "role": "plan" },
            { "id": "node-2", "name": "reviewer", "role": "review" }
        ],
        "edges": [
            ["node-1", "node-2"]
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

/// Pair canvas path finish includes the AC 2 `data.dag` wire-shape.
#[test]
fn wizard_agent_dag_finish_has_dag_wire_shape() {
    let client = http_client();
    let (handle, base, url_file) = start_agent_dag(&client);

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "canvas");
    assert_eq!(state["page"]["layout"], "workspace");
    assert_eq!(state["config"]["layouts"].as_array().unwrap().len(), 3);

    let canvas = client
        .get(format!("{base}/wizard/pages/canvas.html"))
        .send()
        .expect("canvas page")
        .text()
        .expect("canvas html");
    assert!(
        canvas.contains("turbo-flow-canvas"),
        "agent-dag should embed the turbo-flow canvas"
    );
    let bundle = client
        .get(format!("{base}/wizard/dist/canvas.js"))
        .send()
        .expect("canvas bundle");
    assert_eq!(bundle.status(), reqwest::StatusCode::OK);
    assert!(!bundle.text().expect("bundle").is_empty());

    let canvas_data = pair_canvas_data();
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": canvas_data,
            "next": page("node-detail", "Configure node", "pages/detail.html")
        }),
    );

    let node1 = serde_json::json!({
        "node_id": "node-1",
        "name": "planner",
        "role": "plan"
    });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": node1,
            "next": page("review", "Review", "pages/review.html")
        }),
    );

    let dag = pair_dag();
    let finish_data = serde_json::json!({ "dag": dag });
    let stack = serde_json::json!([
        {
            "page": page("canvas", "Agent DAG", "pages/canvas.html"),
            "data": canvas_data
        },
        {
            "page": page("node-detail", "Configure node", "pages/detail.html"),
            "data": node1
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
