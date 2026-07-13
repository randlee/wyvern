//! Load/validation/run-stage errors and JSON emission helpers.

use wyvern_schema::{ErrorCode, StderrError, ValidationError};
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
        LoadError::Parse { message } => StderrError::new(ErrorCode::ParseError, message.clone())
            .cause("Input was not valid JSON")
            .recovery("Ensure input is valid JSON")
            .recovery("Check for trailing commas, unquoted keys, or truncated input")
            .docs("docs/wyvern-schema/requirements.md (REQ-0069)")
            .to_json_string(),
        LoadError::Io { field, message } => StderrError::new(ErrorCode::IoError, message.clone())
            .field(field.clone())
            .cause(format!("Failed to read input from '{field}'"))
            .recovery("Verify the file path exists and is readable")
            .recovery("Pass JSON inline as an argv string or via stdin")
            .docs("docs/wyvern-schema/requirements.md (REQ-0071)")
            .to_json_string(),
        LoadError::Usage { .. } => unreachable!("Usage handled in main"),
    }
}

/// Serialize a validation/state error as stderr JSON.
pub fn emit_validation_error(err: &ValidationError) -> String {
    match err {
        ValidationError::Validation { field, message } => {
            let mut envelope = StderrError::new(ErrorCode::ValidationError, message.clone())
                .field(field.clone())
                .cause(format!("Command JSON failed schema checks on '{field}'"))
                .docs("docs/wyvern-schema/requirements.md (REQ-0051, REQ-0070)");
            for step in validation_recovery(field, message) {
                envelope = envelope.recovery(step);
            }
            envelope.to_json_string()
        }
        ValidationError::State { field, message } => {
            StderrError::new(ErrorCode::StateError, message.clone())
                .field(field.clone())
                .cause("Lifecycle action used outside interactive mode")
                .recovery("Run with --interactive to use lifecycle actions (show/hide/exit)")
                .recovery("Omit the action field for one-shot chrome commands")
                .docs("docs/wyvern-schema/requirements.md (REQ-0072)")
                .to_json_string()
        }
    }
}

fn validation_recovery(field: &str, message: &str) -> Vec<String> {
    if field == "title" && message.contains("missing required field") {
        return vec![
            "Add required field \"title\" with a string value".into(),
            "Example: {\"type\":\"chrome\",\"title\":\"Foundation\"}".into(),
        ];
    }
    if field == "type" && message.contains("missing required field") {
        return vec![
            "Add required field \"type\" with value \"chrome\"".into(),
            "Example: {\"type\":\"chrome\",\"title\":\"Foundation\"}".into(),
        ];
    }
    if field == "type" && message.contains("expected one of") {
        return vec![
            "Set \"type\" to \"chrome\" (Phase A executable surface)".into(),
            "Other dialog types ship in later phases".into(),
        ];
    }
    if message.contains("expected string") {
        return vec![format!("Provide field \"{field}\" as a JSON string")];
    }
    if message.contains("unknown field") {
        return vec![format!(
            "Remove unknown field \"{field}\"; chrome allows only type, title, and status"
        )];
    }
    if message.contains("expected JSON object") {
        return vec!["Pass a single JSON object as the command payload".into()];
    }
    vec![format!(
        "Fix field \"{field}\" to match the Phase A chrome schema"
    )]
}

/// Serialize a window/run error as stderr JSON (`window_create` | `event_loop`).
pub fn emit_run_error(err: &RunError) -> String {
    match err {
        RunError::WindowCreate { message } => {
            StderrError::new(ErrorCode::WindowCreateError, message.clone())
                .cause("Native window or webview construction failed")
                .recovery("Ensure a display server / desktop session is available")
                .recovery("Check platform windowing dependencies (WebKit/WebView2/WebKitGTK)")
                .docs("docs/wyvern-schema/requirements.md (REQ-0073)")
                .to_json_string()
        }
        RunError::EventLoop { message } => {
            StderrError::new(ErrorCode::EventLoopError, message.clone())
                .cause("Window event loop could not start or exited with an OS error")
                .recovery("Retry the command")
                .recovery("Check OS graphics / windowing subsystem health")
                .docs("docs/wyvern-schema/requirements.md (REQ-0073)")
                .to_json_string()
        }
    }
}

/// Serialize a successful [`wyvern_schema::CommandResult`] for stdout.
///
/// # Panics
///
/// Panics if `result` fails to serialize (should be impossible for schema types).
pub fn emit_stdout(result: &wyvern_schema::CommandResult) -> String {
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
    use wyvern_schema::CommandResult;

    #[test]
    fn emit_load_error_parse_with_quotes_is_valid_json() {
        let err = LoadError::Parse {
            message: r#"expected value at line 1: "bad""#.to_string(),
        };
        let out = emit_load_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "parse");
        assert_eq!(value["code"], "PARSE_ERROR");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
        assert!(value.get("cause").is_some());
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
        assert_eq!(value["code"], "IO_ERROR");
        assert_eq!(value["field"], "file");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
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
        assert_eq!(value["code"], "VALIDATION_ERROR");
        assert_eq!(value["field"], "title");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_validation_error_missing_title_has_actionable_recovery() {
        let err = ValidationError::Validation {
            field: "title".to_string(),
            message: "missing required field 'title'".to_string(),
        };
        let out = emit_validation_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("title")));
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
        assert_eq!(value["code"], "STATE_ERROR");
        assert_eq!(value["field"], "action");
        assert!(!value["recovery"].as_array().unwrap().is_empty());
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
        assert_eq!(value["code"], "WINDOW_CREATE_ERROR");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_run_error_event_loop() {
        let err = RunError::EventLoop {
            message: "loop failed".into(),
        };
        let out = emit_run_error(&err);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "event_loop");
        assert_eq!(value["code"], "EVENT_LOOP_ERROR");
        assert_eq!(value["message"], "loop failed");
        assert!(!value["recovery"].as_array().unwrap().is_empty());
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
