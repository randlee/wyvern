//! Load/validation/run-stage errors and JSON emission helpers.

use wyvern_schema::{ErrorCode, FieldName, SerializeError, StderrError, ValidationError};
use wyvern_window::RunError;

/// Failure while loading command input from argv or stdin.
#[derive(Debug)]
pub enum LoadError {
    /// JSON text could not be parsed.
    Parse { message: String },
    /// A file or stdin read failed.
    Io { field: FieldName, message: String },
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

impl LoadError {
    /// Stable exit code for this load failure.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Parse { .. } => ErrorCode::ParseError.exit_code(),
            Self::Io { .. } => ErrorCode::IoError.exit_code(),
            Self::Usage { .. } => 1,
        }
    }
}

/// Failure serializing stdout or structured stderr JSON at the CLI emit boundary.
#[derive(Debug)]
pub enum EmitError {
    /// `serde_json` could not serialize the envelope or result.
    Serialize(SerializeError),
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for EmitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialize(e) => Some(e),
        }
    }
}

#[cfg(test)]
thread_local! {
    /// Scoped test seam: only the arming thread sees forced stdout emit failures.
    static FORCE_EMIT_STDOUT_FAIL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// RAII guard that forces [`emit_stdout`] to fail on this thread.
#[cfg(test)]
struct ForceEmitStdoutFailGuard;

#[cfg(test)]
impl ForceEmitStdoutFailGuard {
    fn arm() -> Self {
        FORCE_EMIT_STDOUT_FAIL.with(|f| f.set(true));
        Self
    }
}

#[cfg(test)]
impl Drop for ForceEmitStdoutFailGuard {
    fn drop(&mut self) {
        FORCE_EMIT_STDOUT_FAIL.with(|f| f.set(false));
    }
}

/// Serialize a parse load error as stderr JSON.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized, or
/// when `err` is not [`LoadError::Parse`] (miswire).
pub fn emit_parse_error(err: &LoadError) -> Result<String, EmitError> {
    let LoadError::Parse { message } = err else {
        debug_assert!(matches!(err, LoadError::Parse { .. }));
        return Err(EmitError::Serialize(SerializeError {
            message: "emit_parse_error: expected Parse".into(),
        }));
    };
    StderrError::new(ErrorCode::ParseError, message.clone())
        .cause("Input was not valid JSON")
        .recovery("Ensure input is valid JSON")
        .recovery("Check for trailing commas, unquoted keys, or truncated input")
        .docs("docs/wyvern-schema/requirements.md (REQ-0069)")
        .to_json_string()
        .map_err(EmitError::Serialize)
}

/// Serialize an I/O load error as stderr JSON.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized, or
/// when `err` is not [`LoadError::Io`] (miswire).
pub fn emit_io_error(err: &LoadError) -> Result<String, EmitError> {
    let LoadError::Io { field, message } = err else {
        debug_assert!(matches!(err, LoadError::Io { .. }));
        return Err(EmitError::Serialize(SerializeError {
            message: "emit_io_error: expected Io".into(),
        }));
    };
    StderrError::new(ErrorCode::IoError, message.clone())
        .field(field.clone())
        .cause(format!("Failed to read input from '{}'", field.as_str()))
        .recovery("Verify the file path exists and is readable")
        .recovery("Pass JSON inline as an argv string or via stdin")
        .docs("docs/wyvern-schema/requirements.md (REQ-0071)")
        .to_json_string()
        .map_err(EmitError::Serialize)
}

/// Serialize a validation/state error as stderr JSON.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_validation_error(err: &ValidationError) -> Result<String, EmitError> {
    let envelope = match err {
        ValidationError::Validation { field, message } => {
            let mut envelope = StderrError::new(ErrorCode::ValidationError, message.clone())
                .field(field.clone())
                .cause(format!("Command JSON failed schema checks on '{field}'"))
                .docs("docs/wyvern-schema/requirements.md (REQ-0051, REQ-0070)");
            for step in validation_recovery(field.as_str(), message) {
                envelope = envelope.recovery(step);
            }
            envelope
        }
        ValidationError::State { field, message } => {
            StderrError::new(ErrorCode::StateError, message.clone())
                .field(field.clone())
                .cause("Lifecycle action used outside interactive mode")
                .recovery("Run with --interactive to use lifecycle actions (show/hide/exit)")
                .recovery("Omit the action field for one-shot chrome commands")
                .docs("docs/wyvern-schema/requirements.md (REQ-0072)")
        }
    };
    envelope.to_json_string().map_err(EmitError::Serialize)
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
            "Set \"type\" to an executable value for this phase (chrome or message)".into(),
            "Example: {\"type\":\"message\",\"title\":\"T\",\"message\":\"Hi\",\"buttons\":\"ok\"}"
                .into(),
        ];
    }
    if field == "buttons" {
        return vec![
            "Set \"buttons\" to one of: ok, ok_cancel, yes_no, yes_no_cancel, retry_cancel, custom"
                .into(),
        ];
    }
    if field == "level" {
        return vec!["Set \"level\" to one of: info, warning, error, question".into()];
    }
    if field == "custom_buttons" {
        return vec![
            "Provide \"custom_buttons\" as a string array only when \"buttons\" is \"custom\""
                .into(),
        ];
    }
    if field == "default_button" {
        return vec![
            "Set \"default_button\" to a 0-based index within the active button list".into(),
        ];
    }
    if field == "markdown" {
        return vec!["Provide \"markdown\" as a JSON boolean (true or false)".into()];
    }
    if field == "file" && message.contains("exactly one of") {
        return vec![
            "Provide exactly one of \"file\" or \"content\" for markdown commands".into(),
            "Example: {\"type\":\"markdown\",\"file\":\"doc.md\"}".into(),
            "Example: {\"type\":\"markdown\",\"content\":\"# Hello\"}".into(),
        ];
    }
    if message.contains("expected string") {
        return vec![format!("Provide field \"{field}\" as a JSON string")];
    }
    if message.contains("unknown field") {
        return vec![format!(
            "Remove unknown field \"{field}\"; check the schema for this command type"
        )];
    }
    if message.contains("expected JSON object") {
        return vec!["Pass a single JSON object as the command payload".into()];
    }
    vec![format!(
        "Fix field \"{field}\" to match the current phase command schema"
    )]
}

