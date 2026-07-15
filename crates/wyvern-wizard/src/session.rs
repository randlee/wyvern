//! [`WizardSession`] — concrete wizard stack state (ADR-0007).

use wyvern_schema::{WizardCommand, WizardPageDescriptor, WizardStackEntry};

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

/// Wizard session errors (navigate/finish land in d.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardError {
    /// `navigate_back` when already on the first page (d.2).
    AtFirstPage,
    /// Navigate/finish payload failed a field check (d.2).
    InvalidCommand {
        /// Dot-path of the offending field (e.g. `page.id`, `stack`).
        field: String,
        /// Human-readable failure detail.
        reason: String,
    },
    /// Client finish stack ≠ session-derived stack (d.2).
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

/// One wizard invocation — stack + cursor (d.1: `new` / `snapshot` only).
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{WizardPageHtml, WizardPageId, WizardPageTitle};

    fn cmd(page_html: &str) -> WizardCommand {
        WizardCommand {
            page: WizardPageDescriptor {
                id: WizardPageId::new("start"),
                title: WizardPageTitle::new("Start"),
                html: WizardPageHtml::new(page_html),
                layout: None,
            },
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
}
