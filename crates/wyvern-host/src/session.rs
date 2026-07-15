//! One-shot dialog session state and result channel.
//!
//! [`SessionState`] is cheaply cloned into every axum handler. Handlers need
//! shared access to the validated [`Command`] and a one-shot completion sender,
//! so the inner state lives behind `Arc<tokio::sync::Mutex<_>>`.
//!
//! - `Arc` clones across concurrent GET `/api/dialog`, POST `/api/result`, and
//!   picker routes without moving ownership of the command or channel.
//! - `Mutex` ensures `complete` takes the `oneshot::Sender` exactly once while
//!   serializing that take with reads of `command`. Splitting immutable command
//!   state from a completion flag would work but adds pieces for a one-shot
//!   session; the mutex keeps command and completion co-located.
//! - `tokio::sync::Mutex` fits async handlers (picker/result paths already await
//!   in this context and may grow awaits under the lock later).

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{oneshot, Mutex, OwnedSemaphorePermit, Semaphore};
use wyvern_schema::{
    Command, CommandResult, WizardPageDescriptor, WizardResult, WizardStackEntry,
    WizardTerminalButton,
};
use wyvern_wizard::{NavigateOutcome, WizardError, WizardSession, WizardSnapshot};

use crate::picker::MockPickerConfig;

/// Max time a native `rfd` picker may block a `spawn_blocking` worker.
pub(crate) const PICKER_TIMEOUT: Duration = Duration::from_secs(300);

/// Shared state for the active one-shot dialog.
#[derive(Clone)]
pub(crate) struct SessionState {
    inner: Arc<Mutex<SessionInner>>,
    /// Caps concurrent native pickers (one dialog → one picker at a time).
    picker_slots: Arc<Semaphore>,
    /// Optional in-process picker mock (tests); env mock remains for CLI/e2e.
    mock_picker: Option<MockPickerConfig>,
}

struct SessionInner {
    command: Command,
    /// Present when the active command is [`Command::Wizard`].
    wizard: Option<WizardSession>,
    /// Public origin (`http://127.0.0.1:PORT`) set after bind for navigate URLs.
    public_origin: Option<String>,
    result_tx: Option<oneshot::Sender<wyvern_schema::CommandResult>>,
}

impl SessionState {
    /// Create a session that will deliver the result once via `result_tx`.
    pub(crate) fn new(
        command: Command,
        result_tx: oneshot::Sender<wyvern_schema::CommandResult>,
        mock_picker: Option<MockPickerConfig>,
    ) -> Self {
        let wizard = match &command {
            Command::Wizard(wizard_cmd) => Some(WizardSession::new(wizard_cmd)),
            _ => None,
        };
        Self {
            inner: Arc::new(Mutex::new(SessionInner {
                command,
                wizard,
                public_origin: None,
                result_tx: Some(result_tx),
            })),
            // Serialize picker spawn_blocking so repeated POSTs cannot exhaust
            // the blocking pool (RSH-006).
            picker_slots: Arc::new(Semaphore::new(1)),
            mock_picker,
        }
    }

    /// Record the bound HTTP origin used to build absolute wizard page URLs.
    pub(crate) async fn set_public_origin(&self, origin: String) {
        self.inner.lock().await.public_origin = Some(origin);
    }

    /// Clone of the active command for `/api/dialog`.
    pub(crate) async fn command(&self) -> Command {
        self.inner.lock().await.command.clone()
    }

    /// Snapshot of the wizard session, if this is a wizard dialog.
    ///
    /// Clones the session under the mutex, then builds the snapshot after the
    /// lock is released so cloning config/page/stack does not extend hold time.
    pub(crate) async fn wizard_snapshot(&self) -> Result<WizardSnapshot, WizardError> {
        let session = {
            let guard = self.inner.lock().await;
            guard
                .wizard
                .clone()
                .ok_or(WizardError::SessionNotInitialized)?
        };
        Ok(session.snapshot())
    }

    /// Run `navigate_next` on the live wizard session.
    pub(crate) async fn wizard_navigate_next(
        &self,
        data: serde_json::Value,
        next: WizardPageDescriptor,
    ) -> Result<(NavigateOutcome, String), WizardError> {
        let mut guard = self.inner.lock().await;
        if guard.result_tx.is_none() {
            return Err(WizardError::ResultAlreadySubmitted);
        }
        let origin = guard
            .public_origin
            .clone()
            .ok_or(WizardError::PublicOriginNotSet)?;
        let session = guard
            .wizard
            .as_mut()
            .ok_or(WizardError::SessionNotInitialized)?;
        let outcome = session.navigate_next(data, next)?;
        let url = wizard_page_url(&origin, outcome.page.html.as_str());
        Ok((outcome, url))
    }

