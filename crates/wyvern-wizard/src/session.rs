//! [`WizardSession`] — concrete wizard stack state (ADR-0007).

use wyvern_schema::{
    ButtonLabel, WizardCommand, WizardPageDescriptor, WizardResult, WizardStackEntry,
};

use crate::history::History;

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
    StackMismatch,
    /// Host asked for a snapshot but no wizard session was initialized.
    NotInitialized,
}

impl WizardError {
    /// Construct a structured invalid-command error.
    pub fn invalid_command(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidCommand {
            field: field.into(),
            reason: reason.into(),
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
            Self::StackMismatch => f.write_str("client stack does not match session stack"),
            Self::NotInitialized => f.write_str("no wizard session for this dialog"),
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
    /// This method currently always returns `Ok`; `Result` is reserved for
    /// future validation hooks shared with the HTTP layer.
    pub fn navigate_next(
        &mut self,
        data: serde_json::Value,
        next: WizardPageDescriptor,
    ) -> Result<NavigateOutcome, WizardError> {
        let request_data = data.clone();
        // Opaque whole-blob replace on current before push/restore/advance.
        self.history.write_current_data(data);
        self.history.navigate_next(next, request_data);
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
    /// - [`WizardError::InvalidCommand`] for unknown `button` values
    /// - [`WizardError::StackMismatch`] when client `stack` ≠ session-derived
    ///   stack for `finish` / `dismissed`
    pub fn finish(
        &self,
        button: ButtonLabel,
        data: serde_json::Value,
        stack: Vec<WizardStackEntry>,
    ) -> Result<WizardResult, WizardError> {
        let label = button.as_str();
        match label {
            "cancel" => Ok(WizardResult {
                button,
                data: serde_json::json!({}),
                stack: Vec::new(),
            }),
            "finish" | "dismissed" => {
                let derived = self.history.visited_stack_with_current(data.clone());
                if stack != derived {
                    return Err(WizardError::StackMismatch);
                }
                let stdout_data = if label == "finish" {
                    data
                } else {
                    serde_json::json!({})
                };
                Ok(WizardResult {
                    button,
                    data: stdout_data,
                    stack: derived,
                })
            }
            _ => Err(WizardError::invalid_command(
                "button",
                format!("expected finish|cancel|dismissed, got '{label}'"),
            )),
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
            .finish(ButtonLabel::new("finish"), data.clone(), derived.clone())
            .expect("finish");
        assert_eq!(result.button.as_str(), "finish");
        assert_eq!(result.data, data);
        assert_eq!(result.stack, derived);

        let mismatch = session
            .finish(ButtonLabel::new("finish"), data, vec![])
            .expect_err("mismatch");
        assert_eq!(mismatch, WizardError::StackMismatch);

        let cancel = session
            .finish(
                ButtonLabel::new("cancel"),
                serde_json::json!({"ignored": true}),
                vec![],
            )
            .expect("cancel");
        assert_eq!(cancel.stack, Vec::<WizardStackEntry>::new());
        assert_eq!(cancel.data, serde_json::json!({}));
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
                ButtonLabel::new("dismissed"),
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
}
