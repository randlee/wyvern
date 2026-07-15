//! Sprint d.7 — N=1 wizard session: snapshot + navigate/finish paths.
//!
//! Page JS owns chrome UX; these tests lock the stack machine behaviour for a
//! single-page wizard (cursor=0, empty prior `stack`).

use wyvern_schema::{
    WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
    WizardStackEntry, WizardTerminalButton,
};
use wyvern_wizard::{WizardError, WizardSession};

fn page(id: &str, html: &str) -> WizardPageDescriptor {
    WizardPageDescriptor {
        id: WizardPageId::new(id),
        title: WizardPageTitle::new(id),
        html: WizardPageHtml::new(html),
        layout: None,
    }
}

fn single_page_cmd() -> WizardCommand {
    WizardCommand {
        page: page("only", "pages/only.html"),
        config: serde_json::json!({"note": "N=1"}),
        width: Some(480),
        height: Some(320),
    }
}

/// N=1 snapshot: current page is `only`, prior `stack` empty, `page_data` `{}`.
#[test]
fn single_page_snapshot_empty_stack() {
    let session = WizardSession::new(&single_page_cmd());
    let snap = session.snapshot();
    assert_eq!(snap.page.id.as_str(), "only");
    assert_eq!(snap.page.html.as_str(), "pages/only.html");
    assert_eq!(snap.page_data, serde_json::json!({}));
    assert!(snap.stack.is_empty());
    assert_eq!(snap.config["note"], "N=1");
}

/// Finish from the sole page without navigating — full visited stack length 1.
#[test]
fn single_page_finish_without_navigate() {
    let session = WizardSession::new(&single_page_cmd());
    let data = serde_json::json!({"note": "done"});
    let stack = vec![WizardStackEntry {
        page: page("only", "pages/only.html"),
        data: data.clone(),
    }];
    let result = session
        .finish(WizardTerminalButton::Finish, data.clone(), stack)
        .expect("finish N=1");
    assert_eq!(result.button.as_str(), "finish");
    assert_eq!(result.data, data);
    assert_eq!(result.stack.len(), 1);
    assert_eq!(result.stack[0].page.id.as_str(), "only");
    assert_eq!(result.stack[0].data, data);
}

/// Empty `{}` finish data is valid on N=1 (chrome treats missing data as `{}`).
#[test]
fn single_page_finish_empty_data() {
    let session = WizardSession::new(&single_page_cmd());
    let data = serde_json::json!({});
    let stack = vec![WizardStackEntry {
        page: page("only", "pages/only.html"),
        data: data.clone(),
    }];
    let result = session
        .finish(WizardTerminalButton::Finish, data.clone(), stack)
        .expect("finish empty");
    assert_eq!(result.data, serde_json::json!({}));
    assert_eq!(result.stack.len(), 1);
    assert_eq!(result.stack[0].data, serde_json::json!({}));
}

/// Back at cursor=0 (first / only page) is an error — chrome hides Back instead.
#[test]
fn single_page_back_at_first_errors() {
    let mut session = WizardSession::new(&single_page_cmd());
    let err = session
        .navigate_back(serde_json::json!({}))
        .expect_err("back at N=1");
    assert!(matches!(err, WizardError::AtFirstPage));
}

/// Navigate away from N=1 then finish on the second page (chrome next path).
#[test]
fn single_page_navigate_then_finish() {
    let mut session = WizardSession::new(&single_page_cmd());
    let only_data = serde_json::json!({"note": "leaving"});
    let out = session
        .navigate_next(only_data.clone(), page("done", "pages/done.html"))
        .expect("only→done");
    assert_eq!(out.page.id.as_str(), "done");
    assert_eq!(out.stack.len(), 1);
    assert_eq!(out.stack[0].page.id.as_str(), "only");
    assert_eq!(out.stack[0].data, only_data);
    assert_eq!(out.page_data, serde_json::json!({}));

    let finish_data = serde_json::json!({});
    let stack = vec![
        WizardStackEntry {
            page: page("only", "pages/only.html"),
            data: only_data,
        },
        WizardStackEntry {
            page: page("done", "pages/done.html"),
            data: finish_data.clone(),
        },
    ];
    let result = session
        .finish(WizardTerminalButton::Finish, finish_data, stack)
        .expect("finish after navigate");
    assert_eq!(result.button.as_str(), "finish");
    assert_eq!(result.stack.len(), 2);
}

/// After leaving N=1, back restores the sole prior page and empty prior stack.
#[test]
fn single_page_navigate_back_restores() {
    let mut session = WizardSession::new(&single_page_cmd());
    let only_data = serde_json::json!({"note": "keep"});
    session
        .navigate_next(only_data.clone(), page("done", "pages/done.html"))
        .expect("only→done");
    let out = session
        .navigate_back(serde_json::json!({}))
        .expect("back to only");
    assert_eq!(out.page.id.as_str(), "only");
    assert_eq!(out.page_data, only_data);
    assert!(out.stack.is_empty());
}
