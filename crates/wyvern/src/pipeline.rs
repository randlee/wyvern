//! CLI pipeline: validate → load markdown files → host run → emit.

use serde_json::Value;
use wyvern_host::{run as host_run, HostError, HostOptions};
use wyvern_schema::{Command, FieldName};

use crate::error::{
    emit_host_error, emit_io_error, emit_stdout, emit_validation_error, EmitError, LoadError,
};
use crate::observability;

/// Pipeline failure after load: stage stderr + exit, or emit-boundary serialize failure.
#[derive(Debug)]
pub enum PipelineError {
    /// Stage failed after structured stderr was built successfully.
    Stage { stderr: String, exit_code: i32 },
    /// Stdout or stage stderr JSON could not be serialized.
    Emit(EmitError),
}

/// Validate `value`, run the host, and return stdout JSON on success.
///
/// # Errors
///
/// Returns [`PipelineError::Stage`] with stderr JSON and a non-zero exit code on
/// validation, markdown I/O, or host failure. Returns [`PipelineError::Emit`] when
/// structured JSON serialization fails (REQ-0078).
pub fn run_from_loaded(value: Value, host: HostOptions) -> Result<String, PipelineError> {
    observability::log_command_received(&value);
    let command = match wyvern_schema::validate(&value) {
        Ok(cmd) => {
            observability::log_validation_result(true);
            cmd
        }
        Err(e) => {
            observability::log_validation_result(false);
            observability::log_error("validate", &format!("{e:?}"));
            let stderr = emit_validation_error(&e).map_err(PipelineError::Emit)?;
            return Err(PipelineError::Stage {
                stderr,
                exit_code: e.exit_code(),
            });
        }
    };

    let command = match load_markdown_file(command) {
        Ok(cmd) => cmd,
        Err(e) => {
            observability::log_error("load_markdown", &format!("{e:?}"));
            let stderr = emit_io_error(&e).map_err(PipelineError::Emit)?;
            return Err(PipelineError::Stage {
                stderr,
                exit_code: e.exit_code(),
            });
        }
    };

    observability::log_host_start(command_type_name(&command));
    match host_run(command, host) {
        Ok(result) => {
            observability::log_host_result(true);
            emit_stdout(&result).map_err(PipelineError::Emit)
        }
        Err(err) => {
            observability::log_error("host", &format!("{err:?}"));
            observability::log_host_result(false);
            let exit_code = host_error_exit_code(&err);
            let stderr = emit_host_error(&err).map_err(PipelineError::Emit)?;
            Err(PipelineError::Stage { stderr, exit_code })
        }
    }
}

fn command_type_name(command: &Command) -> &'static str {
    match command {
        Command::Chrome { .. } => "chrome",
        Command::Message { .. } => "message",
        Command::Input { .. } => "input",
        Command::Markdown { .. } => "markdown",
        Command::Question { .. } => "question",
    }
}

fn host_error_exit_code(err: &HostError) -> i32 {
    match err {
        HostError::Bind { .. } => wyvern_schema::ErrorCode::HostBindError.exit_code(),
        HostError::UiNotFound { .. } | HostError::UnsupportedType { .. } => {
            wyvern_schema::ErrorCode::HostError.exit_code()
        }
        HostError::ViewerNotFound { .. } | HostError::ViewerUnsupported { .. } => {
            wyvern_schema::ErrorCode::HostViewerError.exit_code()
        }
        HostError::InvalidResult { .. } | HostError::Internal { .. } => {
            wyvern_schema::ErrorCode::HostError.exit_code()
        }
    }
}

/// Read markdown `file` into `content` before the host opens (REQ-0071).
///
/// Missing or unreadable paths return [`LoadError::Io`] so the CLI emits `io`
/// stderr without opening a dialog. Oversized file bodies are rejected at the
/// CLI boundary using the same limit as schema validation.
fn load_markdown_file(command: Command) -> Result<Command, LoadError> {
    match command {
        Command::Markdown {
            title,
            file: Some(path),
            content: None,
            status,
            buttons,
        } => {
            let body = std::fs::read_to_string(&path).map_err(|err| LoadError::Io {
                field: FieldName::new("file"),
                message: format!("could not read path '{path}': {err}"),
            })?;
            if body.len() > wyvern_schema::MARKDOWN_CONTENT_MAX_BYTES {
                return Err(LoadError::Io {
                    field: FieldName::new("file"),
                    message: format!(
                        "markdown content exceeds maximum of {} bytes (got {} bytes)",
                        wyvern_schema::MARKDOWN_CONTENT_MAX_BYTES,
                        body.len()
                    ),
                });
            }
            Ok(Command::Markdown {
                title,
                file: Some(path),
                content: Some(body),
                status,
                buttons,
            })
        }
        other => Ok(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{ButtonsPreset, ChromeTitle};

    #[test]
    fn load_markdown_file_missing_is_io() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let missing = tmp.path().join("definitely-missing-wyvern-b5.md");
        let cmd = Command::Markdown {
            title: Some(ChromeTitle::new("missing.md")),
            file: Some(missing.to_string_lossy().into_owned()),
            content: None,
            status: None,
            buttons: ButtonsPreset::Ok,
        };
        let err = load_markdown_file(cmd).expect_err("missing");
        match err {
            LoadError::Io { field, message } => {
                assert_eq!(field, "file");
                assert!(message.contains("could not read path"));
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn load_markdown_file_reads_utf8() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let path = tmp.path().join("sample.md");
        std::fs::write(&path, "# Hello\n\n- a\n- b\n").unwrap();

        let cmd = Command::Markdown {
            title: Some(ChromeTitle::new("sample.md")),
            file: Some(path.to_string_lossy().into_owned()),
            content: None,
            status: None,
            buttons: ButtonsPreset::Ok,
        };
        let loaded = load_markdown_file(cmd).expect("read");
        match loaded {
            Command::Markdown {
                content: Some(body),
                ..
            } => {
                assert!(body.contains("# Hello"));
            }
            other => panic!("expected loaded Markdown, got {other:?}"),
        }
    }

    #[test]
    fn load_markdown_inline_content_passthrough() {
        let cmd = Command::Markdown {
            title: Some(ChromeTitle::new("Markdown")),
            file: None,
            content: Some("# Inline\n".into()),
            status: None,
            buttons: ButtonsPreset::Ok,
        };
        let loaded = load_markdown_file(cmd).expect("passthrough");
        match loaded {
            Command::Markdown {
                file: None,
                content: Some(body),
                ..
            } => {
                assert_eq!(body, "# Inline\n");
            }
            other => panic!("expected inline Markdown, got {other:?}"),
        }
    }

    #[test]
    fn load_markdown_file_rejects_oversized_body() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let path = tmp.path().join("huge.md");
        let body = "y".repeat(wyvern_schema::MARKDOWN_CONTENT_MAX_BYTES + 1);
        std::fs::write(&path, &body).unwrap();

        let cmd = Command::Markdown {
            title: Some(ChromeTitle::new("huge.md")),
            file: Some(path.to_string_lossy().into_owned()),
            content: None,
            status: None,
            buttons: ButtonsPreset::Ok,
        };
        let err = load_markdown_file(cmd).expect_err("oversized");
        match err {
            LoadError::Io { field, message } => {
                assert_eq!(field, "file");
                assert!(message.contains("exceeds maximum"));
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }
}
