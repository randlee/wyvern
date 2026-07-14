//! Wyvern HTTP dialog host — bind, serve packaged UI, await `POST /api/result`.
//!
//! Greenfield crate (sprint c.10). No wry/winit. One-shot `run()` serves a single
//! dialog session over loopback HTTP.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod error;
mod options;
mod picker;
mod routes;
mod server;
mod session;
mod static_files;

pub use error::{DialogTypeName, HostError};
pub use options::{
    BrowserId, HostOptions, ViewerLaunchOptions, ViewerMode, DEFAULT_SESSION_TIMEOUT,
};
pub use picker::MockPickerConfig;

use tokio::sync::oneshot;
use wyvern_schema::{Command, CommandResult};

use crate::server::{bind_server, publish_dialog_url, serve_until_result};
use crate::session::SessionState;

/// One-shot convenience for viewer modes the host owns (`None` in c.10).
///
/// Binds HTTP, optionally publishes the dialog URL (stderr / file), serves static UI + API,
/// and returns when the page POSTs `/api/result`.
///
/// # Errors
///
/// Returns [`HostError`] on unsupported type/viewer, bind/UI failures, or
/// internal server faults.
pub fn run(command: Command, options: HostOptions) -> Result<CommandResult, HostError> {
    match options.viewer {
        ViewerMode::None => {}
        other => {
            return Err(HostError::ViewerUnsupported { mode: other });
        }
    }

    let type_name = match &command {
        Command::Message { .. } => DialogTypeName::Message,
        Command::Input { .. } => DialogTypeName::Input,
        Command::Chrome { .. } => {
            return Err(HostError::UnsupportedType {
                type_name: DialogTypeName::Chrome,
            });
        }
        Command::Markdown { .. } => {
            return Err(HostError::UnsupportedType {
                type_name: DialogTypeName::Markdown,
            });
        }
        Command::Question { .. } => {
            return Err(HostError::UnsupportedType {
                type_name: DialogTypeName::Question,
            });
        }
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| HostError::Internal {
            message: format!("failed to create tokio runtime: {e}"),
        })?;

    rt.block_on(run_async(command, options, type_name))
}

async fn run_async(
    command: Command,
    options: HostOptions,
    type_name: DialogTypeName,
) -> Result<CommandResult, HostError> {
    let (result_tx, result_rx) = oneshot::channel();
    let session = SessionState::new(command, result_tx, options.mock_picker.clone());
    let (bound, ui_root) = bind_server(
        options.bind,
        options.allow_non_loopback,
        &options.ui_root,
        type_name.as_str(),
    )
    .await?;

    if options.dialog_url_env {
        publish_dialog_url(&bound.dialog_url, options.dialog_url_file.as_deref());
    }

    let state_session = session.clone();
    serve_until_result(
        bound,
        state_session,
        ui_root,
        result_rx,
        options.session_timeout,
    )
    .await
}
