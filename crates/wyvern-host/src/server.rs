//! Axum router, bind, and serve loop for one-shot dialogs.

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::routing::{get, post};
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;

use crate::error::HostError;
use crate::routes::{dialog, picker, result};
use crate::session::SessionState;
use crate::static_files::require_type_dir;

/// Bound listener + dialog URL after successful TCP bind.
pub(crate) struct BoundServer {
    pub(crate) listener: TcpListener,
    pub(crate) dialog_url: String,
    pub(crate) local_addr: SocketAddr,
}

/// Bind TCP and build the dialog URL for `type_name`.
pub(crate) async fn bind_server(
    bind: SocketAddr,
    ui_root: &std::path::Path,
    type_name: &str,
) -> Result<(BoundServer, PathBuf), HostError> {
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
            local_addr,
        },
        ui_root,
    ))
}

/// Build the axum router for the one-shot session.
pub(crate) fn build_router(session: SessionState, ui_root: PathBuf) -> Router {
    let static_files = ServeDir::new(ui_root).append_index_html_on_directories(true);
    Router::new()
        .route("/api/dialog", get(dialog::get_dialog))
        .route("/api/result", post(result::post_result))
        .route("/api/picker/file", post(picker::post_picker_file))
        .route("/api/picker/folder", post(picker::post_picker_folder))
        .fallback_service(static_files)
        .with_state(session)
}
/// Serve until a result arrives on `result_rx`, then shut down.
pub(crate) async fn serve_until_result(
    bound: BoundServer,
    session: SessionState,
    ui_root: PathBuf,
    result_rx: oneshot::Receiver<wyvern_schema::CommandResult>,
) -> Result<wyvern_schema::CommandResult, HostError> {
    let app = build_router(session, ui_root);
    let server = axum::serve(bound.listener, app);
    tokio::select! {
        result = result_rx => {
            result.map_err(|_| HostError::Internal {
                message: "result channel closed without a value".into(),
            })
        }
        serve_result = server => {
            serve_result.map_err(|e| HostError::Internal {
                message: format!("HTTP server error: {e}"),
            })?;
            Err(HostError::Internal {
                message: "HTTP server exited before a result was posted".into(),
            })
        }
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
