//! Load/validation-stage errors and stderr JSON emission.

use wyvern_schema::ValidationError;

/// Failure while loading command input from argv or stdin.
#[derive(Debug)]
pub enum LoadError {
    /// JSON text could not be parsed.
    Parse { message: String },
    /// A file or stdin read failed.
    Io { field: String, message: String },
    /// Invalid argv shape; caller prints plain usage text (not JSON).
    Usage { message: String },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { message } => write!(f, "parse error: {message}"),
            Self::Io { field, message } => write!(f, "io error ({field}): {message}"),
            Self::Usage { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Serialize a parse/io load error as stderr JSON.
///
/// # Panics
///
/// Panics if `err` is [`LoadError::Usage`], which must be handled in `main`.
pub fn emit_load_error(err: &LoadError) -> String {
    match err {
        LoadError::Parse { message } => {
            serde_json::json!({ "error": "parse", "message": message }).to_string()
        }
        LoadError::Io { field, message } => {
            serde_json::json!({ "error": "io", "field": field, "message": message }).to_string()
        }
        LoadError::Usage { .. } => unreachable!("Usage handled in main"),
    }
}

/// Serialize a validation/state error as stderr JSON.
pub fn emit_validation_error(err: &ValidationError) -> String {
    match err {
        ValidationError::Validation { field, message } => {
            serde_json::json!({ "error": "validation", "field": field, "message": message })
                .to_string()
        }
        ValidationError::State { field, message } => {
            serde_json::json!({ "error": "state", "field": field, "message": message }).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_load_error_parse_with_quotes_is_valid_json() {
        let err = LoadError::Parse {
            message: r#"expected value at line 1: "bad""#.to_string(),
        };
        let out = emit_load_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "parse");
        assert!(value["message"].as_str().unwrap().contains('"'));
    }

    #[test]
    fn emit_load_error_io_with_quotes_is_valid_json() {
        let err = LoadError::Io {
            field: "file".to_string(),
            message: r#"could not read path 'say "hi".json'"#.to_string(),
        };
        let out = emit_load_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "io");
        assert_eq!(value["field"], "file");
        assert!(value["message"].as_str().unwrap().contains('"'));
    }

    #[test]
    fn emit_validation_error_message_with_quotes_is_valid_json() {
        let err = ValidationError::Validation {
            field: "title".to_string(),
            message: r#"field 'title' expected string, got "oops""#.to_string(),
        };
        let out = emit_validation_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "validation");
        assert_eq!(value["field"], "title");
        assert!(value["message"].as_str().unwrap().contains('"'));
    }

    #[test]
    fn emit_validation_error_state() {
        let err = ValidationError::State {
            field: "action".to_string(),
            message: "show is only valid in --interactive mode".to_string(),
        };
        let out = emit_validation_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "state");
        assert_eq!(value["field"], "action");
    }
}
