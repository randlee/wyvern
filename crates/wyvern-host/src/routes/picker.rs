//! `POST /api/picker/file` and `POST /api/picker/folder` — native `rfd` helpers.

use std::path::PathBuf;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use wyvern_schema::{Command, InputMode};

use crate::picker::{pick_file, pick_folder};
use crate::session::SessionState;

/// Request body for `POST /api/picker/file`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PickerFileRequest {
    /// Extension filters (merged with dialog fields when omitted).
    pub filter: Option<Vec<String>>,
    /// Multi-select (merged with dialog `multiple` when omitted).
    pub multiple: Option<bool>,
    /// Initial directory (merged with dialog `start_path` when omitted).
    pub start_path: Option<String>,
}

/// Request body for `POST /api/picker/folder`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PickerFolderRequest {
    /// Initial directory (merged with dialog `start_path` when omitted).
    pub start_path: Option<String>,
}

/// Response body for picker routes ([`PickerResponse`] in HTTP-TYPES.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PickerResponse {
    /// `true` when the user selected one or more paths.
    pub ok: bool,
    /// Selected absolute paths (omitted when cancelled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
    /// `true` when the user cancelled the native picker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
}

impl PickerResponse {
    fn selected(paths: Vec<String>) -> Self {
        Self {
            ok: true,
            paths: Some(paths),
            cancelled: None,
        }
    }

    fn cancelled() -> Self {
        Self {
            ok: false,
            paths: None,
            cancelled: Some(true),
        }
    }
}

/// Open a file picker; only valid while an `input` dialog with `mode: file` is active.
pub async fn post_picker_file(
    State(session): State<SessionState>,
    Json(body): Json<PickerFileRequest>,
) -> Result<Json<PickerResponse>, (StatusCode, String)> {
    let command = session.command().await;
    let (filter, multiple, start_path) = match &command {
        Command::Input {
            mode: InputMode::File,
            filter,
            multiple,
            start_path,
            ..
        } => {
            let filter = body
                .filter
                .unwrap_or_else(|| filter.clone().unwrap_or_default());
            let multiple = body.multiple.unwrap_or(*multiple);
            let start_path = body.start_path.or_else(|| start_path.clone());
            (filter, multiple, start_path)
        }
        Command::Input { mode, .. } => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "file picker requires input mode 'file', got '{}'",
                    mode.as_str()
                ),
            ));
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "file picker is only available for input dialogs".into(),
            ));
        }
    };

    let start = start_path.as_ref().map(PathBuf::from);
    let picked =
        tokio::task::spawn_blocking(move || pick_file(&filter, multiple, start.as_deref()))
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("picker task failed: {e}"),
                )
            })?;

    Ok(Json(match picked {
        Some(paths) => PickerResponse::selected(paths_to_strings(paths)),
        None => PickerResponse::cancelled(),
    }))
}

/// Open a folder picker; only valid while an `input` dialog with `mode: folder` is active.
pub async fn post_picker_folder(
    State(session): State<SessionState>,
    Json(body): Json<PickerFolderRequest>,
) -> Result<Json<PickerResponse>, (StatusCode, String)> {
    let command = session.command().await;
    let start_path = match &command {
        Command::Input {
            mode: InputMode::Folder,
            start_path,
            ..
        } => body.start_path.or_else(|| start_path.clone()),
        Command::Input { mode, .. } => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "folder picker requires input mode 'folder', got '{}'",
                    mode.as_str()
                ),
            ));
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "folder picker is only available for input dialogs".into(),
            ));
        }
    };

    let start = start_path.as_ref().map(PathBuf::from);
    let picked = tokio::task::spawn_blocking(move || pick_folder(start.as_deref()))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("picker task failed: {e}"),
            )
        })?;

    Ok(Json(match picked {
        Some(path) => PickerResponse::selected(vec![path_to_string(path)]),
        None => PickerResponse::cancelled(),
    }))
}

fn paths_to_strings(paths: Vec<PathBuf>) -> Vec<String> {
    paths.into_iter().map(path_to_string).collect()
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
