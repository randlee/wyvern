//! Host-level failures returned from [`crate::run`].

use std::path::PathBuf;

/// Failure from the HTTP dialog host (bind, UI, type matrix, result parse).
#[derive(Debug)]
pub enum HostError {
    /// TCP bind failed (maps to stderr `host_bind` / exit 7).
    Bind {
        /// Human-readable bind failure detail.
        message: String,
    },
    /// UI root missing or required static file not found (maps to `host_error` / exit 6).
    UiNotFound {
        /// Path that was missing or unreadable.
        path: PathBuf,
    },
    /// Active command type not implemented on the host matrix yet (c.10–c.14).
    UnsupportedType {
        /// Dialog `type` wire name.
        type_name: String,
    },
    /// POST `/api/result` JSON invalid for the active type.
    InvalidResult {
        /// Parse / shape failure detail.
        message: String,
    },
    /// Named browser not installed (`HOST_VIEWER_ERROR`) — reserved for c.15.
    ViewerNotFound {
        /// Catalog / registry id.
        id: String,
        /// Install or fallback hint.
        hint: String,
    },
    /// Viewer mode not implemented yet (c.10: only `none`).
    ViewerUnsupported {
        /// Requested viewer mode name.
        mode: String,
    },
    /// Internal server fault.
    Internal {
        /// Failure detail.
        message: String,
    },
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bind { message } => write!(f, "bind failed: {message}"),
            Self::UiNotFound { path } => write!(f, "UI not found: {}", path.display()),
            Self::UnsupportedType { type_name } => {
                write!(f, "unsupported dialog type: {type_name}")
            }
            Self::InvalidResult { message } => write!(f, "invalid result: {message}"),
            Self::ViewerNotFound { id, hint } => {
                write!(f, "viewer '{id}' not found; {hint}")
            }
            Self::ViewerUnsupported { mode } => {
                write!(f, "viewer mode '{mode}' is not implemented yet")
            }
            Self::Internal { message } => write!(f, "internal host error: {message}"),
        }
    }
}

impl std::error::Error for HostError {}
