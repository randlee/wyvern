//! Private browser-history stack for [`crate::WizardSession`].
//!
//! Not part of the public API — host must not import this module (ADR-0007).

use wyvern_schema::{WizardPageDescriptor, WizardStackEntry};

/// Maximum number of history entries (including the first page).
///
/// Chosen as a hard service limit so unbounded navigate-next cannot grow the
/// session without bound. Forward-same-page restore does not count against growth.
pub(crate) const MAX_WIZARD_STACK_DEPTH: usize = 64;

/// One history entry: page descriptor + opaque data.
#[derive(Debug, Clone)]
pub(crate) struct HistoryEntry {
    pub(crate) page: WizardPageDescriptor,
    pub(crate) data: serde_json::Value,
}

impl HistoryEntry {
    pub(crate) fn new(page: WizardPageDescriptor) -> Self {
        Self {
            page,
            data: serde_json::json!({}),
        }
    }

    pub(crate) fn to_stack_entry(&self) -> WizardStackEntry {
        WizardStackEntry {
            page: self.page.clone(),
            data: self.data.clone(),
        }
    }
}

/// Browser-history model: visited entries + cursor (ADR-0005).
#[derive(Debug, Clone)]
pub(crate) struct History {
    pub(crate) entries: Vec<HistoryEntry>,
    pub(crate) cursor: usize,
}

/// Navigation failure inside the private history stack (RBP-F011).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoryNavigateError {
    /// Branch push would exceed [`MAX_WIZARD_STACK_DEPTH`].
    StackDepthExceeded,
}

impl History {
    /// Seed with the first page at cursor 0.
    pub(crate) fn seed(first_page: WizardPageDescriptor) -> Self {
        Self {
            entries: vec![HistoryEntry::new(first_page)],
            cursor: 0,
        }
    }

    pub(crate) fn current(&self) -> &HistoryEntry {
        &self.entries[self.cursor]
    }

    /// Prior entries only (`entries[0..cursor]`), exclusive of current (REQ-0024).
    pub(crate) fn prior_stack(&self) -> Vec<WizardStackEntry> {
        self.entries[..self.cursor]
            .iter()
            .map(HistoryEntry::to_stack_entry)
            .collect()
    }

    /// Full visited stack including current, with `current_data` replacing the
    /// in-memory current entry blob (finish derivation — session not mutated).
    pub(crate) fn visited_stack_with_current(
        &self,
        current_data: serde_json::Value,
    ) -> Vec<WizardStackEntry> {
        let mut stack = self.prior_stack();
        stack.push(WizardStackEntry {
            page: self.current().page.clone(),
            data: current_data,
        });
        stack
    }

    /// Whole-blob replace of the current entry's opaque data.
    pub(crate) fn write_current_data(&mut self, data: serde_json::Value) {
        self.entries[self.cursor].data = data;
    }

    /// Overwrite current data only when `data` is a meaningful payload.
    pub(crate) fn write_current_if_meaningful(&mut self, data: serde_json::Value) {
        if is_meaningful_payload(&data) {
            self.write_current_data(data);
        }
    }

    /// Advance forward (restore same page or truncate-then-push).
    ///
    /// Returns [`HistoryNavigateError::StackDepthExceeded`] when a branch push
    /// would exceed [`MAX_WIZARD_STACK_DEPTH`].
    /// Forward-same-page restore always succeeds (no growth).
    ///
    /// Caller must have already written the *current* entry's data.
    pub(crate) fn navigate_next(
        &mut self,
        next: WizardPageDescriptor,
        _data: serde_json::Value,
    ) -> Result<(), HistoryNavigateError> {
        let forward = self.cursor + 1;
        if forward < self.entries.len() && self.entries[forward].page == next {
            // Forward-same-page restore (ADR-0005). Request `data` was already
            // written to the outgoing current entry by `WizardSession::navigate_next`;
            // never overwrite the restored entry's cached blob.
            self.cursor = forward;
            return Ok(());
        }
        // Branch: truncate stale forward entries, then push — if under depth cap.
        if self.cursor + 1 >= MAX_WIZARD_STACK_DEPTH {
            return Err(HistoryNavigateError::StackDepthExceeded);
        }
        self.entries.truncate(self.cursor + 1);
        self.entries.push(HistoryEntry::new(next));
        self.cursor += 1;
        Ok(())
    }

    /// Move cursor back. Returns `false` when already at the first page.
    ///
    /// Applies meaningful-payload overwrite to the current entry before moving.
    pub(crate) fn navigate_back(&mut self, data: serde_json::Value) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.write_current_if_meaningful(data);
        self.cursor -= 1;
        true
    }
}

