//! Stable machine-readable error codes for stderr JSON.
//!
//! # Stability
//!
//! Once published, [`ErrorCode`] variants must not be removed or renamed.
//! New variants may be added in a non-breaking way. Serde emits
//! `SCREAMING_SNAKE_CASE` strings (e.g. `PARSE_ERROR`).

use serde::{Deserialize, Serialize};

/// Stable error codes for scripting consumers of Wyvern stderr JSON.
///
/// These codes are **additive** alongside the historical `error` slug field
/// (`parse`, `validation`, …). Consumers that only check `error` remain valid;
/// new consumers should prefer `code` for stable branching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// JSON text could not be parsed (CLI load stage).
    ParseError,
    /// File or stdin read failed.
    IoError,
    /// Schema or field-level validation failure.
    ValidationError,
    /// Mode/lifecycle state failure (e.g. action outside `--interactive`).
    StateError,
    /// Native window or webview construction failed.
    WindowCreateError,
    /// Event loop creation or run failed.
    EventLoopError,
    /// Stdout/stderr JSON serialization failed at the CLI emit boundary (REQ-0078).
    InternalError,
}

impl ErrorCode {
    /// Stable process exit code for this failure category.
    pub fn exit_code(self) -> i32 {
        match self {
            Self::ParseError => 2,
            Self::IoError => 3,
            Self::ValidationError => 4,
            Self::StateError => 5,
            Self::WindowCreateError => 6,
            Self::EventLoopError => 7,
            Self::InternalError => 8,
        }
    }

    /// Wire slug historically emitted in the `error` field (REQ-0051–0073, REQ-0078).
    pub fn error_slug(self) -> &'static str {
        match self {
            Self::ParseError => "parse",
            Self::IoError => "io",
            Self::ValidationError => "validation",
            Self::StateError => "state",
            Self::WindowCreateError => "window_create",
            Self::EventLoopError => "event_loop",
            Self::InternalError => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_serde_is_screaming_snake() {
        let cases = [
            (ErrorCode::ParseError, "PARSE_ERROR"),
            (ErrorCode::IoError, "IO_ERROR"),
            (ErrorCode::ValidationError, "VALIDATION_ERROR"),
            (ErrorCode::StateError, "STATE_ERROR"),
            (ErrorCode::WindowCreateError, "WINDOW_CREATE_ERROR"),
            (ErrorCode::EventLoopError, "EVENT_LOOP_ERROR"),
            (ErrorCode::InternalError, "INTERNAL_ERROR"),
        ];
        for (code, expected) in cases {
            let json = serde_json::to_string(&code).expect("serialize");
            assert_eq!(json, format!("\"{expected}\""));
            let round: ErrorCode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(round, code);
        }
    }
}
