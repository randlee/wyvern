//! One-shot dialog session state and result channel.

use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};
use wyvern_schema::Command;

/// Shared state for the active one-shot dialog.
#[derive(Clone)]
pub(crate) struct SessionState {
    inner: Arc<Mutex<SessionInner>>,
}

struct SessionInner {
    command: Command,
    result_tx: Option<oneshot::Sender<wyvern_schema::CommandResult>>,
}

impl SessionState {
    /// Create a session that will deliver the result once via `result_tx`.
    pub(crate) fn new(
        command: Command,
        result_tx: oneshot::Sender<wyvern_schema::CommandResult>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionInner {
                command,
                result_tx: Some(result_tx),
            })),
        }
    }

    /// Clone of the active command for `/api/dialog`.
    pub(crate) async fn command(&self) -> Command {
        self.inner.lock().await.command.clone()
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
