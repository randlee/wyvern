//! ADR-0005 browser-history regression matrix (sprint d.3).
//!
//! Uses only the public [`WizardSession`] API from d.2 — no private history
//! access. Cursor position is observed via `snapshot().stack.len()` (REQ-0024:
//! `stack` is `entries[0..cursor]`).

use wyvern_schema::{
    WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
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

fn cmd(first_html: &str) -> WizardCommand {
    WizardCommand {
        page: page("a", first_html),
        config: serde_json::json!({"theme": "dark"}),
        width: Some(640),
        height: Some(480),
    }
}

// Module path so `cargo test history_five_cases` matches these names.
mod history_five_cases {
    use super::*;

    /// A→B→C advances the cursor (observed as growing prior `stack`).
    #[test]
    fn forward_push_advances_cursor() {
        let mut session = WizardSession::new(&cmd("a.html"));
        assert_eq!(session.snapshot().stack.len(), 0);
        assert_eq!(session.snapshot().page.id.as_str(), "a");

        let out = session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("A→B");
        assert_eq!(out.page.id.as_str(), "b");
        assert_eq!(out.stack.len(), 1);
        assert_eq!(out.stack[0].page.id.as_str(), "a");
        assert_eq!(out.stack[0].data, serde_json::json!({"a": 1}));

        let out = session
            .navigate_next(serde_json::json!({"b": 2}), page("c", "c.html"))
            .expect("B→C");
        assert_eq!(out.page.id.as_str(), "c");
        assert_eq!(out.stack.len(), 2);
        assert_eq!(out.stack[0].page.id.as_str(), "a");
        assert_eq!(out.stack[1].page.id.as_str(), "b");
        assert_eq!(out.stack[1].data, serde_json::json!({"b": 2}));

        let snap = session.snapshot();
        assert_eq!(snap.page.id.as_str(), "c");
        assert_eq!(snap.stack.len(), 2);
    }

    /// Back moves the cursor without discarding forward entries.
    #[test]
    fn back_moves_cursor_without_truncation() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("A→B");
        session
            .navigate_next(serde_json::json!({"b": 2}), page("c", "c.html"))
            .expect("B→C");

        let out = session
            .navigate_back(serde_json::json!({}))
            .expect("back to B");
        assert_eq!(out.page.id.as_str(), "b");
        assert_eq!(out.page_data, serde_json::json!({"b": 2}));
        assert_eq!(out.stack.len(), 1);

        let out = session
            .navigate_back(serde_json::json!({}))
            .expect("back to A");
        assert_eq!(out.page.id.as_str(), "a");
        assert_eq!(out.stack.len(), 0);

        // Forward entries still intact: same-page restore reaches C with stack depth 2.
        let out = session
            .navigate_next(serde_json::json!({}), page("b", "b.html"))
            .expect("restore B");
        assert_eq!(out.page.id.as_str(), "b");
        assert_eq!(out.page_data, serde_json::json!({"b": 2}));
        assert_eq!(out.stack.len(), 1);

        let out = session
            .navigate_next(serde_json::json!({}), page("c", "c.html"))
            .expect("restore C");
        assert_eq!(out.page.id.as_str(), "c");
        assert_eq!(out.stack.len(), 2);
    }

    /// Same `next` descriptor restores cached data; `null`/`{}`/`[]`/`""` do not overwrite.
    #[test]
    fn forward_same_page_restores_data() {
        let non_meaningful = [
            serde_json::Value::Null,
            serde_json::json!({}),
            serde_json::json!([]),
            serde_json::json!(""),
        ];

        for restore_payload in non_meaningful {
            let mut session = WizardSession::new(&cmd("a.html"));
            session
                .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
                .expect("A→B");
            // Seed B's cached data by leaving B toward C.
            session
                .navigate_next(serde_json::json!({"b": "cached"}), page("c", "c.html"))
                .expect("B→C");
            session
                .navigate_back(serde_json::json!({}))
                .expect("back to B");
            session
                .navigate_back(serde_json::json!({}))
                .expect("back to A");

            let out = session
                .navigate_next(restore_payload.clone(), page("b", "b.html"))
                .unwrap_or_else(|_| panic!("restore B with {restore_payload}"));
            assert_eq!(
                out.page_data,
                serde_json::json!({"b": "cached"}),
                "non-meaningful {restore_payload} must restore cached B data"
            );
            assert_eq!(out.page.id.as_str(), "b");
            assert_eq!(out.stack.len(), 1);
        }

        // Meaningful payload overwrites the restored destination.
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("A→B");
        session
            .navigate_next(serde_json::json!({"b": "cached"}), page("c", "c.html"))
            .expect("B→C");
        session
            .navigate_back(serde_json::json!({}))
            .expect("back to B");
        session
            .navigate_back(serde_json::json!({}))
            .expect("back to A");
        let out = session
            .navigate_next(serde_json::json!({"b": "fresh"}), page("b", "b.html"))
            .expect("overwrite B");
        assert_eq!(out.page_data, serde_json::json!({"b": "fresh"}));
    }

    /// Forward to a different page truncates stale forward entries.
    #[test]
    fn forward_different_page_truncates() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("A→B");
        session
            .navigate_next(serde_json::json!({"b": 2}), page("c", "c.html"))
            .expect("B→C");
        session
            .navigate_back(serde_json::json!({}))
            .expect("back to B");
        session
            .navigate_back(serde_json::json!({}))
            .expect("back to A");

        let out = session
            .navigate_next(serde_json::json!({"a": 9}), page("d", "d.html"))
            .expect("branch to D");
        assert_eq!(out.page.id.as_str(), "d");
        assert_eq!(out.stack.len(), 1);
        assert_eq!(out.stack[0].page.id.as_str(), "a");
        assert_eq!(out.stack[0].data, serde_json::json!({"a": 9}));

        // Stale B/C are gone: cannot restore C; pushing C is a fresh branch tip.
        let out = session
            .navigate_next(serde_json::json!({}), page("c", "c.html"))
            .expect("push C as new tip");
        assert_eq!(out.page.id.as_str(), "c");
        assert_eq!(out.page_data, serde_json::json!({}));
        assert_eq!(out.stack.len(), 2);
        assert_eq!(out.stack[1].page.id.as_str(), "d");
    }

    /// Same `html`, different `id` is a branch (truncate), not a forward restore.
    #[test]
    fn forward_same_html_different_id_truncates() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "shared.html"))
            .expect("A→B");
        session
            .navigate_next(serde_json::json!({"b": "cached"}), page("c", "c.html"))
            .expect("B→C");
        session
            .navigate_back(serde_json::json!({}))
            .expect("back to B");
        session
            .navigate_back(serde_json::json!({}))
            .expect("back to A");

        // Same html as B, different id → truncate, not restore.
        let out = session
            .navigate_next(serde_json::json!({"a": 2}), page("b-alt", "shared.html"))
            .expect("branch b-alt");
        assert_eq!(out.page.id.as_str(), "b-alt");
        assert_eq!(out.page.html.as_str(), "shared.html");
        assert_eq!(out.page_data, serde_json::json!({}));
        assert_eq!(out.stack.len(), 1);
        assert_eq!(out.stack[0].data, serde_json::json!({"a": 2}));

        // Original B/C forward path is gone — stack no longer grows to depth 2 via restore.
        let snap = session.snapshot();
        assert_eq!(snap.page.id.as_str(), "b-alt");
        assert_eq!(snap.stack.len(), 1);
    }
} // mod history_four_cases
