//! Axum router, bind, and serve loop for one-shot dialogs.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use axum::routing::{get, post};
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use wyvern_schema::{
    ButtonLabel, ChromeResult, Command, CommandResult, InputResult, MarkdownResult, MessageResult,
};

use crate::error::HostError;
use crate::routes::{dialog, picker, result};
use crate::session::SessionState;
use crate::static_files::require_type_dir;

/// Max JSON body size for `/api/*` routes (dialog payloads are small).
const API_BODY_LIMIT_BYTES: usize = 256 * 1024;

/// Bound listener + dialog URL after successful TCP bind.
pub(crate) struct BoundServer {
    pub(crate) listener: TcpListener,
    pub(crate) dialog_url: String,
}

/// Bind TCP and build the dialog URL for `type_name`.
pub(crate) async fn bind_server(
    bind: SocketAddr,
    allow_non_loopback: bool,
    ui_root: &std::path::Path,
    type_name: &str,
) -> Result<(BoundServer, PathBuf), HostError> {
    enforce_bind_policy(bind, allow_non_loopback)?;
    let ui_root = require_type_dir(ui_root, type_name)?;
    let listener = TcpListener::bind(bind).await.map_err(|e| HostError::Bind {
        message: format!("failed to bind {bind}: {e}"),
    })?;
    let local_addr = listener.local_addr().map_err(|e| HostError::Bind {
        message: format!("failed to read local address: {e}"),
    })?;
    let dialog_url = format!("http://{local_addr}/{type_name}/");
    Ok((
        BoundServer {
            listener,
            dialog_url,
        },
        ui_root,
    ))
}

/// Reject non-loopback binds unless explicitly opted in (ADR-0016 / http-dialog-contract).
fn enforce_bind_policy(bind: SocketAddr, allow_non_loopback: bool) -> Result<(), HostError> {
    if bind.ip().is_loopback() {
        return Ok(());
    }
    if !allow_non_loopback {
        return Err(HostError::Bind {
            message: format!(
                "refusing non-loopback bind {bind}; pass --allow-non-loopback (or HostOptions::allow_non_loopback) to opt in"
            ),
        });
    }
    tracing::warn!(
        bind = %bind,
        "binding dialog host to a non-loopback address; packaged UI and /api are reachable from other hosts"
    );
    Ok(())
}

/// Build the axum router for the one-shot session.
pub(crate) fn build_router(session: SessionState, ui_root: PathBuf) -> Router {
    let static_files = ServeDir::new(ui_root).append_index_html_on_directories(true);
    let api = Router::new()
        .route("/api/dialog", get(dialog::get_dialog))
        .route("/api/result", post(result::post_result))
        .route("/api/picker/file", post(picker::post_picker_file))
        .route("/api/picker/folder", post(picker::post_picker_folder))
        .layer(RequestBodyLimitLayer::new(API_BODY_LIMIT_BYTES));
    Router::new()
        .merge(api)
        .fallback_service(static_files)
        .with_state(session)
}

/// Serve until a result arrives, session timeout, or server failure.
pub(crate) async fn serve_until_result(
    bound: BoundServer,
    session: SessionState,
    ui_root: PathBuf,
    result_rx: oneshot::Receiver<CommandResult>,
    session_timeout: Duration,
) -> Result<CommandResult, HostError> {
    use std::future::IntoFuture;

    let app = build_router(session.clone(), ui_root);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let mut server = std::pin::pin!(axum::serve(bound.listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        })
        .into_future());
    let timeout = tokio::time::sleep(session_timeout);
    tokio::pin!(timeout);

    let outcome = tokio::select! {
        result = result_rx => {
            result.map_err(|_| HostError::Internal {
                message: "result channel closed without a value".into(),
            })
        }
        () = &mut timeout => {
            let command = session.command().await;
            Ok(dismissed_for_command(&command))
        }
        serve_result = &mut server => {
            serve_result.map_err(|e| HostError::Internal {
                message: format!("HTTP server error: {e}"),
            })?;
            return Err(HostError::Internal {
                message: "HTTP server exited before a result was posted".into(),
            });
        }
    };

    // Stop accepting connections and drain in-flight requests before returning.
    let _ = shutdown_tx.send(());
    if let Err(e) = server.await {
        if outcome.is_ok() {
            tracing::warn!(error = %e, "HTTP server error during graceful shutdown");
        } else {
            return Err(HostError::Internal {
                message: format!("HTTP server error: {e}"),
            });
        }
    }

    outcome
}

/// REQ-0097 dismissed shape for the active command type.
fn dismissed_for_command(command: &Command) -> CommandResult {
    match command {
        Command::Message { .. } => CommandResult::Message(MessageResult {
            button: ButtonLabel::dismissed(),
        }),
        Command::Input { .. } => CommandResult::Input(InputResult {
            button: ButtonLabel::dismissed(),
            input: None,
        }),
        Command::Markdown { .. } => CommandResult::Markdown(MarkdownResult {
            button: ButtonLabel::dismissed(),
        }),
        Command::Chrome { .. } => CommandResult::Chrome(ChromeResult {
            button: ButtonLabel::dismissed(),
        }),
        Command::Question { questions_raw, .. } => CommandResult::Question(
            wyvern_schema::QuestionResult::dismissed(questions_raw.clone()),
        ),
    }
}

/// Publish dialog URL for headless harnesses (`WYVERN_DIALOG_URL`).
pub(crate) fn publish_dialog_url(url: &str, dialog_url_file: Option<&std::path::Path>) {
    // Safety: one-shot CLI sets this before blocking; e2e reads via file/stderr.
    std::env::set_var("WYVERN_DIALOG_URL", url);
    let file_path = dialog_url_file
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("WYVERN_DIALOG_URL_FILE").map(std::path::PathBuf::from));
    if let Some(path) = file_path {
        if let Err(err) = std::fs::write(&path, url) {
            tracing::warn!(
                path = %path.display(),
                error = %err,
                "failed to write dialog URL file"
            );
        }
    }
    // Stable stderr line for harnesses that scrape without env/file (REQ-0092).
    eprintln!("WYVERN_DIALOG_URL={url}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn bind_policy_rejects_non_loopback_without_opt_in() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let err = enforce_bind_policy(addr, false).expect_err("policy");
        assert!(matches!(err, HostError::Bind { .. }));
    }

    #[test]
    fn bind_policy_allows_loopback() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        enforce_bind_policy(addr, false).expect("loopback ok");
    }
}
