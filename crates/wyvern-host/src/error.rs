//! Host-level failures returned from [`crate::run`].

use std::fmt;
use std::path::PathBuf;

use crate::options::{BrowserId, ViewerMode};

/// Closed set of dialog `type` wire names known to the host matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogTypeName {
    /// `chrome` frame (on HTTP host matrix as of c.14).
    Chrome,
    /// `message` dialog.
    Message,
    /// `input` dialog.
    Input,
    /// `markdown` dialog.
    Markdown,
    /// `question` dialog.
    Question,
}

impl DialogTypeName {
    /// Wire name for errors and logging.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Message => "message",
            Self::Input => "input",
            Self::Markdown => "markdown",
            Self::Question => "question",
        }
    }
}

impl fmt::Display for DialogTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq<str> for DialogTypeName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for DialogTypeName {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Failure from the HTTP dialog host (bind, UI, type matrix, result parse).
#[derive(Debug)]
pub enum HostError {
    /// TCP bind failed (maps to stderr `host_bind` / exit 7).
    Bind {
        /// Human-readable bind failure detail.
        message: String,
        /// Underlying IO failure when bind / local_addr failed.
        source: Option<std::io::Error>,
    },
    /// UI root missing or required static file not found (maps to `host_error` / exit 6).
    UiNotFound {
        /// Path that was missing or unreadable.
        path: PathBuf,
        /// Underlying IO failure when canonicalize / filesystem access failed.
        source: Option<std::io::Error>,
    },
    /// Active command type not implemented on the host matrix yet (c.10–c.14).
    UnsupportedType {
        /// Dialog `type` wire name.
        type_name: DialogTypeName,
    },
    /// POST `/api/result` JSON invalid for the active type.
    InvalidResult {
        /// Parse / shape failure detail.
        message: String,
    },
    /// Named browser not installed (`HOST_VIEWER_ERROR`).
    ViewerNotFound {
        /// Catalog / registry id.
        id: BrowserId,
        /// Install or fallback hint.
        hint: String,
    },
    /// Viewer mode cannot be handled by the current API entrypoint.
    ///
    /// [`crate::run`] rejects [`ViewerMode::Embedded`] — use [`crate::begin`] instead.
    ViewerUnsupported {
        /// Requested viewer mode.
        mode: ViewerMode,
    },
    /// Internal server fault.
    Internal {
        /// Failure detail.
        message: String,
    },
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind { message, source } => match source {
                Some(err) => write!(f, "bind failed: {message}: {err}"),
                None => write!(f, "bind failed: {message}"),
            },
            Self::UiNotFound { path, source } => match source {
                Some(err) => write!(f, "UI not found: {}: {err}", path.display()),
                None => write!(f, "UI not found: {}", path.display()),
            },
            Self::UnsupportedType { type_name } => {
                write!(f, "unsupported dialog type: {type_name}")
            }
            Self::InvalidResult { message } => write!(f, "invalid result: {message}"),
            Self::ViewerNotFound { id, hint } => {
                write!(f, "viewer '{id}' not found; {hint}")
            }
            Self::ViewerUnsupported { mode } => {
                write!(
                    f,
                    "viewer mode '{}' is not supported by host::run (use begin + CLI spawn for embedded)",
                    mode.as_str()
                )
            }
            Self::Internal { message } => write!(f, "internal host error: {message}"),
        }
    }
}

impl std::error::Error for HostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bind {
                source: Some(err), ..
            }
            | Self::UiNotFound {
                source: Some(err), ..
            } => Some(err),
            _ => None,
        }
    }
}
