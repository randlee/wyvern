//! Shared stderr JSON envelope for load/validation/run failures.
//!
//! Extends the REQ-0051–0073 wire shape (`error`, optional `field`, `message`)
//! with stable [`ErrorCode`] plus recovery-oriented fields (`cause`, `recovery`,
//! `docs`). Empty optional fields are omitted via `skip_serializing_if`.

use serde::Serialize;

use crate::error_code::ErrorCode;
use crate::field_name::FieldName;

/// Failure serializing a [`StderrError`] (or similar) to JSON.
#[derive(Debug)]
pub struct SerializeError {
    /// Human-readable serialization failure detail.
    pub message: String,
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "serialize error: {}", self.message)
    }
}

impl std::error::Error for SerializeError {}

/// Structured stderr JSON payload emitted by the CLI on failure.
///
/// # Wire shape
///
/// ```json
/// {
///   "error": "validation",
///   "code": "VALIDATION_ERROR",
///   "field": "title",
///   "message": "missing required field 'title'",
///   "cause": "...",
///   "recovery": ["..."],
///   "docs": "docs/wyvern-schema/requirements.md"
/// }
/// ```
///
/// The `error` slug is preserved for existing tests and REQ-0051–0073.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StderrError {
    /// Historical error slug (`parse`, `io`, `validation`, …).
    pub error: &'static str,
    /// Stable machine code (SCREAMING_SNAKE_CASE).
    pub code: ErrorCode,
    /// Field path when applicable (`title`, `file`, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<FieldName>,
    /// Human-readable failure message.
    pub message: String,
    /// Why the failure occurred (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// Actionable recovery steps.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recovery: Vec<String>,
    /// Repo-relative docs or requirements reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

#[cfg(test)]
static FORCE_SERIALIZE_FAIL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

impl StderrError {
    /// Start a stderr envelope for `code` with the historical slug and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: code.error_slug(),
            code,
            field: None,
            message: message.into(),
            cause: None,
            recovery: Vec::new(),
            docs: None,
        }
    }

    /// Attach a field path.
    pub fn field(mut self, field: impl Into<FieldName>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Attach a cause string.
    pub fn cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    /// Append one recovery step.
    pub fn recovery(mut self, step: impl Into<String>) -> Self {
        self.recovery.push(step.into());
        self
    }

    /// Attach a docs reference.
    pub fn docs(mut self, docs: impl Into<String>) -> Self {
        self.docs = Some(docs.into());
        self
    }

    /// Serialize to a JSON string for stderr.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError`] when `serde_json` cannot serialize this envelope.
    pub fn to_json_string(&self) -> Result<String, SerializeError> {
        #[cfg(test)]
        if FORCE_SERIALIZE_FAIL.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(SerializeError {
                message: "forced".into(),
            });
        }
        serde_json::to_string(self).map_err(|e| SerializeError {
            message: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn omits_empty_optional_fields() {
        let err = StderrError::new(ErrorCode::ParseError, "bad json")
            .cause("trailing comma")
            .recovery("Ensure input is valid JSON");
        let value: serde_json::Value =
            serde_json::from_str(&err.to_json_string().expect("serialize")).expect("valid JSON");
        assert_eq!(value["error"], "parse");
        assert_eq!(value["code"], "PARSE_ERROR");
        assert!(value.get("field").is_none());
        assert!(value.get("docs").is_none());
        assert_eq!(value["cause"], "trailing comma");
        assert_eq!(
            value["recovery"],
            serde_json::json!(["Ensure input is valid JSON"])
        );
    }

    #[test]
    fn serialize_error_forced_fail() {
        FORCE_SERIALIZE_FAIL.store(true, Ordering::Relaxed);
        let err = StderrError::new(ErrorCode::ParseError, "x");
        assert!(err.to_json_string().is_err());
        FORCE_SERIALIZE_FAIL.store(false, Ordering::Relaxed);
    }
}
