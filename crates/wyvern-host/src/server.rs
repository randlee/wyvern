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
use wyvern_schema::{Command, CommandResult};

use crate::error::{DialogTypeName, HostError};
use crate::routes::{dialog, picker, result, wizard};
use crate::session::SessionState;
use crate::static_files::{require_shared_ui_root, require_type_dir, require_wizard_page};

/// Max JSON body size for `/api/*` routes (dialog payloads are small).
const API_BODY_LIMIT_BYTES: usize = 256 * 1024;

/// Per-request HTTP budget — above [`crate::session::PICKER_TIMEOUT`] so native
/// pickers are not cut off by the tower timeout layer (RSH-001).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(310);

/// Header name for request correlation (RSH-003).
const REQUEST_ID_HEADER: &str = "x-request-id";

/// Bound listener + dialog URL after successful TCP bind.
pub(crate) struct BoundServer {
    pub(crate) listener: TcpListener,
    pub(crate) dialog_url: String,
}

/// Resolved static roots for the router.
pub(crate) struct StaticRoots {
    pub(crate) ui_root: PathBuf,
    pub(crate) shared_ui_root: PathBuf,
    pub(crate) is_wizard: bool,
}

/// Bind TCP and build the dialog URL for the active command.
pub(crate) async fn bind_server(
    bind: SocketAddr,
    allow_non_loopback: bool,
    ui_root: &std::path::Path,
    shared_ui_root: &std::path::Path,
    command: &Command,
    type_name: DialogTypeName,
) -> Result<(BoundServer, StaticRoots), HostError> {
    enforce_bind_policy(bind, allow_non_loopback)?;
    let shared = require_shared_ui_root(shared_ui_root)?;
    let (ui_root, dialog_url_path, is_wizard) = match command {
        Command::Wizard(wizard_cmd) => {
            let root = require_wizard_page(ui_root, &wizard_cmd.page.html)?;
            let path = format!("/wizard/{}", wizard_cmd.page.html.trim_start_matches('/'));
            (root, path, true)
        }
        _ => {
            let root = require_type_dir(ui_root, type_name)?;
            let path = format!("/{}/", type_name.as_str());
            (root, path, false)
        }
    };
    let listener = TcpListener::bind(bind).await.map_err(|e| HostError::Bind {
        message: format!("failed to bind {bind}"),
        source: Some(e),
    })?;
    let local_addr = listener.local_addr().map_err(|e| HostError::Bind {
        message: "failed to read local address".into(),
        source: Some(e),
    })?;
    let dialog_url = format!("http://{local_addr}{dialog_url_path}");
    Ok((
        BoundServer {
            listener,
            dialog_url,
        },
        StaticRoots {
            ui_root,
            shared_ui_root: shared,
            is_wizard,
        },
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
            source: None,
        });
    }
    tracing::warn!(
        bind = %bind,
        "binding dialog host to a non-loopback address; packaged UI and /api are reachable from other hosts"
    );
    Ok(())
}

/// Build the axum router for the one-shot session.
pub(crate) fn build_router(session: SessionState, roots: StaticRoots) -> Router {
    use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
    use tower_http::timeout::TimeoutLayer;
    use tower_http::trace::TraceLayer;

    let shared_dir = ServeDir::new(roots.shared_ui_root.join("shared"));
    let api = Router::new()
        .route("/api/dialog", get(dialog::get_dialog))
        .route("/api/result", post(result::post_result))
        .route("/api/picker/file", post(picker::post_picker_file))
        .route("/api/picker/folder", post(picker::post_picker_folder))
        .route("/api/wizard/state", get(wizard::get_wizard_state))
        .route("/api/wizard/navigate", post(wizard::post_wizard_navigate))
        .route("/api/wizard/finish", post(wizard::post_wizard_finish))
        .layer(RequestBodyLimitLayer::new(API_BODY_LIMIT_BYTES));

    let app = if roots.is_wizard {
        let wizard_pages = ServeDir::new(roots.ui_root);
        Router::new()
            .merge(api)
            .nest_service("/wizard", wizard_pages)
            .nest_service("/shared", shared_dir)
            .with_state(session)
    } else {
        let static_files = ServeDir::new(roots.ui_root).append_index_html_on_directories(true);
        // Dual-mount `/shared` from packaged root so `--ui-root` overrides cannot
        // hide `wyvern-api.js` (wizard contract; harmless for blocking dialogs).
        Router::new()
            .merge(api)
            .nest_service("/shared", shared_dir)
            .fallback_service(static_files)
            .with_state(session)
    };

    app.layer(TimeoutLayer::with_status_code(
        axum::http::StatusCode::GATEWAY_TIMEOUT,
        REQUEST_TIMEOUT,
    ))
    .layer(TraceLayer::new_for_http())
    .layer(PropagateRequestIdLayer::new(
        axum::http::HeaderName::from_static(REQUEST_ID_HEADER),
    ))
    .layer(SetRequestIdLayer::new(
        axum::http::HeaderName::from_static(REQUEST_ID_HEADER),
        MakeRequestUuid,
    ))
}

/// Serve until a result arrives, session timeout, viewer-exit signal, or server failure.
pub(crate) async fn serve_until_result(
    bound: BoundServer,
    session: SessionState,
    roots: StaticRoots,
    result_rx: oneshot::Receiver<CommandResult>,
    session_timeout: Duration,
    mut dismiss_rx: oneshot::Receiver<()>,
) -> Result<CommandResult, HostError> {
    use std::future::IntoFuture;

    let app = build_router(session.clone(), roots);
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
            Ok(session.dismissed_on_exit_or_timeout().await)
        }
        _ = &mut dismiss_rx => {
            Ok(session.dismissed_on_exit_or_timeout().await)
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

/// Publish dialog URL for headless harnesses (stderr + optional file).
///
/// Does **not** call [`std::env::set_var`]: by the time this runs, the multi-thread
/// Tokio runtime is already live, and mutating process env is unsound. Harnesses
/// scrape `WYVERN_DIALOG_URL=…` from stderr or read [`HostOptions::dialog_url_file`].
pub(crate) fn publish_dialog_url(url: &str, dialog_url_file: Option<&std::path::Path>) {
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
