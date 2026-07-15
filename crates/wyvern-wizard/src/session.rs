//! [`WizardSession`] — concrete wizard stack state (ADR-0007).

use wyvern_schema::{
    WizardCommand, WizardPageDescriptor, WizardPageId, WizardResult, WizardStackEntry,
    WizardTerminalButton,
};

use crate::history::{History, MAX_WIZARD_STACK_DEPTH};

/// Snapshot for `GET /api/wizard/state` — prior entries only in `stack` (REQ-0024).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WizardSnapshot {
    /// Opaque wizard-wide config from the command.
    pub config: serde_json::Value,
    /// Current page descriptor.
    pub page: WizardPageDescriptor,
    /// Opaque data for the current page.
    pub page_data: serde_json::Value,
    /// Prior stack entries (`entries[0..cursor]`, exclusive of current).
    pub stack: Vec<WizardStackEntry>,
}

/// Host uses this to build navigate response URL + state refresh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigateOutcome {
    /// Destination page descriptor after navigation.
    pub page: WizardPageDescriptor,
    /// Opaque data for the destination page.
    pub page_data: serde_json::Value,
    /// Prior stack entries only (`entries[0..cursor]`).
    pub stack: Vec<WizardStackEntry>,
}

/// Wizard session errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardError {
    /// `navigate_back` when already on the first page.
    AtFirstPage,
    /// Navigate/finish payload failed a field check.
    InvalidCommand {
        /// Dot-path of the offending field (e.g. `page.id`, `stack`).
        field: String,
        /// Human-readable failure detail.
        reason: String,
    },
    /// Client finish stack ≠ session-derived stack.
    StackMismatch {
        /// First differing index, or `None` when lengths differ.
        index: Option<usize>,
        /// Expected page id at the mismatch (when applicable).
        expected_page_id: Option<WizardPageId>,
        /// Client page id at the mismatch (when applicable).
        got_page_id: Option<WizardPageId>,
        /// Human-readable summary of the diff.
        reason: String,
    },
    /// `navigate_next` would exceed the maximum wizard stack depth.
    StackDepthExceeded {
        /// Configured maximum entry count.
        max: usize,
    },
    /// Host asked for a snapshot/navigate but no wizard session was initialized.
    SessionNotInitialized,
    /// Navigate URL cannot be built because the public origin was not set after bind.
    PublicOriginNotSet,
    /// Navigate/finish after the one-shot result channel was already completed.
    ResultAlreadySubmitted,
}

impl WizardError {
    /// Construct a structured invalid-command error.
    pub fn invalid_command(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidCommand {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Stable machine-readable sub-code for CLI stderr mapping (RBP-F004).
    #[must_use]
    pub fn subcode(&self) -> &'static str {
        match self {
            Self::AtFirstPage => "WIZARD_AT_FIRST_PAGE",
            Self::InvalidCommand { .. } => "WIZARD_INVALID_COMMAND",
            Self::StackMismatch { .. } => "WIZARD_STACK_MISMATCH",
            Self::StackDepthExceeded { .. } => "WIZARD_STACK_DEPTH_EXCEEDED",
            Self::SessionNotInitialized => "WIZARD_SESSION_NOT_INITIALIZED",
            Self::PublicOriginNotSet => "WIZARD_PUBLIC_ORIGIN_NOT_SET",
            Self::ResultAlreadySubmitted => "WIZARD_RESULT_ALREADY_SUBMITTED",
        }
    }

    /// Compare client vs derived stacks and build a contextual mismatch error.
    pub(crate) fn stack_mismatch(
        client: &[WizardStackEntry],
        derived: &[WizardStackEntry],
    ) -> Self {
        if client.len() != derived.len() {
            return Self::StackMismatch {
                index: None,
                expected_page_id: None,
                got_page_id: None,
                reason: format!(
                    "length mismatch: expected {}, got {}",
                    derived.len(),
                    client.len()
                ),
            };
        }
        for (index, (got, expected)) in client.iter().zip(derived.iter()).enumerate() {
            if got.page != expected.page {
                return Self::StackMismatch {
                    index: Some(index),
                    expected_page_id: Some(expected.page.id.clone()),
                    got_page_id: Some(got.page.id.clone()),
                    reason: format!(
                        "page mismatch at index {index}: expected '{}', got '{}'",
                        expected.page.id.as_str(),
                        got.page.id.as_str()
                    ),
                };
            }
            if got.data != expected.data {
                return Self::StackMismatch {
                    index: Some(index),
                    expected_page_id: Some(expected.page.id.clone()),
                    got_page_id: Some(got.page.id.clone()),
                    reason: format!(
                        "data mismatch at index {index} (page '{}')",
                        expected.page.id.as_str()
                    ),
                };
            }
        }
        Self::StackMismatch {
            index: None,
            expected_page_id: None,
            got_page_id: None,
            reason: "client stack does not match session stack".to_string(),
        }
    }
}

impl std::fmt::Display for WizardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtFirstPage => f.write_str("already at the first wizard page"),
            Self::InvalidCommand { field, reason } => {
                write!(f, "invalid wizard command ({field}): {reason}")
            }
            Self::StackMismatch { reason, .. } => {
                write!(f, "client stack does not match session stack: {reason}")
            }
            Self::StackDepthExceeded { max } => {
                write!(f, "wizard stack depth would exceed maximum of {max}")
            }
            Self::SessionNotInitialized => f.write_str("no wizard session for this dialog"),
            Self::PublicOriginNotSet => f.write_str("wizard public origin not set after bind"),
            Self::ResultAlreadySubmitted => {
                f.write_str("wizard result already submitted for this session")
            }
        }
    }
}

