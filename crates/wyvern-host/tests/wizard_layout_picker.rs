//! L1: layout-picker fixture — DAG navigate + finish over HTTP (no GUI).

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

fn layout_picker_ui_root() -> PathBuf {
    workspace_root().join("examples/wizards/layout-picker")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn load_layout_picker_command() -> Command {
    let path = layout_picker_ui_root().join("wizard.json");
    let raw = std::fs::read_to_string(&path).expect("read layout-picker wizard.json");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse wizard.json");
    validate(&value).expect("validate layout-picker wizard.json")
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

fn start_layout_picker(client: &reqwest::blocking::Client) -> (DialogHandle, String, PathBuf) {
    let url_file = unique_path("wyvern-wizard-layout-picker-url");
    let handle =
        begin(load_layout_picker_command(), host_options(url_file.clone())).expect("begin");
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

/// Happy path: solo layout → one agent → finish with full stack.
#[test]
fn wizard_layout_picker_solo_flow_finishes_with_full_stack() {
    let client = http_client();
    let (handle, base, url_file) = start_layout_picker(&client);

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "layout-picker");
    assert_eq!(state["config"]["layouts"].as_array().unwrap().len(), 3);
    assert_eq!(state["config"]["layouts"][0]["id"], "solo");
    assert_eq!(state["config"]["layouts"][0]["agents"], 1);

    let layout_data = serde_json::json!({
        "layout_id": "solo",
        "label": "Solo",
        "agent_count": 1
    });
    let nav = post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": layout_data,
            "next": page("agent-1", "Agent 1", "pages/agent.html")
        }),
    );
    assert_eq!(nav["ok"], true);
    assert!(
        nav["url"]
            .as_str()
            .unwrap()
            .ends_with("/wizard/pages/agent.html"),
        "url={}",
        nav["url"]
    );

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "agent-1");
    assert_eq!(state["stack"].as_array().unwrap().len(), 1);
    assert_eq!(state["stack"][0]["data"], layout_data);

    let agent_data = serde_json::json!({
        "name": "Alpha",
        "description": "Solo scout"
    });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": agent_data,
            "next": page("finish", "Review", "pages/finish.html")
        }),
    );

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "finish");
    assert_eq!(state["stack"].as_array().unwrap().len(), 2);

    let finish_data = serde_json::json!({});
    let stack = serde_json::json!([
        {
            "page": page("layout-picker", "Choose layout", "pages/layout-picker.html"),
            "data": layout_data
        },
        {
            "page": page("agent-1", "Agent 1", "pages/agent.html"),
            "data": agent_data
        },
        {
            "page": page("finish", "Review", "pages/finish.html"),
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
    assert_eq!(body["stack"].as_array().unwrap().len(), 3);
    assert_eq!(body["stack"][0]["data"]["layout_id"], "solo");
    assert_eq!(body["stack"][1]["data"]["name"], "Alpha");

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.button.as_str(), "finish");
            assert_eq!(w.stack.len(), 3);
            assert_eq!(w.stack[0].data["layout_id"], "solo");
            assert_eq!(w.stack[1].data["name"], "Alpha");
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
}

/// Pair → agent-1 → back → switch to solo → complete (branch + restore).
#[test]
fn wizard_layout_picker_pair_back_to_solo_branches() {
    let client = http_client();
    let (handle, base, url_file) = start_layout_picker(&client);

    let pair_data = serde_json::json!({
        "layout_id": "pair",
        "label": "Pair",
        "agent_count": 2
    });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": pair_data,
            "next": page("agent-1", "Agent 1", "pages/agent.html")
        }),
    );

    let agent1 = serde_json::json!({
        "name": "One",
        "description": "first of pair"
    });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": agent1,
            "next": page("agent-2", "Agent 2", "pages/agent.html")
        }),
    );

    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "agent-2");
    assert_eq!(state["stack"].as_array().unwrap().len(), 2);

    // Back to agent-1, then back to layout-picker.
    post_navigate(
        &client,
        &base,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "agent-1");
    assert_eq!(state["page_data"]["name"], "One");

    post_navigate(
        &client,
        &base,
        serde_json::json!({ "action": "back", "data": {} }),
    );
    let state = wait_for_wizard_state(&client, &base);
    assert_eq!(state["page"]["id"], "layout-picker");
    assert_eq!(state["page_data"]["layout_id"], "pair");

    // Branch to solo — truncates forward pair history.
    let solo_data = serde_json::json!({
        "layout_id": "solo",
        "label": "Solo",
        "agent_count": 1
    });
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

    let agent_data = serde_json::json!({
        "name": "Soloist",
        "description": "after branch"
    });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": agent_data,
            "next": page("finish", "Review", "pages/finish.html")
        }),
    );

    let finish_data = serde_json::json!({});
    let stack = serde_json::json!([
        {
            "page": page("layout-picker", "Choose layout", "pages/layout-picker.html"),
            "data": solo_data
        },
        {
            "page": page("agent-1", "Agent 1", "pages/agent.html"),
            "data": agent_data
        },
        {
            "page": page("finish", "Review", "pages/finish.html"),
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

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.stack.len(), 3);
            assert_eq!(w.stack[0].data["layout_id"], "solo");
            assert_eq!(w.stack[1].data["name"], "Soloist");
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
}

/// Pair layout collects two agent configs before finish.
#[test]
fn wizard_layout_picker_pair_collects_two_agents() {
    let client = http_client();
    let (handle, base, url_file) = start_layout_picker(&client);

    let layout_data = serde_json::json!({
        "layout_id": "pair",
        "label": "Pair",
        "agent_count": 2
    });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": layout_data,
            "next": page("agent-1", "Agent 1", "pages/agent.html")
        }),
    );
    let a1 = serde_json::json!({ "name": "A", "description": "one" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": a1,
            "next": page("agent-2", "Agent 2", "pages/agent.html")
        }),
    );
    let a2 = serde_json::json!({ "name": "B", "description": "two" });
    post_navigate(
        &client,
        &base,
        serde_json::json!({
            "action": "next",
            "data": a2,
            "next": page("finish", "Review", "pages/finish.html")
        }),
    );

    let finish_data = serde_json::json!({});
    let stack = serde_json::json!([
        {
            "page": page("layout-picker", "Choose layout", "pages/layout-picker.html"),
            "data": layout_data
        },
        {
            "page": page("agent-1", "Agent 1", "pages/agent.html"),
            "data": a1
        },
        {
            "page": page("agent-2", "Agent 2", "pages/agent.html"),
            "data": a2
        },
        {
            "page": page("finish", "Review", "pages/finish.html"),
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
    let body: serde_json::Value = resp.json().expect("json");
    assert_eq!(body["stack"].as_array().unwrap().len(), 4);
    assert_eq!(body["stack"][2]["data"]["name"], "B");

    let result = handle.await_result().expect("result");
    match result {
        wyvern_schema::CommandResult::Wizard(w) => {
            assert_eq!(w.stack.len(), 4);
        }
        other => panic!("expected wizard result, got {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
}
