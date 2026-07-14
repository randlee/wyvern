//! Host option flags (`--bind`, `--ui-root`, `--viewer`) and argv splitting.

use std::net::SocketAddr;
use std::path::PathBuf;

use wyvern_host::{HostOptions, ViewerMode};

use crate::error::LoadError;

/// Parsed CLI invocation: host options + remaining positional/stdin args.
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Options passed to [`wyvern_host::run`] / [`wyvern_host::begin`].
    pub host: HostOptions,
    /// Non-flag argv entries (JSON / file path).
    pub positionals: Vec<String>,
}

/// Split argv into host flags and positionals.
///
/// Product default (c.15+): omitted `--viewer` → [`ViewerMode::Embedded`].
/// `WYVERN_VIEWER` overrides when set. Unknown flags → usage error.
///
/// # Errors
///
/// Returns [`LoadError::Usage`] for bad flags or values.
pub fn parse_cli_args(args: &[String]) -> Result<CliArgs, LoadError> {
    let mut bind = SocketAddr::from(([127, 0, 0, 1], 0));
    let mut ui_root = default_ui_root();
    let mut viewer = viewer_from_env().unwrap_or(ViewerMode::Embedded);
    let mut allow_non_loopback = false;
    let mut positionals = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--bind" {
            let value = require_flag_value(args, i, "--bind")?;
            bind = value.parse().map_err(|e| LoadError::Usage {
                message: format!("invalid --bind '{value}': {e}"),
            })?;
            i += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--bind=") {
            bind = value.parse().map_err(|e| LoadError::Usage {
                message: format!("invalid --bind '{value}': {e}"),
            })?;
            i += 1;
            continue;
        }
        if arg == "--allow-non-loopback" {
            allow_non_loopback = true;
            i += 1;
            continue;
        }
        if arg == "--ui-root" {
            let value = require_flag_value(args, i, "--ui-root")?;
            ui_root = PathBuf::from(value);
            i += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--ui-root=") {
            ui_root = PathBuf::from(value);
            i += 1;
            continue;
        }
        if arg == "--viewer" {
            let value = require_flag_value(args, i, "--viewer")?;
            viewer = parse_viewer(value)?;
            i += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--viewer=") {
            viewer = parse_viewer(value)?;
            i += 1;
            continue;
        }
        if arg == "--version" || arg == "-V" {
            positionals.push(arg.clone());
            i += 1;
            continue;
        }
        if arg.starts_with('-') {
            return Err(LoadError::Usage {
                message: format!("unknown flag '{arg}'\n{}", usage_message()),
            });
        }
        positionals.push(arg.clone());
        i += 1;
    }

    let dialog_url_env = matches!(viewer, ViewerMode::None);
    Ok(CliArgs {
        host: HostOptions {
            bind,
            ui_root,
            viewer,
            dialog_url_env,
            dialog_url_file: std::env::var_os("WYVERN_DIALOG_URL_FILE").map(PathBuf::from),
            allow_non_loopback,
            session_timeout: wyvern_host::DEFAULT_SESSION_TIMEOUT,
            mock_picker: None,
        },
        positionals,
    })
}

fn require_flag_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &str,
) -> Result<&'a str, LoadError> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| LoadError::Usage {
            message: format!("missing value for {flag}\n{}", usage_message()),
        })
}

fn parse_viewer(value: &str) -> Result<ViewerMode, LoadError> {
    ViewerMode::parse(value).ok_or_else(|| LoadError::Usage {
        message: format!(
            "invalid --viewer '{value}' (expected embedded|none|system|chrome|safari|edge|firefox)\n{}",
            usage_message()
        ),
    })
}

fn viewer_from_env() -> Option<ViewerMode> {
    std::env::var("WYVERN_VIEWER")
        .ok()
        .as_deref()
        .and_then(ViewerMode::parse)
}

/// Default UI root: `WYVERN_UI_ROOT`, else `./ui`, else `./share/wyvern/ui`.
pub fn default_ui_root() -> PathBuf {
    if let Ok(path) = std::env::var("WYVERN_UI_ROOT") {
        return PathBuf::from(path);
    }
    let cwd_ui = PathBuf::from("ui");
    if cwd_ui.is_dir() {
        return cwd_ui;
    }
    let share = PathBuf::from("share/wyvern/ui");
    if share.is_dir() {
        return share;
    }
    cwd_ui
}

/// Canonical usage text for invalid argv / empty stdin.
pub fn usage_message() -> String {
    concat!(
        "Usage: wyvern '<json>' | <file.json> | <file.md> [options]\n",
        "       echo '<json>' | wyvern [options]\n",
        "       wyvern browsers list|refresh\n",
        "       wyvern --version\n",
        "\n",
        "Options:\n",
        "  --bind <ADDR:PORT>         HTTP bind (default 127.0.0.1:0)\n",
        "  --allow-non-loopback       Permit non-loopback --bind (0.0.0.0 / LAN)\n",
        "  --ui-root <PATH>           Packaged UI root (default ./ui)\n",
        "  --viewer <MODE>            embedded|none|system|chrome|safari|edge|firefox\n",
        "                             (default: embedded; CI uses none / WYVERN_VIEWER)\n",
        "\n",
        "Pass exactly one JSON string, .json file, or .md file; or pipe JSON on stdin.",
    )
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parse_defaults_viewer_embedded() {
        // Ensure env override does not leak from other tests.
        std::env::remove_var("WYVERN_VIEWER");
        let parsed = parse_cli_args(&args(&[r#"{"type":"message"}"#])).expect("parse");
        assert_eq!(parsed.host.viewer, ViewerMode::Embedded);
        assert!(!parsed.host.dialog_url_env);
        assert_eq!(parsed.positionals.len(), 1);
    }

    #[test]
    fn parse_viewer_none_explicit() {
        let parsed =
            parse_cli_args(&args(&[r#"{"type":"message"}"#, "--viewer", "none"])).expect("parse");
        assert_eq!(parsed.host.viewer, ViewerMode::None);
        assert!(parsed.host.dialog_url_env);
    }

    #[test]
    fn parse_ui_root_and_bind() {
        let parsed = parse_cli_args(&args(&[
            "--ui-root",
            "./custom-ui",
            "--bind",
            "127.0.0.1:0",
            r#"{"type":"message"}"#,
        ]))
        .expect("parse");
        assert_eq!(parsed.host.ui_root, PathBuf::from("./custom-ui"));
        assert_eq!(parsed.positionals.len(), 1);
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let err = parse_cli_args(&args(&["--nope"])).expect_err("flag");
        assert!(matches!(err, LoadError::Usage { .. }));
    }
}
