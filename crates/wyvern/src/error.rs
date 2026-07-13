//! Load/validation/run-stage errors and JSON emission helpers.

use wyvern_schema::{CommandResult, ValidationError};
use wyvern_window::RunError;

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

/// Serialize a window/run error as stderr JSON (`window_create` | `event_loop`).
pub fn emit_run_error(err: &RunError) -> String {
    match err {
        RunError::WindowCreate { message } => {
            serde_json::json!({ "error": "window_create", "message": message }).to_string()
        }
        RunError::EventLoop { message } => {
            serde_json::json!({ "error": "event_loop", "message": message }).to_string()
        }
    }
}

/// Serialize a successful [`CommandResult`] for stdout.
///
/// # Panics
///
/// Panics if `result` fails to serialize (should be impossible for schema types).
pub fn emit_stdout(result: &CommandResult) -> String {
    serde_json::to_string(result).expect("CommandResult serializes")
}

/// Map a run failure to stderr JSON plus a non-zero exit code.
pub fn handle_run_failure(err: &RunError) -> (String, i32) {
    (emit_run_error(err), 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::ChromeResult;

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

    #[test]
    fn emit_stdout_chrome_wire_shape() {
        let result = CommandResult::Chrome(ChromeResult {
            button: "dismissed".into(),
        });
        assert_eq!(emit_stdout(&result), r#"{"button":"dismissed"}"#);
    }

    #[test]
    fn emit_run_error_window_create() {
        let err = RunError::WindowCreate {
            message: r#"create failed: "boom""#.into(),
        };
        let out = emit_run_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "window_create");
        assert!(value["message"].as_str().unwrap().contains('"'));
    }

    #[test]
    fn emit_run_error_event_loop() {
        let err = RunError::EventLoop {
            message: "loop failed".into(),
        };
        let out = emit_run_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "event_loop");
        assert_eq!(value["message"], "loop failed");
    }

    #[test]
    fn handle_run_failure_maps_stderr_json_and_nonzero_exit() {
        let err = RunError::WindowCreate {
            message: "no display".into(),
        };
        let (json, code) = handle_run_failure(&err);
        assert_ne!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(value["error"], "window_create");
        assert_eq!(value["message"], "no display");
    }

    #[test]
    fn handle_run_failure_event_loop() {
        let err = RunError::EventLoop {
            message: "os error".into(),
        };
        let (json, code) = handle_run_failure(&err);
        assert_ne!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(value["error"], "event_loop");
        assert_eq!(value["message"], "os error");
    }
}
