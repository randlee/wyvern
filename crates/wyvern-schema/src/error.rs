//! Validation-stage errors for schema checking.

use crate::error_code::ErrorCode;
use crate::field_name::FieldName;

/// Failure while validating command JSON against the phase surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Schema or field-level validation failure.
    Validation { field: FieldName, message: String },
    /// Mode/lifecycle state failure (e.g. action outside `--interactive`).
    State { field: FieldName, message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation { field, message } => {
                write!(f, "validation error ({field}): {message}")
            }
            Self::State { field, message } => write!(f, "state error ({field}): {message}"),
        }
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    /// Build a schema validation error for `field`.
    pub(crate) fn validation(field: impl Into<FieldName>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Build a state error for `field`.
    pub(crate) fn state(field: impl Into<FieldName>, message: impl Into<String>) -> Self {
        Self::State {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Stable exit code for this validation failure.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation { .. } => ErrorCode::ValidationError.exit_code(),
            Self::State { .. } => ErrorCode::StateError.exit_code(),
        }
    }
}
