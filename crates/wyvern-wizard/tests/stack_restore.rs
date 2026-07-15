//! Sprint d.4 — `page_data` / `stack` restore + opaque JSON round-trip (REQ-0024).
//!
//! No new stack logic: exercises the public [`WizardSession`] API from d.2.

use wyvern_schema::{
    WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
    WizardStackEntry,
};
use wyvern_wizard::WizardSession;

fn page(id: &str, html: &str) -> WizardPageDescriptor {
    WizardPageDescriptor {
        id: WizardPageId::new(id),
        title: WizardPageTitle::new(id),
        html: WizardPageHtml::new(html),
        layout: None,
    }
}

fn cmd() -> WizardCommand {
    WizardCommand {
        page: page("a", "a.html"),
        config: serde_json::json!({"theme": "dark", "opaque_cfg": {"x": 1}}),
        width: Some(640),
        height: Some(480),
    }
}

/// Nested opaque payload with keys the host must never reinterpret.
fn opaque_page_data(label: &str) -> serde_json::Value {
    serde_json::json!({
        "label": label,
        "nested": {
            "keep_me": true,
            "count": 42,
            "list": [1, "two", {"k": "v"}]
        },
        "weird.key": "dot-name",
        "unicode_キー": "値"
    })
}

/// Back/forward restores `page_data`; `stack` stays prior-only (REQ-0024).
#[test]
fn stack_restore_page_data_after_back_and_forward() {
    let mut session = WizardSession::new(&cmd());
    let a_data = opaque_page_data("a");
    let b_data = opaque_page_data("b");

    session
        .navigate_next(a_data.clone(), page("b", "b.html"))
        .expect("A→B");
    session
        .navigate_next(b_data.clone(), page("c", "c.html"))
        .expect("B→C");

    let snap = session.snapshot();
    assert_eq!(snap.page.id.as_str(), "c");
    assert_eq!(snap.page_data, serde_json::json!({}));
    assert_eq!(snap.stack.len(), 2);
    assert_eq!(snap.stack[0].data, a_data);
    assert_eq!(snap.stack[1].data, b_data);
    // Current page is not in stack (REQ-0024).
    assert!(!snap
        .stack
        .iter()
        .any(|e| e.page.id.as_str() == snap.page.id.as_str()));

    let out = session
        .navigate_back(serde_json::json!({}))
        .expect("back to B");
    assert_eq!(out.page.id.as_str(), "b");
    assert_eq!(out.page_data, b_data);
    assert_eq!(out.stack.len(), 1);
    assert_eq!(out.stack[0].data, a_data);
    assert!(!out
        .stack
        .iter()
        .any(|e| e.page.id.as_str() == out.page.id.as_str()));

    let out = session
        .navigate_back(serde_json::json!({}))
        .expect("back to A");
    assert_eq!(out.page.id.as_str(), "a");
    assert_eq!(out.page_data, a_data);
    assert!(out.stack.is_empty());

    // Forward restore with non-meaningful payload: destination B keeps cached
    // page_data. Current (A) is whole-blob-replaced with `{}` before advance.
    let out = session
        .navigate_next(serde_json::json!({}), page("b", "b.html"))
        .expect("restore B");
    assert_eq!(out.page.id.as_str(), "b");
    assert_eq!(out.page_data, b_data);
    assert_eq!(out.stack.len(), 1);
    assert_eq!(out.stack[0].page.id.as_str(), "a");
    assert_eq!(out.stack[0].data, serde_json::json!({}));

    let snap = session.snapshot();
    assert_eq!(snap.page.id.as_str(), "b");
    assert_eq!(snap.page_data, b_data);
    assert_eq!(snap.stack.len(), 1);
}

/// After `navigate_next`, snapshot `stack` is prior steps only — current via `page`/`page_data`.
#[test]
fn stack_restore_prior_only_after_navigate_next() {
    let mut session = WizardSession::new(&cmd());
    assert!(session.snapshot().stack.is_empty());

    let out = session
        .navigate_next(opaque_page_data("a"), page("b", "b.html"))
        .expect("A→B");
    assert_eq!(out.page.id.as_str(), "b");
    assert_eq!(out.page_data, serde_json::json!({}));
    assert_eq!(out.stack.len(), 1);
    assert_eq!(out.stack[0].page.id.as_str(), "a");
    assert_ne!(out.stack[0].page.id.as_str(), out.page.id.as_str());

    let snap = session.snapshot();
    assert_eq!(snap.page.id.as_str(), "b");
    assert_eq!(snap.stack.len(), 1);
    assert_eq!(snap.config["theme"], "dark");
    assert_eq!(snap.config["opaque_cfg"]["x"], 1);
}

/// JSON round-trip of stack entries and `page_data` preserves opaque keys.
#[test]
fn stack_restore_json_round_trip_preserves_opaque_keys() {
    let mut session = WizardSession::new(&cmd());
    let a_data = opaque_page_data("a");
    let b_data = opaque_page_data("b");

    session
        .navigate_next(a_data.clone(), page("b", "b.html"))
        .expect("A→B");
    session
        .navigate_next(b_data.clone(), page("c", "c.html"))
        .expect("B→C");
    session
        .navigate_back(serde_json::json!({}))
        .expect("back to B");

    let snap = session.snapshot();
    assert_eq!(snap.page_data, b_data);

    // Round-trip current page_data as opaque JSON.
    let page_data_wire = serde_json::to_value(&snap.page_data).expect("serialize page_data");
    let page_data_back: serde_json::Value =
        serde_json::from_value(page_data_wire).expect("deserialize page_data");
    assert_eq!(page_data_back, b_data);
    assert_eq!(page_data_back["nested"]["keep_me"], true);
    assert_eq!(page_data_back["weird.key"], "dot-name");
    assert_eq!(page_data_back["unicode_キー"], "値");

    // Round-trip prior stack entries (REQ-0024 wire shape).
    let stack_wire = serde_json::to_value(&snap.stack).expect("serialize stack");
    let stack_back: Vec<WizardStackEntry> =
        serde_json::from_value(stack_wire).expect("deserialize stack");
    assert_eq!(stack_back, snap.stack);
    assert_eq!(stack_back.len(), 1);
    assert_eq!(stack_back[0].data, a_data);
    assert_eq!(stack_back[0].data["nested"]["list"][2]["k"], "v");
    assert_eq!(stack_back[0].data["unicode_キー"], "値");
}
