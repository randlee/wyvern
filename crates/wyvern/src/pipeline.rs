//! CLI pipeline: validate → load markdown files → run → emit (argv load stays in `main`).

use serde_json::Value;

use wyvern_schema::{Command, ErrorCode, FieldName};
use wyvern_window::RunError;

use crate::error::{
    emit_io_error, emit_run_error, emit_stdout, emit_validation_error, EmitError, LoadError,
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

/// Validate `value`, run the window, and return stdout JSON on success.
///
/// # Errors
///
/// Returns [`PipelineError::Stage`] with stderr JSON and a non-zero exit code on
/// validation, markdown I/O, or run failure. Returns [`PipelineError::Emit`] when
/// structured JSON serialization fails (REQ-0078).
pub fn run_from_loaded(value: Value) -> Result<String, PipelineError> {
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

    observability::log_window_open();
    let result = match wyvern_window::run(command) {
        Ok(r) => {
            observability::log_window_close();
            r
        }
        Err(e) => {
            observability::log_error("run", &format!("{e:?}"));
            let stderr = emit_run_error(&e).map_err(PipelineError::Emit)?;
            let exit_code = match &e {
                RunError::WindowCreate { .. } => ErrorCode::WindowCreateError.exit_code(),
                RunError::EventLoop { .. } => ErrorCode::EventLoopError.exit_code(),
            };
            return Err(PipelineError::Stage { stderr, exit_code });
        }
    };
    observability::log_result_emitted();
    emit_stdout(&result).map_err(PipelineError::Emit)
}

/// Read markdown `file` into `content` before the window opens (REQ-0071).
///
/// Missing or unreadable paths return [`LoadError::Io`] so the CLI emits `io`
/// stderr without opening a window.
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
        let cmd = Command::Markdown {
            title: Some(ChromeTitle::new("missing.md")),
            file: Some("/definitely/missing/wyvern-b5.md".into()),
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
        let dir = std::env::temp_dir().join(format!("wyvern-b5-md-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.md");
        std::fs::write(&path, "# Hello\n\n- a\n- b\n").unwrap();

        let cmd = Command::Markdown {
            title: Some(ChromeTitle::new("sample.md")),
            file: Some(path.to_str().unwrap().to_string()),
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

        let _ = std::fs::remove_dir_all(&dir);
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
}
