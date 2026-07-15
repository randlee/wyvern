//! CLI pipeline: validate → load markdown files → host run / embedded spawn → emit.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::Value;
use wyvern_host::{begin, run as host_run, HostError, HostOptions, ViewerMode};
use wyvern_schema::{Command, FieldName};

use crate::error::{
    emit_host_error, emit_io_error, emit_stdout, emit_validation_error, EmitError, LoadError,
};
use crate::observability;
use crate::viewer_spawn::{spawn_embedded_viewer, wait_for_viewer_exit, ViewerSpawnError};

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
    let result = match host.viewer {
        ViewerMode::Embedded => run_embedded(command, host),
        ViewerMode::None | ViewerMode::System | ViewerMode::Named(_) => {
            host_run(command, host).map_err(PipelineHostError::Host)
        }
    };

    match result {
        Ok(result) => {
            observability::log_host_result(true);
            emit_stdout(&result).map_err(PipelineError::Emit)
        }
        Err(PipelineHostError::Host(err)) => {
            observability::log_error("host", &format!("{err:?}"));
            observability::log_host_result(false);
            let exit_code = host_error_exit_code(&err);
            let stderr = emit_host_error(&err).map_err(PipelineError::Emit)?;
            Err(PipelineError::Stage { stderr, exit_code })
        }
        Err(PipelineHostError::Viewer(err)) => {
            observability::log_error("viewer_spawn", &format!("{err:?}"));
            observability::log_host_result(false);
            let stderr = emit_viewer_spawn_error(&err).map_err(PipelineError::Emit)?;
            Err(PipelineError::Stage {
                stderr,
                exit_code: wyvern_schema::ErrorCode::HostViewerError.exit_code(),
            })
        }
    }
}

enum PipelineHostError {
    Host(HostError),
    Viewer(ViewerSpawnError),
}

fn run_embedded(
    command: Command,
    host: HostOptions,
) -> Result<wyvern_schema::CommandResult, PipelineHostError> {
    #[cfg(target_os = "macos")]
    let picker_pump = wyvern_host::MacosPickerPump::install();

    let mut handle = begin(command, host).map_err(PipelineHostError::Host)?;
    let child = match spawn_embedded_viewer(&handle.dialog_url, &handle.viewer_options) {
        Ok(child) => child,
        Err(err) => {
            // Shut down the host session — no viewer will post a result.
            let _ = handle.viewer_exited_without_result();
            return Err(PipelineHostError::Viewer(err));
        }
    };

    let child = Arc::new(Mutex::new(child));
    let dismiss_tx = handle.take_viewer_exit_signal();
    if let Some(tx) = dismiss_tx {
        let child_for_wait = Arc::clone(&child);
        thread::spawn(move || {
            loop {
                let exited = match child_for_wait.lock() {
                    Ok(mut c) => c.try_wait().ok().flatten().is_some(),
                    Err(_) => true,
                };
                if exited {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            let _ = tx.send(());
        });
    } else {
        let child_for_wait = Arc::clone(&child);
        thread::spawn(move || loop {
            let exited = match child_for_wait.lock() {
                Ok(mut c) => c.try_wait().ok().flatten().is_some(),
                Err(_) => true,
            };
            if exited {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        });
    }

    // Give the child a brief moment to fail-fast (missing display, etc.).
    thread::sleep(Duration::from_millis(50));

    let result = {
        #[cfg(target_os = "macos")]
        {
            loop {
                picker_pump.drain(Duration::from_millis(50));
                if let Some(result) = handle.try_recv_result() {
                    let mapped = result.map_err(PipelineHostError::Host);
                    handle.join_host_worker();
                    break mapped;
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            handle.await_result().map_err(PipelineHostError::Host)
        }
    }?;

    // Parent-controlled viewer shutdown after host graceful stop (page only POSTs result).
    if let Ok(mut c) = child.lock() {
        wait_for_viewer_exit(&mut c);
    }

    Ok(result)
}

fn emit_viewer_spawn_error(err: &ViewerSpawnError) -> Result<String, EmitError> {
    use wyvern_schema::{ErrorCode, StderrError};
    let (message, cause, recovery) = match err {
        ViewerSpawnError::NotFound { hint } => (
            "wyvern-viewer binary not found".to_string(),
            hint.clone(),
            vec![
                "Build or install wyvern-viewer next to the wyvern binary".to_string(),
                "Set WYVERN_VIEWER_BIN to the viewer executable".to_string(),
                "Use --viewer none for headless / CI".to_string(),
            ],
        ),
        ViewerSpawnError::Io { message } => (
            format!("failed to spawn wyvern-viewer: {message}"),
            "Could not start the embedded viewer process".to_string(),
            vec![
                "Verify wyvern-viewer is executable".to_string(),
                "Use --viewer none for headless / CI".to_string(),
            ],
        ),
    };
    let mut envelope = StderrError::new(ErrorCode::HostViewerError, message)
        .cause(cause)
        .docs("docs/plans/phase-C/http-viewer-contract.md");
    for step in recovery {
        envelope = envelope.recovery(step);
    }
    envelope.to_json_string().map_err(EmitError::Serialize)
}

fn command_type_name(command: &Command) -> &'static str {
    match command {
        Command::Chrome { .. } => "chrome",
        Command::Message { .. } => "message",
        Command::Input { .. } => "input",
        Command::Markdown { .. } => "markdown",
        Command::Question { .. } => "question",
        Command::Wizard(_) => "wizard",
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
        HostError::InvalidResult { .. }
        | HostError::Registry { .. }
        | HostError::Internal { .. } => wyvern_schema::ErrorCode::HostError.exit_code(),
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
            width,
            height,
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
                width,
                height,
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
            width: None,
            height: None,
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
            width: None,
            height: None,
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
            width: None,
            height: None,
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
            width: None,
            height: None,
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
