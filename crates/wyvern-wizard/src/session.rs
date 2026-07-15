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
    /// Invalid navigate/finish payload (d.2).
    InvalidCommand(String),
    /// Client finish stack ≠ session-derived stack (d.2).
    StackMismatch,
}

impl std::fmt::Display for WizardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtFirstPage => f.write_str("already at the first wizard page"),
            Self::InvalidCommand(msg) => write!(f, "invalid wizard command: {msg}"),
            Self::StackMismatch => f.write_str("client stack does not match session stack"),
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
    /// # Errors
    ///
    /// Returns [`WizardError::InvalidCommand`] when page identity fields are empty
    /// (defense in depth — schema validation should already reject these).
    pub fn new(command: &WizardCommand) -> Result<Self, WizardError> {
        if command.page.id.is_empty()
            || command.page.title.is_empty()
            || command.page.html.is_empty()
        {
            return Err(WizardError::InvalidCommand(
                "page.id, page.title, and page.html must be non-empty".into(),
            ));
        }
        Ok(Self {
            config: command.config.clone(),
            history: History::seed(command.page.clone()),
        })
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
    use wyvern_schema::WizardPageDescriptor;

    fn cmd(page_html: &str) -> WizardCommand {
        WizardCommand {
            page: WizardPageDescriptor {
                id: "start".into(),
                title: "Start".into(),
                html: page_html.into(),
                layout: None,
            },
            config: serde_json::json!({"theme": "dark"}),
            width: Some(640),
            height: Some(480),
        }
    }

    #[test]
    fn new_and_snapshot_first_page_empty_stack() {
        let session = WizardSession::new(&cmd("pages/start.html")).expect("new");
        let snap = session.snapshot();
        assert_eq!(snap.page.id, "start");
        assert_eq!(snap.page.html, "pages/start.html");
        assert_eq!(snap.page_data, serde_json::json!({}));
        assert!(snap.stack.is_empty());
        assert_eq!(snap.config["theme"], "dark");
    }

    #[test]
    fn new_rejects_empty_page_id() {
        let mut command = cmd("a.html");
        command.page.id.clear();
        let err = WizardSession::new(&command).expect_err("empty id");
        assert!(matches!(err, WizardError::InvalidCommand(_)));
    }
}
