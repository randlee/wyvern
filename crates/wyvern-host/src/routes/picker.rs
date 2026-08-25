//! `POST /api/picker/file` and `POST /api/picker/folder` — native `rfd` helpers.

use std::path::PathBuf;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};
use wyvern_schema::{
    Command, FilterPattern, FilterPatternError, InputMode, PickerPath, PickerPathError,
};

use crate::picker::{pick_file, pick_folder};
use crate::routes::api_error::ApiError;
use crate::session::{SessionState, PICKER_TIMEOUT};

/// Docs pointer for picker route errors (RBP error-context contract).
const PICKER_DOCS: &str = "docs/plans/phase-C/http-post-schema.md (POST /api/picker/*)";

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

/// Open a file picker for `input` mode `file` or an active wizard session.
pub async fn post_picker_file(
    State(session): State<SessionState>,
    Json(body): Json<PickerFileRequest>,
) -> Result<Json<PickerResponse>, ApiError> {
    if let Some(ref filter) = body.filter {
        for (i, pat) in filter.iter().enumerate() {
            FilterPattern::try_new(pat.as_str()).map_err(|err| filter_pattern_error(i, err))?;
        }
    }
    if let Some(ref start_path) = body.start_path {
        PickerPath::try_new(start_path.as_str()).map_err(picker_path_error)?;
    }

    let command = session.command().await;
    let (filter, multiple, start_path) = match command.as_ref() {
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
        Command::Wizard(_) => (
            body.filter.unwrap_or_default(),
            body.multiple.unwrap_or(false),
            body.start_path,
        ),
        Command::Input { mode, .. } => {
            return Err(picker_bad_request(
                format!(
                    "file picker requires input mode 'file', got '{}'",
                    mode.as_str()
                ),
                format!("active dialog mode is '{}'", mode.as_str()),
                "Open an input dialog with mode 'file' before calling POST /api/picker/file",
            ));
        }
        _ => {
            return Err(picker_bad_request(
                "file picker is only available for input dialogs and wizard sessions",
                "active dialog is not an input (mode: file) or wizard command",
                "Start an input (mode: file) or wizard dialog before calling POST /api/picker/file",
            ));
        }
    };

    let permit = session
        .acquire_picker_slot()
        .await
        .map_err(|_| picker_unavailable("dialog session closed while waiting for picker slot"))?;
    let mock = session.mock_picker().cloned();
    let start = start_path.as_ref().map(PathBuf::from);
    // Keep the permit in the async handler (not inside spawn_blocking) so an HTTP
    // timeout can drop it and unblock the next picker (RSH-002). The detached
    // blocking task may still finish after we return 504.
    let join = tokio::task::spawn_blocking(move || {
        pick_file(&filter, multiple, start.as_deref(), mock.as_ref())
    });
    let picked = match tokio::time::timeout(PICKER_TIMEOUT, join).await {
        Ok(Ok(paths)) => {
            drop(permit);
            paths
        }
        Ok(Err(e)) => {
            drop(permit);
            event!(
                name: "picker.file.error",
                Level::WARN,
                route = "/api/picker/file",
                error_class = "internal",
                error = %e,
                "file picker task failed"
            );
            return Err(picker_internal(format!("picker task failed: {e}")));
        }
        Err(_) => {
            drop(permit);
            event!(
                name: "picker.file.timeout",
                Level::WARN,
                route = "/api/picker/file",
                error_class = "timeout",
                "file picker timed out"
            );
            return Err(picker_timeout(format!(
                "file picker timed out after {} seconds",
                PICKER_TIMEOUT.as_secs()
            )));
        }
    };

    event!(
        name: "picker.file.ok",
        Level::DEBUG,
        route = "/api/picker/file",
        cancelled = picked.is_none(),
        "file picker completed"
    );
    Ok(Json(match picked {
        Some(paths) => PickerResponse::selected(paths_to_strings(paths)),
        None => PickerResponse::cancelled(),
    }))
}

