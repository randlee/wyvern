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

use tokio::sync::{oneshot, Mutex, Semaphore};
use wyvern_schema::Command;

/// Max time a native `rfd` picker may block a `spawn_blocking` worker.
pub(crate) const PICKER_TIMEOUT: Duration = Duration::from_secs(300);

/// Shared state for the active one-shot dialog.
#[derive(Clone)]
pub(crate) struct SessionState {
    inner: Arc<Mutex<SessionInner>>,
    /// Caps concurrent native pickers (one dialog → one picker at a time).
    picker_slots: Arc<Semaphore>,
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
            // Serialize picker spawn_blocking so repeated POSTs cannot exhaust
            // the blocking pool (RSH-006).
            picker_slots: Arc::new(Semaphore::new(1)),
        }
    }

    /// Clone of the active command for `/api/dialog`.
    pub(crate) async fn command(&self) -> Command {
        self.inner.lock().await.command.clone()
    }

    /// Acquire the single picker permit for this session.
    pub(crate) async fn acquire_picker_slot(
        &self,
    ) -> Result<tokio::sync::SemaphorePermit<'_>, HostSessionClosed> {
        self.picker_slots
            .acquire()
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

/// Session was dropped while waiting for a picker slot (should not happen in-process).
#[derive(Debug)]
pub(crate) struct HostSessionClosed;

impl std::fmt::Display for HostSessionClosed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("dialog session closed")
    }
}
