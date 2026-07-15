//! Two-phase dialog handoff (`begin` → launch → `await_result`).

use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use tokio::sync::oneshot;
use wyvern_schema::Command;

use crate::error::{DialogTypeName, HostError};
use crate::options::{HostOptions, ViewerLaunchOptions, ViewerMode};
use crate::server::{bind_server, publish_dialog_url, serve_until_result};
use crate::session::SessionState;
use wyvern_schema::CommandResult;

/// Two-phase handoff after bind — caller launches a viewer, then awaits the result.
#[derive(Debug)]
pub struct DialogHandle {
    /// Full dialog URL, e.g. `http://127.0.0.1:PORT/message/`.
    pub dialog_url: String,
    /// Optional window hints for an embedded viewer.
    pub viewer_options: ViewerLaunchOptions,
    dismiss_tx: Option<oneshot::Sender<()>>,
    done_rx: mpsc::Receiver<Result<CommandResult, HostError>>,
    join: Option<JoinHandle<()>>,
}

impl DialogHandle {
    /// Block until `POST /api/result` (or dismiss timeout / viewer-exit signal).
    ///
    /// # Errors
    ///
    /// Returns [`HostError`] when the serve loop fails or the worker panics.
    pub fn await_result(mut self) -> Result<CommandResult, HostError> {
        let result = self.done_rx.recv().map_err(|_| HostError::Internal {
            message: "dialog worker closed without a result".into(),
        })?;
        self.join_worker();
        result
    }

    /// Non-blocking poll for dialog completion (macOS picker pump loops).
    pub fn try_recv_result(&mut self) -> Option<Result<CommandResult, HostError>> {
        self.done_rx.try_recv().ok()
    }

    /// Join the background host worker after [`Self::try_recv_result`] returns.
    pub fn join_host_worker(&mut self) {
        self.join_worker();
    }

    /// CLI fallback when `wyvern-viewer` exits without posting a result (REQ-0097).
    ///
    /// # Errors
    ///
    /// Returns [`HostError`] when the serve loop fails or the worker panics.
    pub fn viewer_exited_without_result(mut self) -> Result<CommandResult, HostError> {
        if let Some(tx) = self.dismiss_tx.take() {
            let _ = tx.send(());
        }
        self.await_result()
    }

    /// Take the oneshot used to signal viewer process exit (CLI child watcher).
    ///
    /// Sending on this channel is equivalent to [`Self::viewer_exited_without_result`]
    /// without consuming the handle — pair with [`Self::await_result`].
    pub fn take_viewer_exit_signal(&mut self) -> Option<oneshot::Sender<()>> {
        self.dismiss_tx.take()
    }

    fn join_worker(&mut self) {
        if let Some(join) = self.join.take() {
            if let Err(err) = join.join() {
                tracing::warn!(?err, "dialog worker thread panicked");
            }
        }
    }
}

impl Drop for DialogHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.dismiss_tx.take() {
            let _ = tx.send(());
        }
        // Drain so the worker can shut down if the handle is dropped early.
        while self.done_rx.try_recv().is_ok() {}
        self.join_worker();
    }
}

/// Bind and serve a dialog; return a handle before external viewer launch.
///
/// Required for [`ViewerMode::Embedded`] (CLI spawns `wyvern-viewer` between
/// `begin` and [`DialogHandle::await_result`]). Also valid for other modes when
/// the caller wants explicit two-phase control.
///
/// # Errors
///
/// Returns [`HostError`] on bind/UI failures or runtime creation errors.
pub fn begin(command: Command, options: HostOptions) -> Result<DialogHandle, HostError> {
    options.validate()?;
    let type_name = dialog_type_name(&command);
    let viewer_options = ViewerLaunchOptions::from_command(&command);
    let (ready_tx, ready_rx) = mpsc::channel::<Result<ReadyPayload, HostError>>();
    let (done_tx, done_rx) = mpsc::channel::<Result<CommandResult, HostError>>();

    let join = thread::Builder::new()
        .name("wyvern-host-dialog".into())
        .spawn(move || {
            let outcome = run_begin_worker(command, options, type_name, ready_tx);
            let _ = done_tx.send(outcome);
        })
        .map_err(|e| HostError::Internal {
            message: format!("failed to spawn dialog worker: {e}"),
        })?;

    let ready = ready_rx.recv().map_err(|_| HostError::Internal {
        message: "dialog worker exited before bind".into(),
    })??;

    Ok(DialogHandle {
        dialog_url: ready.dialog_url,
        viewer_options,
        dismiss_tx: Some(ready.dismiss_tx),
        done_rx,
        join: Some(join),
    })
}

struct ReadyPayload {
    dialog_url: String,
    dismiss_tx: oneshot::Sender<()>,
}