    /// Run `navigate_back` on the live wizard session.
    pub(crate) async fn wizard_navigate_back(
        &self,
        data: serde_json::Value,
    ) -> Result<(NavigateOutcome, String), WizardError> {
        let mut guard = self.inner.lock().await;
        if guard.result_tx.is_none() {
            return Err(WizardError::ResultAlreadySubmitted);
        }
        let origin = guard
            .public_origin
            .clone()
            .ok_or(WizardError::PublicOriginNotSet)?;
        let session = guard
            .wizard
            .as_mut()
            .ok_or(WizardError::SessionNotInitialized)?;
        let outcome = session.navigate_back(data)?;
        let url = wizard_page_url(&origin, outcome.page.html.as_str());
        Ok((outcome, url))
    }

    /// Run `finish` and complete the one-shot session with the derived result.
    ///
    /// Returns `Ok(None)` when a result was already submitted (HTTP 409).
    pub(crate) async fn wizard_finish(
        &self,
        button: WizardTerminalButton,
        data: serde_json::Value,
        stack: Vec<WizardStackEntry>,
    ) -> Result<Option<WizardResult>, WizardError> {
        let result = {
            let guard = self.inner.lock().await;
            let session = guard
                .wizard
                .as_ref()
                .ok_or(WizardError::SessionNotInitialized)?;
            session.finish(button, data, stack)?
        };
        if self.complete(CommandResult::Wizard(result.clone())).await {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// In-process mock picker config, if any.
    pub(crate) fn mock_picker(&self) -> Option<&MockPickerConfig> {
        self.mock_picker.as_ref()
    }

    /// Acquire the single picker permit for this session.
    ///
    /// Returns an [`OwnedSemaphorePermit`] so the caller can move it into
    /// `spawn_blocking` and hold it until the native (or mock) picker returns —
    /// including after an HTTP timeout drops the async handler.
    pub(crate) async fn acquire_picker_slot(
        &self,
    ) -> Result<OwnedSemaphorePermit, HostSessionClosed> {
        Arc::clone(&self.picker_slots)
            .acquire_owned()
            .await
            .map_err(|_| HostSessionClosed)
    }

    /// Deliver a validated result and close the channel (idempotent after first).
    pub(crate) async fn complete(&self, result: wyvern_schema::CommandResult) -> bool {
        let mut guard = self.inner.lock().await;
        if let Some(tx) = guard.result_tx.take() {
            let _ = tx.send(result);
            true
        } else {
            false
        }
    }
}

fn wizard_page_url(origin: &str, html: &str) -> String {
    let html = html.trim_start_matches('/');
    format!("{origin}/wizard/{html}")
}

/// Session was dropped while waiting for a picker slot (should not happen in-process).
#[derive(Debug)]
pub(crate) struct HostSessionClosed;

impl std::fmt::Display for HostSessionClosed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("dialog session closed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{
        WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
    };

    fn wizard_cmd() -> Command {
        Command::Wizard(WizardCommand {
            page: WizardPageDescriptor {
                id: WizardPageId::new("a"),
                title: WizardPageTitle::new("a"),
                html: WizardPageHtml::new("pages/a.html"),
                layout: None,
            },
            config: serde_json::json!({}),
            width: None,
            height: None,
        })
    }

    #[tokio::test]
    async fn navigate_after_result_submitted_returns_conflict_error() {
        let (tx, _rx) = oneshot::channel();
        let session = SessionState::new(wizard_cmd(), tx, None);
        session.set_public_origin("http://127.0.0.1:9".into()).await;

        assert!(
            session
                .complete(CommandResult::Wizard(WizardResult::dismissed()))
                .await
        );

        let err = session
            .wizard_navigate_next(
                serde_json::json!({}),
                WizardPageDescriptor {
                    id: WizardPageId::new("b"),
                    title: WizardPageTitle::new("b"),
                    html: WizardPageHtml::new("pages/b.html"),
                    layout: None,
                },
            )
            .await
            .expect_err("should reject post-complete navigate");
        assert_eq!(err, WizardError::ResultAlreadySubmitted);

        let err = session
            .wizard_navigate_back(serde_json::json!({}))
            .await
            .expect_err("back should also reject");
        assert_eq!(err, WizardError::ResultAlreadySubmitted);
    }
}
