//! Argv/stdin command input loaders.

use std::io::Read;
use std::path::Path;

use serde_json::Value;

use wyvern_schema::FieldName;

use crate::error::LoadError;

/// Canonical usage text for invalid argv / empty stdin.
pub fn usage_message() -> String {
    concat!(
        "Usage: wyvern '<json>' | <file.json> | <file.md>\n",
        "       echo '<json>' | wyvern\n",
        "       wyvern --version\n",
        "\n",
        "Pass exactly one JSON string, .json file, or .md file; or pipe JSON on stdin.",
    )
    .to_string()
}

/// Load a command [`Value`] from positional args or stdin.
///
/// Detection for a single positional arg:
/// - `.md` → `{ "type": "markdown", "file": <path> }` (path only; file not read)
/// - `.json` → read file and parse JSON
/// - otherwise → parse the argument as inline JSON
///
/// # Errors
///
/// Returns [`LoadError::Usage`] for invalid argv shapes or empty stdin,
/// [`LoadError::Parse`] for invalid JSON, and [`LoadError::Io`] for read failures.
pub fn load_command_input(args: &[String], stdin: impl Read) -> Result<Value, LoadError> {
    match args {
        [] => load_stdin(stdin),
        [arg] if arg.starts_with('-') => Err(LoadError::Usage {
            message: usage_message(),
        }),
        [arg] => load_positional(arg),
        _ => Err(LoadError::Usage {
            message: usage_message(),
        }),
    }
}

fn load_positional(arg: &str) -> Result<Value, LoadError> {
    let path = Path::new(arg);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("md") => Ok(serde_json::json!({
            "type": "markdown",
            "file": arg,
        })),
        Some(ext) if ext.eq_ignore_ascii_case("json") => load_json_file(path),
        _ => parse_json(arg),
    }
}

fn load_json_file(path: &Path) -> Result<Value, LoadError> {
    let text = std::fs::read_to_string(path).map_err(|err| LoadError::Io {
        field: FieldName::new("file"),
        message: format!("could not read path '{}': {err}", path.display()),
    })?;
    parse_json(&text)
}

fn load_stdin(mut stdin: impl Read) -> Result<Value, LoadError> {
    let mut buf = String::new();
    stdin
        .read_to_string(&mut buf)
        .map_err(|err| LoadError::Io {
            field: FieldName::new("stdin"),
            message: format!("could not read stdin: {err}"),
        })?;
    if buf.trim().is_empty() {
        return Err(LoadError::Usage {
            message: usage_message(),
        });
    }
    parse_json(&buf)
}

fn parse_json(text: &str) -> Result<Value, LoadError> {
    serde_json::from_str(text).map_err(|err| LoadError::Parse {
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::emit_load_error;
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
        let dir = std::env::temp_dir().join(format!("wyvern-a3-json-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cmd.json");
        std::fs::write(&path, r#"{"type":"chrome","title":"FromFile"}"#).unwrap();

        let value = load_command_input(&args(&[path.to_str().unwrap()]), Cursor::new(""))
            .expect("json file");
        assert_eq!(value["type"], "chrome");
        assert_eq!(value["title"], "FromFile");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn input_md_path_loads_markdown_value() {
        let value =
            load_command_input(&args(&["docs/readme.md"]), Cursor::new("")).expect("md path");
        assert_eq!(value["type"], "markdown");
        assert_eq!(value["file"], "docs/readme.md");
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
        let err = load_command_input(
            &args(&["/definitely/missing/wyvern-a3.json"]),
            Cursor::new(""),
        )
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
        let out = emit_load_error(&err);
        let value: Value = serde_json::from_str(&out).expect("valid JSON stderr");
        assert_eq!(value["error"], "parse");
        assert!(value["message"].is_string());
    }

    #[test]
    fn input_io_error_with_quotes_in_path_emits_valid_json() {
        let dir = std::env::temp_dir().join(format!("wyvern-a3-quote-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Path that does not exist; message will include the path string.
        let path = dir.join(r#"say "hi".json"#);
        let err = load_command_input(&args(&[path.to_str().unwrap()]), Cursor::new(""))
            .expect_err("missing quoted path");
        let out = emit_load_error(&err);
        let value: Value = serde_json::from_str(&out).expect("valid JSON stderr");
        assert_eq!(value["error"], "io");
        assert_eq!(value["field"], "file");
        assert!(value["message"].as_str().unwrap().contains('"'));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
