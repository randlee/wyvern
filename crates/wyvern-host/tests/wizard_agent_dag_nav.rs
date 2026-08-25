//! L1: Agent DAG nav — canvas pair → configure → back → solo → finish (g.7 AC 1).

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
                    "name": "planner",
                    "role": "plan"
                }
            }
        },
        "editing_node_id": "node-1"
    })
}

fn solo_canvas_data() -> serde_json::Value {
    serde_json::json!({
        "nodes": [
            {
                "id": "node-1",
                "type": "turbo",
                "position": { "x": 0, "y": 0 },
                "data": { "label": "scout", "subtitle": "explore" }
            }
        ],
        "edges": [],
        "details": {
            "node-1": {
                "core": {
                    "node_id": "node-1",
                    "name": "scout",
                    "role": "explore"
                }
            }
        },
        "editing_node_id": null
    })
}

/// Pair canvas → configure node-1 → back → restore → solo graph → finish.
#[test]
fn wizard_agent_dag_pair_back_to_solo_finishes_one_node() {
    let client = http_client();
    let (handle, base, url_file) = start_agent_dag(&client);

    let pair_data = pair_canvas_data();
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": pair_data,
            "next": page("node-detail", "Configure node", "pages/detail.html")
        }),
    );

    let node1_pair = serde_json::json!({
        "node_id": "node-1",
        "name": "planner",
        "role": "plan"
    });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "back",
            "data": node1_pair
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "canvas");
    assert_eq!(state["page"]["layout"], "workspace");
    assert_eq!(state["page_data"]["nodes"][0]["id"], "node-1");
    assert_eq!(state["page_data"]["nodes"].as_array().unwrap().len(), 2);

    // Revisit configure — forward-same-page restores node-1 fields.
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": pair_data,
            "next": page("node-detail", "Configure node", "pages/detail.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "node-detail");
    assert_eq!(state["page_data"]["name"], "planner");
    assert_eq!(state["page_data"]["role"], "plan");

    post_navigate(
        &client,
        &base,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "canvas");
    assert_eq!(state["page_data"]["nodes"].as_array().unwrap().len(), 2);

    // Switch to solo — canvas blob replaced; pair extras stay out of prior stack.
    let solo_data = solo_canvas_data();
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": solo_data,
            "next": page("review", "Review", "pages/review.html")
        }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "review");
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(
        state["stack"][0]["data"]["nodes"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        state["stack"][0]["data"]["details"]["node-1"]["core"]["name"],
        "scout"
    );

    let dag = serde_json::json!({
        "layout_id": "solo",
        "nodes": [
            { "id": "node-1", "name": "scout", "role": "explore" }
        ],
        "edges": [
            ["node-1", "finish"]
        ]
    });
    let finish_data = serde_json::json!({ "dag": dag });
    let stack = serde_json::json!([
        {
            "page": page("canvas", "Agent DAG", "pages/canvas.html"),
            "data": solo_data
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
    assert_eq!(body["data"]["dag"]["nodes"][0]["id"], "node-1");
    assert_eq!(body["data"]["dag"]["nodes"][0]["name"], "scout");
    assert_ne!(body["data"]["dag"]["nodes"][0]["name"], "planner");

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.data["dag"]["layout_id"], "solo");
            assert_eq!(w.data["dag"]["nodes"].as_array().unwrap().len(), 1);
            assert_eq!(w.stack.len(), 2);
            assert_eq!(w.stack[0].data["nodes"].as_array().unwrap().len(), 1);
            assert_eq!(
                w.stack[0].data["details"]["node-1"]["core"]["name"],
                "scout"
            );
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
}