/// Forward-same-page / navigate_back overwrite predicate (d.2 normative).
pub(crate) fn is_meaningful_payload(data: &serde_json::Value) -> bool {
    match data {
        serde_json::Value::Null => false,
        serde_json::Value::Object(map) if map.is_empty() => false,
        serde_json::Value::Array(items) if items.is_empty() => false,
        serde_json::Value::String(s) if s.is_empty() => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{WizardPageHtml, WizardPageId, WizardPageTitle};

    fn page(id: &str, html: &str) -> WizardPageDescriptor {
        WizardPageDescriptor {
            id: WizardPageId::new(id),
            title: WizardPageTitle::new(id),
            html: WizardPageHtml::new(html),
            layout: None,
        }
    }

    #[test]
    fn meaningful_payload_predicate() {
        assert!(!is_meaningful_payload(&serde_json::Value::Null));
        assert!(!is_meaningful_payload(&serde_json::json!({})));
        assert!(!is_meaningful_payload(&serde_json::json!([])));
        assert!(!is_meaningful_payload(&serde_json::json!("")));
        assert!(is_meaningful_payload(&serde_json::json!({"a": 1})));
        assert!(is_meaningful_payload(&serde_json::json!([1])));
        assert!(is_meaningful_payload(&serde_json::json!("x")));
        assert!(is_meaningful_payload(&serde_json::json!(0)));
        assert!(is_meaningful_payload(&serde_json::json!(false)));
    }

    #[test]
    fn forward_same_page_restores_cached_data_even_with_meaningful_payload() {
        let mut h = History::seed(page("agent-1", "agent-1.html"));
        h.write_current_data(serde_json::json!({"description": "1/3"}));
        h.navigate_next(page("agent-2", "agent-2.html"), serde_json::json!({}))
            .expect("agent-2");
        h.write_current_data(serde_json::json!({"description": "2/3"}));
        assert!(h.navigate_back(serde_json::json!({})));
        assert_eq!(h.cursor, 0);
        // Re-commit agent-1, then forward-restore agent-2 — must not clobber 2/3.
        h.write_current_data(serde_json::json!({"description": "1/3"}));
        h.navigate_next(
            page("agent-2", "agent-2.html"),
            serde_json::json!({"description": "1/3"}),
        )
        .expect("restore agent-2");
        assert_eq!(h.cursor, 1);
        assert_eq!(h.current().data, serde_json::json!({"description": "2/3"}));
    }

    #[test]
    fn forward_same_page_restores_and_preserves_empty_overwrite() {
        let mut h = History::seed(page("a", "a.html"));
        h.write_current_data(serde_json::json!({"a": 1}));
        h.navigate_next(page("b", "b.html"), serde_json::json!({}))
            .expect("next b");
        h.write_current_data(serde_json::json!({"b": 2}));
        h.navigate_next(page("c", "c.html"), serde_json::json!({}))
            .expect("next c");
        assert_eq!(h.cursor, 2);
        assert!(h.navigate_back(serde_json::json!({})));
        assert!(h.navigate_back(serde_json::json!({})));
        assert_eq!(h.cursor, 0);
        // Forward same page restores cached B data; empty payload does not overwrite.
        h.navigate_next(page("b", "b.html"), serde_json::json!({}))
            .expect("restore b");
        assert_eq!(h.cursor, 1);
        assert_eq!(h.current().data, serde_json::json!({"b": 2}));
        assert_eq!(h.entries.len(), 3);
    }

    #[test]
    fn branch_truncates_forward() {
        let mut h = History::seed(page("a", "a.html"));
        h.navigate_next(page("b", "b.html"), serde_json::json!({}))
            .expect("b");
        h.navigate_next(page("c", "c.html"), serde_json::json!({}))
            .expect("c");
        assert!(h.navigate_back(serde_json::json!({})));
        assert!(h.navigate_back(serde_json::json!({})));
        h.navigate_next(page("d", "d.html"), serde_json::json!({}))
            .expect("branch");
        assert_eq!(h.cursor, 1);
        assert_eq!(h.entries.len(), 2);
        assert_eq!(h.current().page.id.as_str(), "d");
    }

    #[test]
    fn navigate_next_rejects_at_max_depth() {
        let mut h = History::seed(page("p0", "p0.html"));
        for i in 1..MAX_WIZARD_STACK_DEPTH {
            let id = format!("p{i}");
            let html = format!("p{i}.html");
            h.navigate_next(page(&id, &html), serde_json::json!({}))
                .unwrap_or_else(|_| panic!("push {i} should succeed"));
        }
        assert_eq!(h.entries.len(), MAX_WIZARD_STACK_DEPTH);
        let err = h.navigate_next(page("overflow", "overflow.html"), serde_json::json!({}));
        assert!(err.is_err());
        assert_eq!(h.entries.len(), MAX_WIZARD_STACK_DEPTH);
    }
}