impl std::error::Error for WizardError {}

/// One wizard invocation — stack + cursor (ADR-0005 / ADR-0007).
#[derive(Debug, Clone)]
pub struct WizardSession {
    config: serde_json::Value,
    history: History,
}

impl WizardSession {
    /// Create a session seeded with the command's first page at cursor 0.
    ///
    /// Page identity fields are validated newtypes from schema validation; this
    /// constructor does not re-check emptiness.
    pub fn new(command: &WizardCommand) -> Self {
        Self {
            config: command.config.clone(),
            history: History::seed(command.page.clone()),
        }
    }

    /// Current snapshot for `GET /api/wizard/state`.
    ///
    /// On the first page `stack` is empty (REQ-0024).
    pub fn snapshot(&self) -> WizardSnapshot {
        let current = self.history.current();
        WizardSnapshot {
            config: self.config.clone(),
            page: current.page.clone(),
            page_data: current.data.clone(),
            stack: self.history.prior_stack(),
        }
    }

    /// Advance to `next`, writing opaque `data` on the current entry first.
    ///
    /// Forward-same-page restore overwrites the destination's cached data only
    /// when `data` is a meaningful payload (d.2 predicate).
    ///
    /// # Errors
    ///
    /// Returns [`WizardError::StackDepthExceeded`] when a branch push would
    /// grow the history past the configured maximum stack depth.
    pub fn navigate_next(
        &mut self,
        data: serde_json::Value,
        next: WizardPageDescriptor,
    ) -> Result<NavigateOutcome, WizardError> {
        let request_data = data.clone();
        // Opaque whole-blob replace on current before push/restore/advance.
        self.history.write_current_data(data);
        self.history
            .navigate_next(next, request_data)
            .map_err(|()| WizardError::StackDepthExceeded {
                max: MAX_WIZARD_STACK_DEPTH,
            })?;
        Ok(self.outcome_from_current())
    }

    /// Move the cursor back without discarding forward entries.
    ///
    /// # Errors
    ///
    /// Returns [`WizardError::AtFirstPage`] when `cursor` is already 0.
    pub fn navigate_back(
        &mut self,
        data: serde_json::Value,
    ) -> Result<NavigateOutcome, WizardError> {
        if !self.history.navigate_back(data) {
            return Err(WizardError::AtFirstPage);
        }
        Ok(self.outcome_from_current())
    }