/// Open a folder picker for `input` mode `folder` or an active wizard session.
pub async fn post_picker_folder(
    State(session): State<SessionState>,
    Json(body): Json<PickerFolderRequest>,
) -> Result<Json<PickerResponse>, ApiError> {
    if let Some(ref start_path) = body.start_path {
        PickerPath::try_new(start_path.as_str()).map_err(picker_path_error)?;
    }

    let command = session.command().await;
    let start_path = match command.as_ref() {
        Command::Input {
            mode: InputMode::Folder,
            start_path,
            ..
        } => body.start_path.or_else(|| start_path.clone()),
        Command::Wizard(_) => body.start_path,
        Command::Input { mode, .. } => {
            return Err(picker_bad_request(
                format!(
                    "folder picker requires input mode 'folder', got '{}'",
                    mode.as_str()
                ),
                format!("active dialog mode is '{}'", mode.as_str()),
                "Open an input dialog with mode 'folder' before calling POST /api/picker/folder",
            ));
        }
        _ => {
            return Err(picker_bad_request(
                "folder picker is only available for input dialogs and wizard sessions",
                "active dialog is not an input (mode: folder) or wizard command",
                "Start an input (mode: folder) or wizard dialog before calling POST /api/picker/folder",
            ));
        }
    };

    let permit = session
        .acquire_picker_slot()
        .await
        .map_err(|_| picker_unavailable("dialog session closed while waiting for picker slot"))?;
    let mock = session.mock_picker().cloned();
    let start = start_path.as_ref().map(PathBuf::from);
    let join = tokio::task::spawn_blocking(move || pick_folder(start.as_deref(), mock.as_ref()));
    let picked = match tokio::time::timeout(PICKER_TIMEOUT, join).await {
        Ok(Ok(path)) => {
            drop(permit);
            path
        }
        Ok(Err(e)) => {
            drop(permit);
            event!(
                name: "picker.folder.error",
                Level::WARN,
                route = "/api/picker/folder",
                error_class = "internal",
                error = %e,
                "folder picker task failed"
            );
            return Err(picker_internal(format!("picker task failed: {e}")));
        }
        Err(_) => {
            drop(permit);
            event!(
                name: "picker.folder.timeout",
                Level::WARN,
                route = "/api/picker/folder",
                error_class = "timeout",
                "folder picker timed out"
            );
            return Err(picker_timeout(format!(
                "folder picker timed out after {} seconds",
                PICKER_TIMEOUT.as_secs()
            )));
        }
    };

    event!(
        name: "picker.folder.ok",
        Level::DEBUG,
        route = "/api/picker/folder",
        cancelled = picked.is_none(),
        "folder picker completed"
    );
    Ok(Json(match picked {
        Some(path) => PickerResponse::selected(vec![path_to_string(path)]),
        None => PickerResponse::cancelled(),
    }))
}

fn filter_pattern_error(index: usize, err: FilterPatternError) -> ApiError {
    match err {
        FilterPatternError::Empty => picker_bad_request(
            format!("filter[{index}] must be a non-empty string"),
            format!("filter[{index}] was empty or whitespace-only"),
            "Pass extension patterns such as '*.json' or 'txt' (see dialog filter field)",
        ),
        FilterPatternError::ContainsNul => picker_bad_request(
            format!("filter[{index}] must not contain NUL bytes"),
            format!("filter[{index}] contained a NUL byte"),
            "Remove NUL bytes from filter patterns before POSTing /api/picker/file",
        ),
    }
}

fn picker_path_error(err: PickerPathError) -> ApiError {
    match err {
        PickerPathError::Empty => picker_bad_request(
            "start_path must be a non-empty string when provided",
            "start_path override was an empty string",
            "Omit start_path or pass a non-empty directory path",
        ),
        PickerPathError::ContainsNul => picker_bad_request(
            "start_path must not contain NUL bytes",
            "start_path override contained a NUL byte",
            "Remove NUL bytes from start_path before POSTing /api/picker/*",
        ),
    }
}

fn picker_bad_request(
    message: impl Into<String>,
    cause: impl Into<String>,
    recovery: &str,
) -> ApiError {
    let message = message.into();
    event!(
        name: "picker.route.bad_request",
        Level::WARN,
        error_class = "bad_request",
        error = %message,
        "picker route rejected request"
    );
    ApiError::bad_request(message)
        .cause(cause)
        .recovery(recovery)
        .docs(PICKER_DOCS)
}

fn picker_unavailable(message: impl Into<String>) -> ApiError {
    ApiError::service_unavailable(message)
        .cause("picker semaphore closed because the dialog session ended")
        .recovery("Keep the dialog session open until the picker returns")
        .recovery("Retry after starting a new input or wizard dialog")
        .docs(PICKER_DOCS)
}

fn picker_internal(message: impl Into<String>) -> ApiError {
    ApiError::internal(message)
        .cause("spawn_blocking picker task joined with an error")
        .recovery("Retry the picker request")
        .recovery("Report a bug if the failure persists without a native UI dialog")
        .docs(PICKER_DOCS)
}

fn picker_timeout(message: impl Into<String>) -> ApiError {
    ApiError::gateway_timeout(message)
        .cause("native or mock picker did not return within the session picker timeout")
        .recovery("Complete or cancel the native file dialog if it is still open")
        .recovery("Retry POST /api/picker/* after the previous picker finishes")
        .docs(PICKER_DOCS)
}

fn paths_to_strings(paths: Vec<PathBuf>) -> Vec<String> {
    paths.into_iter().map(path_to_string).collect()
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