/// Serialize a window/run error as stderr JSON (`window_create` | `event_loop`).
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_run_error(err: &RunError) -> Result<String, EmitError> {
    let envelope = match err {
        RunError::WindowCreate { message } if is_media_window_create(message) => {
            StderrError::new(ErrorCode::WindowCreateError, message.clone())
                .cause("Icon or decorative media could not be resolved at run time")
                .recovery(
                    "Use a known named icon (info, warning, error, question, success, loading) \
                     with an in-range variant index",
                )
                .recovery("Verify icon/image file paths exist and data URIs are well-formed")
                .docs("docs/wyvern-schema/requirements.md (REQ-0073, REQ-0031)")
        }
        RunError::WindowCreate { message } => {
            StderrError::new(ErrorCode::WindowCreateError, message.clone())
                .cause("Native window or webview construction failed")
                .recovery("Ensure a display server / desktop session is available")
                .recovery("Check platform windowing dependencies (WebKit/WebView2/WebKitGTK)")
                .docs("docs/wyvern-schema/requirements.md (REQ-0073)")
        }
        RunError::EventLoop { message } => {
            StderrError::new(ErrorCode::EventLoopError, message.clone())
                .cause("Window event loop could not start or exited with an OS error")
                .recovery("Retry the command")
                .recovery("Check OS graphics / windowing subsystem health")
                .docs("docs/wyvern-schema/requirements.md (REQ-0073)")
        }
    };
    envelope.to_json_string().map_err(EmitError::Serialize)
}

/// True when `WindowCreate` came from icon/media defense-in-depth (not OS windowing).
fn is_media_window_create(message: &str) -> bool {
    message.starts_with("missing level icon embed")
        || message.starts_with("invalid icon spec")
        || message.starts_with("missing embed for")
        || message.starts_with("failed to load media path")
}

/// Serialize a successful [`wyvern_schema::CommandResult`] for stdout.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when `result` cannot be serialized.
pub fn emit_stdout(result: &wyvern_schema::CommandResult) -> Result<String, EmitError> {
    #[cfg(test)]
    {
        if FORCE_EMIT_STDOUT_FAIL.with(std::cell::Cell::get) {
            return Err(EmitError::Serialize(SerializeError {
                message: "forced".into(),
            }));
        }
    }
    serde_json::to_string(result).map_err(|e| {
        EmitError::Serialize(SerializeError {
            message: e.to_string(),
        })
    })
}

