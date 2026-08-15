//! Regression: layout-picker trio flow — back from agent-2 then forward must
//! restore agent-2's cached description, not clobber it with agent-1's payload.
//!
//! User report (Phase D demo): agent-1 `1/3`, agent-2 `2/3`, back → agent-1
//! shows `1/3` (correct), Next (forward-restore) → agent-2 incorrectly showed `1/3`.

use wyvern_schema::{
    WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
};
use wyvern_wizard::WizardSession;

fn agent_page(n: u32) -> WizardPageDescriptor {
    WizardPageDescriptor {
        id: WizardPageId::new(format!("agent-{n}")),
        title: WizardPageTitle::new(format!("Agent {n}")),
        html: WizardPageHtml::new("pages/agent.html"),
        layout: None,
    }
}

fn layout_picker_cmd() -> WizardCommand {
    WizardCommand {
        page: WizardPageDescriptor {
            id: WizardPageId::new("layout-picker"),
            title: WizardPageTitle::new("Choose layout"),
            html: WizardPageHtml::new("pages/layout-picker.html"),
            layout: None,
        },
        config: serde_json::json!({
            "layouts": [
                {"id": "solo", "label": "Solo", "agents": 1},
                {"id": "pair", "label": "Pair", "agents": 2},
                {"id": "trio", "label": "Trio", "agents": 3}
            ]
        }),
        width: None,
        height: None,
    }
}

fn agent_form(description: &str) -> serde_json::Value {
    serde_json::json!({
        "name": "1",
        "description": description
    })
}

/// Public-API reproduction of the reported back → forward-restore bug.
#[test]
fn layout_picker_back_then_forward_restores_agent_two_description() {
    let mut session = WizardSession::new(&layout_picker_cmd());

    // Pick trio → agent-1.
    session
        .navigate_next(
            serde_json::json!({
                "layout_id": "trio",
                "label": "Trio",
                "agent_count": 3
            }),
            agent_page(1),
        )
        .expect("layout-picker → agent-1");

    // Agent-1: description 1/3 → agent-2.
    session
        .navigate_next(agent_form("1/3"), agent_page(2))
        .expect("agent-1 → agent-2");

    // Agent-2: description 2/3 → agent-3 (commits agent-2 blob before advancing).
    session
        .navigate_next(agent_form("2/3"), agent_page(3))
        .expect("agent-2 → agent-3");

    // Back to agent-2 — still 2/3.
    let on_agent_2 = session
        .navigate_back(serde_json::json!({}))
        .expect("agent-3 → agent-2");
    assert_eq!(on_agent_2.page.id.as_str(), "agent-2");
    assert_eq!(on_agent_2.page_data, agent_form("2/3"));

    // Back to agent-1 — still 1/3.
    let on_agent_1 = session
        .navigate_back(serde_json::json!({}))
        .expect("agent-2 → agent-1");
    assert_eq!(on_agent_1.page.id.as_str(), "agent-1");
    assert_eq!(on_agent_1.page_data, agent_form("1/3"));

    // Forward-restore: Next on agent-1 re-submits agent-1 data; agent-2 must keep 2/3.
    let restored = session
        .navigate_next(agent_form("1/3"), agent_page(2))
        .expect("forward-restore agent-2");
    assert_eq!(restored.page.id.as_str(), "agent-2");
    assert_eq!(
        restored.page_data,
        agent_form("2/3"),
        "forward-restore must not overwrite agent-2 with agent-1's navigate payload"
    );
    // Outgoing agent-1 entry was updated with the re-submitted form.
    assert_eq!(restored.stack.len(), 2);
    assert_eq!(restored.stack[0].data["layout_id"], "trio");
    assert_eq!(restored.stack[1].data, agent_form("1/3"));
}

/// Back from agent-3 must persist in-progress form data so forward-restore shows 3/3.
#[test]
fn layout_picker_back_from_agent_three_then_forward_restores_description() {
    let mut session = WizardSession::new(&layout_picker_cmd());

    session
        .navigate_next(
            serde_json::json!({
                "layout_id": "trio",
                "label": "Trio",
                "agent_count": 3
            }),
            agent_page(1),
        )
        .expect("layout-picker → agent-1");
    session
        .navigate_next(agent_form("1/3"), agent_page(2))
        .expect("agent-1 → agent-2");
    session
        .navigate_next(agent_form("2/3"), agent_page(3))
        .expect("agent-2 → agent-3");

    // Empty back payload leaves agent-3 uncached (pre-fix client sent `{}`).
    session
        .navigate_back(serde_json::json!({}))
        .expect("back without form data");
    session
        .navigate_back(serde_json::json!({}))
        .expect("back to agent-1");
    session
        .navigate_next(agent_form("1/3"), agent_page(2))
        .expect("forward agent-2");
    let empty_agent_3 = session
        .navigate_next(agent_form("2/3"), agent_page(3))
        .expect("forward agent-3 without cached data");
    assert_eq!(
        empty_agent_3.page_data,
        serde_json::json!({}),
        "back without meaningful payload does not cache in-progress agent-3 data"
    );

    // Meaningful back payload (collectCurrentPageData contract) preserves 3/3.
    let mut session = WizardSession::new(&layout_picker_cmd());
    session
        .navigate_next(
            serde_json::json!({
                "layout_id": "trio",
                "label": "Trio",
                "agent_count": 3
            }),
            agent_page(1),
        )
        .expect("layout-picker → agent-1");
    session
        .navigate_next(agent_form("1/3"), agent_page(2))
        .expect("agent-1 → agent-2");
    session
        .navigate_next(agent_form("2/3"), agent_page(3))
        .expect("agent-2 → agent-3");

    session
        .navigate_back(agent_form("3/3"))
        .expect("back with agent-3 form data");
    session
        .navigate_back(serde_json::json!({}))
        .expect("back to agent-1");
    session
        .navigate_next(agent_form("1/3"), agent_page(2))
        .expect("forward agent-2");
    let restored = session
        .navigate_next(agent_form("2/3"), agent_page(3))
        .expect("forward agent-3");
    assert_eq!(
        restored.page_data,
        agent_form("3/3"),
        "forward-restore must return agent-3 data saved on meaningful back"
    );
}