fn run_begin_worker(
    command: Command,
    options: HostOptions,
    type_name: DialogTypeName,
    ready_tx: mpsc::Sender<Result<ReadyPayload, HostError>>,
) -> Result<CommandResult, HostError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| HostError::Internal {
            message: format!("failed to create tokio runtime: {e}"),
        })?;

    rt.block_on(async move {
        let (result_tx, result_rx) = oneshot::channel();
        let (dismiss_tx, dismiss_rx) = oneshot::channel();
        let session =
            match SessionState::new(command.clone(), result_tx, options.mock_picker.clone()) {
                Ok(s) => s,
                Err(message) => {
                    let err = HostError::Internal { message };
                    let _ = ready_tx.send(Err(clone_host_error_message(&err)));
                    return Err(err);
                }
            };
        let (bound, roots) = match bind_server(
            options.bind,
            options.allow_non_loopback,
            &options.ui_root,
            &options.shared_ui_root,
            &command,
            type_name,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                let _ = ready_tx.send(Err(clone_host_error_message(&e)));
                return Err(e);
            }
        };

        let dialog_url = bound.dialog_url.clone();
        if options.dialog_url_env {
            publish_dialog_url(&dialog_url, options.dialog_url_file.as_deref());
        }

        if ready_tx
            .send(Ok(ReadyPayload {
                dialog_url: dialog_url.clone(),
                dismiss_tx,
            }))
            .is_err()
        {
            return Err(HostError::Internal {
                message: "caller dropped DialogHandle before ready".into(),
            });
        }

        // System/named launch is owned by `run()`, not `begin()` — caller decides.

        serve_until_result(
            bound,
            session,
            roots,
            result_rx,
            options.session_timeout,
            dismiss_rx,
        )
        .await
    })
}

/// One-shot convenience for viewer modes the host owns (`none` / `system` / `named`).
///
/// **Must not** be used for [`ViewerMode::Embedded`] — use [`begin`] + CLI spawn.
pub(crate) async fn run_owned_async(
    command: Command,
    options: HostOptions,
    type_name: DialogTypeName,
) -> Result<CommandResult, HostError> {
    let (result_tx, result_rx) = oneshot::channel();
    let (_dismiss_tx, dismiss_rx) = oneshot::channel();
    let session = SessionState::new(command.clone(), result_tx, options.mock_picker.clone())
        .map_err(|message| HostError::Internal { message })?;
    let (bound, roots) = bind_server(
        options.bind,
        options.allow_non_loopback,
        &options.ui_root,
        &options.shared_ui_root,
        &command,
        type_name,
    )
    .await?;

    if options.dialog_url_env {
        publish_dialog_url(&bound.dialog_url, options.dialog_url_file.as_deref());
    }

    match &options.viewer {
        ViewerMode::None => {}
        ViewerMode::System | ViewerMode::Named(_) => {
            crate::browser_launch::launch(&options.viewer, &bound.dialog_url)?;
        }
        ViewerMode::Embedded => {
            return Err(HostError::ViewerUnsupported {
                mode: ViewerMode::Embedded,
            });
        }
    }

    serve_until_result(
        bound,
        session,
        roots,
        result_rx,
        options.session_timeout,
        dismiss_rx,
    )
    .await
}

pub(crate) fn dialog_type_name(command: &Command) -> DialogTypeName {
    match command {
        Command::Chrome { .. } => DialogTypeName::Chrome,
        Command::Message { .. } => DialogTypeName::Message,
        Command::Input { .. } => DialogTypeName::Input,
        Command::Markdown { .. } => DialogTypeName::Markdown,
        Command::Question { .. } => DialogTypeName::Question,
        Command::Wizard(_) => DialogTypeName::Wizard,
    }
}

/// HostError is not Clone; for ready-channel failures re-wrap the display text.
fn clone_host_error_message(err: &HostError) -> HostError {
    match err {
        HostError::Bind { message, .. } => HostError::Bind {
            message: message.clone(),
            source: None,
        },
        HostError::UiNotFound { path, .. } => HostError::UiNotFound {
            path: path.clone(),
            source: None,
        },
        HostError::UnsupportedType { type_name } => HostError::UnsupportedType {
            type_name: *type_name,
        },
        HostError::InvalidResult { message } => HostError::InvalidResult {
            message: message.clone(),
        },
        HostError::ViewerNotFound { id, hint } => HostError::ViewerNotFound {
            id: *id,
            hint: hint.clone(),
        },
        HostError::ViewerUnsupported { mode } => {
            HostError::ViewerUnsupported { mode: mode.clone() }
        }
        HostError::Registry { message } => HostError::Registry {
            message: message.clone(),
        },
        HostError::Internal { message } => HostError::Internal {
            message: message.clone(),
        },
    }
}