/// Emit static internal stderr JSON and exit with code 8 (REQ-0078).
///
/// Uses a hand-built JSON string so a serialize failure cannot recurse.
/// Includes `cause` / `recovery` / `docs` per the stderr contract (RBP-F004).
pub fn emit_fatal_internal(err: &EmitError) -> ! {
    let EmitError::Serialize(e) = err;
    let msg_json =
        serde_json::to_string(&e.message).unwrap_or_else(|_| "\"serialization failed\"".into());
    eprintln!(
        r#"{{"error":"internal","code":"INTERNAL_ERROR","message":{msg_json},"cause":"Stdout or stderr JSON serialization failed at the CLI emit boundary","recovery":["Retry the command","Report a bug if the payload is valid JSON but emit still fails"],"docs":"docs/wyvern-schema/requirements.md (REQ-0078)"}}"#
    );
    std::process::exit(ErrorCode::InternalError.exit_code());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use wyvern_schema::{ButtonLabel, ChromeResult, CommandResult, FieldName, MessageResult};

    #[test]
    fn emit_parse_error_with_quotes_is_valid_json() {
        let err = LoadError::Parse {
            message: r#"expected value at line 1: "bad""#.to_string(),
        };
        let out = emit_parse_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "parse");
        assert_eq!(value["code"], "PARSE_ERROR");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
        assert!(value.get("cause").is_some());
    }

    #[test]
    fn emit_io_error_with_quotes_is_valid_json() {
        let err = LoadError::Io {
            field: FieldName::new("file"),
            message: r#"could not read path 'say "hi".json'"#.to_string(),
        };
        let out = emit_io_error(&err).expect("emit");
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
            field: FieldName::new("title"),
            message: r#"field 'title' expected string, got "oops""#.to_string(),
        };
        let out = emit_validation_error(&err).expect("emit");
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
            field: FieldName::new("title"),
            message: "missing required field 'title'".to_string(),
        };
        let out = emit_validation_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("title")));
    }

    #[test]
    fn emit_validation_error_state() {
        let err = ValidationError::State {
            field: FieldName::new("action"),
            message: "show is only valid in --interactive mode".to_string(),
        };
        let out = emit_validation_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "state");
        assert_eq!(value["code"], "STATE_ERROR");
        assert_eq!(value["field"], "action");
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_stdout_chrome_wire_shape() {
        let result = CommandResult::Chrome(ChromeResult {
            button: ButtonLabel::dismissed(),
        });
        assert_eq!(
            emit_stdout(&result).expect("emit"),
            r#"{"button":"dismissed"}"#
        );
    }

    #[test]
    fn emit_stdout_message_wire_shape() {
        let result = CommandResult::Message(MessageResult {
            button: ButtonLabel::new("ok"),
        });
        assert_eq!(emit_stdout(&result).expect("emit"), r#"{"button":"ok"}"#);
    }

    #[test]
    #[serial]
    fn emit_stdout_forced_fail() {
        let _guard = ForceEmitStdoutFailGuard::arm();
        let result = CommandResult::Message(MessageResult {
            button: ButtonLabel::new("ok"),
        });
        assert!(emit_stdout(&result).is_err());
    }

    #[test]
    fn emit_run_error_media_window_create_has_media_recovery() {
        let err = RunError::WindowCreate {
            message: "invalid icon spec 'bad:role:99'".into(),
        };
        let out = emit_run_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "window_create");
        assert_eq!(value["code"], "WINDOW_CREATE_ERROR");
        assert!(value["cause"]
            .as_str()
            .unwrap()
            .contains("Icon or decorative media"));
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("named icon")));
    }

    #[test]
    fn emit_run_error_window_create() {
        let err = RunError::WindowCreate {
            message: r#"create failed: "boom""#.into(),
        };
        let out = emit_run_error(&err).expect("emit");
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
        let out = emit_run_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "event_loop");
        assert_eq!(value["code"], "EVENT_LOOP_ERROR");
        assert_eq!(value["message"], "loop failed");
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn load_error_exit_codes() {
        assert_eq!(
            LoadError::Parse {
                message: "x".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            LoadError::Io {
                field: FieldName::new("file"),
                message: "x".into()
            }
            .exit_code(),
            3
        );
        assert_eq!(
            LoadError::Usage {
                message: "usage".into()
            }
            .exit_code(),
            1
        );
    }

    #[test]
    fn validation_error_exit_codes() {
        assert_eq!(
            ValidationError::Validation {
                field: FieldName::new("title"),
                message: "bad".into(),
            }
            .exit_code(),
            4
        );
        assert_eq!(
            ValidationError::State {
                field: FieldName::new("action"),
                message: "bad".into(),
            }
            .exit_code(),
            5
        );
    }

    #[test]
    fn emit_run_error_maps_window_create_category() {
        let err = RunError::WindowCreate {
            message: "no display".into(),
        };
        let json = emit_run_error(&err).expect("emit");
        assert_eq!(ErrorCode::WindowCreateError.exit_code(), 6);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(value["error"], "window_create");
        assert_eq!(value["message"], "no display");
    }

    #[test]
    fn emit_run_error_maps_event_loop_category() {
        let err = RunError::EventLoop {
            message: "os error".into(),
        };
        let json = emit_run_error(&err).expect("emit");
        assert_eq!(ErrorCode::EventLoopError.exit_code(), 7);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(value["error"], "event_loop");
        assert_eq!(value["message"], "os error");
    }
}
