//! Argv/stdin command input loaders.

use std::io::Read;
use std::path::Path;

use serde_json::Value;

use wyvern_schema::FieldName;

use crate::cli_args::usage_message;
use crate::error::LoadError;

/// 1 MiB cap for stdin and `.json` file loads (RSH-001 / RSH-002).
const MAX_CLI_INPUT_BYTES: usize = 1024 * 1024;

/// Load a command [`Value`] from positional args or stdin.
///
/// Called after extension dispatch fails (no argv match). Remaining cases:
/// - `.json` → read file and parse JSON
/// - otherwise → parse the argument as inline JSON
///
/// `.md` shorthand is handled by the shipped `markdown-suffix` registry in
/// `main` before this function runs.
///
/// # Errors
///
/// Returns [`LoadError::Usage`] for invalid argv shapes or empty stdin,
/// [`LoadError::Parse`] for invalid JSON, and [`LoadError::Io`] for read
/// failures.
pub fn load_command_input(args: &[String], stdin: impl Read) -> Result<Value, LoadError> {
    match args {
        [] => load_stdin(stdin),
        [arg] if arg.starts_with('-') => Err(LoadError::Usage {
            kind: crate::error::UsageErrorKind::Generic,
            message: usage_message(),
        }),
        [arg] => load_positional(arg),
        _ => Err(LoadError::Usage {
            kind: crate::error::UsageErrorKind::Generic,
            message: usage_message(),
        }),
    }
}

fn load_positional(arg: &str) -> Result<Value, LoadError> {
    let path = Path::new(arg);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => load_json_file(path),
        _ => parse_json(arg),
    }
}

fn load_json_file(path: &Path) -> Result<Value, LoadError> {
    let file = std::fs::File::open(path).map_err(|err| LoadError::Io {
        field: FieldName::new("file"),
        message: format!("could not read path '{}': {err}", path.display()),
        source: Some(Box::new(err)),
    })?;
    let text = read_capped(
        file,
        MAX_CLI_INPUT_BYTES,
        "file",
        &path.display().to_string(),
    )?;
    parse_json(&text)
}

fn load_stdin(stdin: impl Read) -> Result<Value, LoadError> {
    let buf = read_capped(stdin, MAX_CLI_INPUT_BYTES, "stdin", "stdin")?;
    if buf.trim().is_empty() {
        return Err(LoadError::Usage {
            kind: crate::error::UsageErrorKind::Generic,
            message: usage_message(),
        });
    }
    parse_json(&buf)
}

fn read_capped(
    reader: impl Read,
    max: usize,
    field: &str,
    origin: &str,
) -> Result<String, LoadError> {
    let mut buf = Vec::new();
    let n = reader
        .take(max as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|err| LoadError::Io {
            field: FieldName::new(field),
            message: format!("could not read {origin}: {err}"),
            source: Some(Box::new(err)),
        })?;
    if n > max {
        return Err(LoadError::Io {
            field: FieldName::new(field),
            message: format!("{origin} exceeds maximum of {max} bytes"),
            source: None,
        });
    }
    String::from_utf8(buf).map_err(|err| LoadError::Io {
        field: FieldName::new(field),
        message: format!("{origin} is not valid UTF-8: {err}"),
        source: Some(Box::new(err)),
    })
}

fn parse_json(text: &str) -> Result<Value, LoadError> {
    serde_json::from_str(text).map_err(|err| LoadError::Parse {
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{emit_io_error, emit_parse_error};
    use std::io::Cursor;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn input_inline_json_loads() {
        let value = load_command_input(
            &args(&[r#"{"type":"chrome","title":"Hi"}"#]),
            Cursor::new(""),
        )
        .expect("inline JSON");
        assert_eq!(value["type"], "chrome");
        assert_eq!(value["title"], "Hi");
    }

    #[test]
    fn input_json_file_loads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cmd.json");
        std::fs::write(&path, r#"{"type":"chrome","title":"FromFile"}"#).unwrap();

        let value = load_command_input(&args(&[path.to_str().unwrap()]), Cursor::new(""))
            .expect("json file");
        assert_eq!(value["type"], "chrome");
        assert_eq!(value["title"], "FromFile");
    }

    #[test]
    fn input_stdin_loads_json() {
        let value = load_command_input(&[], Cursor::new(r#"{"type":"chrome","title":"Stdin"}"#))
            .expect("stdin JSON");
        assert_eq!(value["type"], "chrome");
        assert_eq!(value["title"], "Stdin");
    }

    #[test]
    fn input_no_args_empty_stdin_is_usage() {
        let err = load_command_input(&[], Cursor::new("")).expect_err("empty stdin");
        assert!(matches!(err, LoadError::Usage { .. }));
    }

    #[test]
    fn input_two_positional_args_is_usage() {
        let err = load_command_input(&args(&["a", "b"]), Cursor::new("")).expect_err("two args");
        assert!(matches!(err, LoadError::Usage { .. }));
    }

    #[test]
    fn input_unknown_flag_is_usage() {
        let err =
            load_command_input(&args(&["--unknown-flag"]), Cursor::new("")).expect_err("flag");
        assert!(matches!(err, LoadError::Usage { .. }));
    }

    #[test]
    fn input_two_file_paths_is_usage() {
        let err = load_command_input(&args(&["file.json", "other.json"]), Cursor::new(""))
            .expect_err("two files");
        assert!(matches!(err, LoadError::Usage { .. }));
    }

    #[test]
    fn input_inline_parse_error() {
        let err =
            load_command_input(&args(&["{not-json"]), Cursor::new("")).expect_err("bad inline");
        assert!(matches!(err, LoadError::Parse { .. }));
    }

    #[test]
    fn input_missing_json_file_is_io() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("definitely-missing-wyvern-a3.json");
        let err = load_command_input(&args(&[missing.to_str().unwrap()]), Cursor::new(""))
            .expect_err("missing file");
        match err {
            LoadError::Io { field, .. } => assert_eq!(field, "file"),
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn input_parse_error_with_quotes_emits_valid_json() {
        let err =
            load_command_input(&args(&[r#"{ "bad": }"#]), Cursor::new("")).expect_err("parse");
        let out = emit_parse_error(&err).expect("emit");
        let value: Value = serde_json::from_str(&out).expect("valid JSON stderr");
        assert_eq!(value["error"], "parse");
        assert!(value["message"].is_string());
    }

    #[test]
    fn input_io_error_with_quotes_in_path_emits_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        // Path that does not exist; message will include the path string.
        let path = dir.path().join(r#"say "hi".json"#);
        let err = load_command_input(&args(&[path.to_str().unwrap()]), Cursor::new(""))
            .expect_err("missing quoted path");
        let out = emit_io_error(&err).expect("emit");
        let value: Value = serde_json::from_str(&out).expect("valid JSON stderr");
        assert_eq!(value["error"], "io");
        assert_eq!(value["field"], "file");
        assert!(value["message"].as_str().unwrap().contains('"'));
    }

    #[test]
    fn input_stdin_rejects_oversize() {
        let huge = format!(
            r#"{{"type":"chrome","title":"{}"}}"#,
            "x".repeat(MAX_CLI_INPUT_BYTES)
        );
        let err = load_command_input(&[], Cursor::new(huge)).expect_err("oversize stdin");
        match err {
            LoadError::Io { field, message, .. } => {
                assert_eq!(field, "stdin");
                assert!(message.contains("exceeds maximum"), "{message}");
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }
}