    /// Derive a terminal [`WizardResult`] without mutating the session.
    ///
    /// # Errors
    ///
    /// - [`WizardError::StackMismatch`] when client `stack` ≠ session-derived
    ///   stack for `finish` / `dismissed`
    pub fn finish(
        &self,
        button: WizardTerminalButton,
        data: serde_json::Value,
        stack: Vec<WizardStackEntry>,
    ) -> Result<WizardResult, WizardError> {
        match button {
            WizardTerminalButton::Cancel => Ok(WizardResult {
                button: button.to_button_label(),
                data: serde_json::json!({}),
                stack: Vec::new(),
            }),
            WizardTerminalButton::Finish | WizardTerminalButton::Dismissed => {
                let derived = self.history.visited_stack_with_current(data.clone());
                if stack != derived {
                    return Err(WizardError::stack_mismatch(&stack, &derived));
                }
                let stdout_data = if button == WizardTerminalButton::Finish {
                    data
                } else {
                    serde_json::json!({})
                };
                Ok(WizardResult {
                    button: button.to_button_label(),
                    data: stdout_data,
                    stack: derived,
                })
            }
        }
    }

    fn outcome_from_current(&self) -> NavigateOutcome {
        let current = self.history.current();
        NavigateOutcome {
            page: current.page.clone(),
            page_data: current.data.clone(),
            stack: self.history.prior_stack(),
        }
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

    fn cmd(page_html: &str) -> WizardCommand {
        WizardCommand {
            page: page("start", page_html),
            config: serde_json::json!({"theme": "dark"}),
            width: Some(640),
            height: Some(480),
        }
    }

    #[test]
    fn new_and_snapshot_first_page_empty_stack() {
        let session = WizardSession::new(&cmd("pages/start.html"));
        let snap = session.snapshot();
        assert_eq!(snap.page.id, "start");
        assert_eq!(snap.page.html, "pages/start.html");
        assert_eq!(snap.page_data, serde_json::json!({}));
        assert!(snap.stack.is_empty());
        assert_eq!(snap.config["theme"], "dark");
    }

    #[test]
    fn navigate_next_and_back_preserve_forward() {
        let mut session = WizardSession::new(&cmd("a.html"));
        let out = session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("next");
        assert_eq!(out.page.id, "b");
        assert_eq!(out.stack.len(), 1);
        assert_eq!(out.stack[0].data, serde_json::json!({"a": 1}));

        let out = session
            .navigate_next(serde_json::json!({"b": 2}), page("c", "c.html"))
            .expect("next");
        assert_eq!(out.page.id, "c");
        assert_eq!(out.stack.len(), 2);

        let out = session.navigate_back(serde_json::json!({})).expect("back");
        assert_eq!(out.page.id, "b");
        assert_eq!(out.page_data, serde_json::json!({"b": 2}));

        // Empty back payload preserves current before move; forward still intact.
        let out = session
            .navigate_next(serde_json::json!({}), page("c", "c.html"))
            .expect("forward restore");
        assert_eq!(out.page.id, "c");
        assert_eq!(out.stack.len(), 2);
    }

    #[test]
    fn navigate_back_at_first_page_errors() {
        let mut session = WizardSession::new(&cmd("a.html"));
        let err = session
            .navigate_back(serde_json::json!({}))
            .expect_err("at first");
        assert_eq!(err, WizardError::AtFirstPage);
    }

    #[test]
    fn navigate_back_empty_preserves_current_data() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("next");
        session
            .history
            .write_current_data(serde_json::json!({"b": "keep"}));
        session.navigate_back(serde_json::json!({})).expect("back");
        // Re-enter B — cached data preserved because back used {}.
        let out = session
            .navigate_next(serde_json::json!({}), page("b", "b.html"))
            .expect("restore b");
        assert_eq!(out.page_data, serde_json::json!({"b": "keep"}));
    }

    #[test]
    fn branch_truncates_stale_forward() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({}), page("b", "b.html"))
            .expect("b");
        session
            .navigate_next(serde_json::json!({}), page("c", "c.html"))
            .expect("c");
        session.navigate_back(serde_json::json!({})).expect("back");
        session.navigate_back(serde_json::json!({})).expect("back");
        let out = session
            .navigate_next(serde_json::json!({"a": 9}), page("d", "d.html"))
            .expect("branch");
        assert_eq!(out.page.id, "d");
        assert_eq!(out.stack.len(), 1);
        assert_eq!(out.stack[0].data, serde_json::json!({"a": 9}));
    }

    #[test]
    fn finish_validates_stack_and_cancel_clears() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("next");
        let data = serde_json::json!({"b": 2});
        let derived = vec![
            WizardStackEntry {
                page: page("start", "a.html"),
                data: serde_json::json!({"a": 1}),
            },
            WizardStackEntry {
                page: page("b", "b.html"),
                data: data.clone(),
            },
        ];
        let result = session
            .finish(WizardTerminalButton::Finish, data.clone(), derived.clone())
            .expect("finish");
        assert_eq!(result.button.as_str(), "finish");
        assert_eq!(result.data, data);
        assert_eq!(result.stack, derived);

        let mismatch = session
            .finish(WizardTerminalButton::Finish, data, vec![])
            .expect_err("mismatch");
        match mismatch {
            WizardError::StackMismatch { reason, index, .. } => {
                assert!(index.is_none());
                assert!(reason.contains("length mismatch"));
            }
            other => panic!("expected StackMismatch, got {other:?}"),
        }

        let cancel = session
            .finish(
                WizardTerminalButton::Cancel,
                serde_json::json!({"ignored": true}),
                vec![],
            )
            .expect("cancel");
        assert_eq!(cancel.stack, Vec::<WizardStackEntry>::new());
        assert_eq!(cancel.data, serde_json::json!({}));
    }

    #[test]
    fn finish_stack_mismatch_includes_page_id_diff() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("next");
        let bad = vec![
            WizardStackEntry {
                page: page("start", "a.html"),
                data: serde_json::json!({"a": 1}),
            },
            WizardStackEntry {
                page: page("wrong", "b.html"),
                data: serde_json::json!({"b": 2}),
            },
        ];
        let err = session
            .finish(
                WizardTerminalButton::Finish,
                serde_json::json!({"b": 2}),
                bad,
            )
            .expect_err("mismatch");
        match err {
            WizardError::StackMismatch {
                index,
                expected_page_id,
                got_page_id,
                reason,
            } => {
                assert_eq!(index, Some(1));
                assert_eq!(expected_page_id.as_deref(), Some("b"));
                assert_eq!(got_page_id.as_deref(), Some("wrong"));
                assert!(reason.contains("page mismatch at index 1"));
            }
            other => panic!("expected StackMismatch, got {other:?}"),
        }
    }

    #[test]
    fn finish_dismissed_uses_full_stack_empty_data() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({"a": 1}), page("b", "b.html"))
            .expect("next");
        let derived = vec![
            WizardStackEntry {
                page: page("start", "a.html"),
                data: serde_json::json!({"a": 1}),
            },
            WizardStackEntry {
                page: page("b", "b.html"),
                data: serde_json::json!({"b": 2}),
            },
        ];
        let result = session
            .finish(
                WizardTerminalButton::Dismissed,
                serde_json::json!({"b": 2}),
                derived.clone(),
            )
            .expect("dismissed");
        assert_eq!(result.data, serde_json::json!({}));
        assert_eq!(result.stack, derived);
    }

    #[test]
    fn forward_same_page_meaningful_overwrite() {
        let mut session = WizardSession::new(&cmd("a.html"));
        session
            .navigate_next(serde_json::json!({}), page("b", "b.html"))
            .expect("b");
        session
            .history
            .write_current_data(serde_json::json!({"cached": true}));
        session.navigate_back(serde_json::json!({})).expect("back");
        let out = session
            .navigate_next(serde_json::json!({"new": 1}), page("b", "b.html"))
            .expect("overwrite");
        // Request data was written to A; destination B overwritten because meaningful.
        assert_eq!(out.page_data, serde_json::json!({"new": 1}));
        assert_eq!(out.stack[0].data, serde_json::json!({"new": 1}));
    }

    #[test]
    fn navigate_next_rejects_at_max_depth() {
        let mut session = WizardSession::new(&cmd("p0.html"));
        for i in 1..MAX_WIZARD_STACK_DEPTH {
            let id = format!("p{i}");
            let html = format!("p{i}.html");
            session
                .navigate_next(serde_json::json!({}), page(&id, &html))
                .unwrap_or_else(|_| panic!("push {i}"));
        }
        let err = session
            .navigate_next(serde_json::json!({}), page("overflow", "overflow.html"))
            .expect_err("depth");
        assert_eq!(
            err,
            WizardError::StackDepthExceeded {
                max: MAX_WIZARD_STACK_DEPTH
            }
        );
    }
}
