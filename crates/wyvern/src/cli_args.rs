//! Host option flags (`--bind`, `--ui-root`, `--viewer`) and argv splitting.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

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
    // Packaged shared assets are never overridden by `--ui-root` (d.1 dual mount).
    let shared_ui_root = default_ui_root();
    let mut ui_root = shared_ui_root.clone();
    let mut viewer = viewer_from_env().unwrap_or(ViewerMode::Embedded);
    let mut allow_non_loopback = false;
    let mut positionals = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--bind" {
            let value = require_flag_value(args, i, "--bind")?;
            bind = parse_bind(value)?;
            i += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--bind=") {
            bind = parse_bind(value)?;
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
            shared_ui_root,
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

fn parse_bind(value: &str) -> Result<SocketAddr, LoadError> {
    value.parse().map_err(|e| LoadError::Usage {
        message: format!(
            "invalid --bind '{value}': {e}\n\
             Recovery:\n\
             - Use host:port form (example: 127.0.0.1:0 for an ephemeral loopback port)\n\
             - For 0.0.0.0 / LAN binds, also pass --allow-non-loopback\n\
             - Check the address is a valid IPv4/IPv6 socket address\n\
             {}",
            usage_message()
        ),
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

fn viewer_from_env_with(value: Option<&str>) -> Option<ViewerMode> {
    value.and_then(ViewerMode::parse)
}

fn viewer_from_env() -> Option<ViewerMode> {
    viewer_from_env_with(
        std::env::var("WYVERN_VIEWER")
            .ok()
            .as_deref()
            .filter(|s| !s.is_empty()),
    )
}

/// Default UI root discovery order:
///
/// 1. `WYVERN_UI_ROOT` environment variable
/// 2. `./ui` (dev workspace — cwd contains ui/)
/// 3. `./share/wyvern/ui` (cwd install layout)
/// 4. `<exe_dir>/share/wyvern/ui` (release tarball layout — REQ-0093 / REQ-0116)
/// 5. `<exe_dir>/ui` (sibling to binary)
/// 6. Embedded assets extracted to platform cache dir (`cargo install` layout)
/// 7. Fallback `./ui` — caller receives a clear "UI not found" error downstream
pub fn default_ui_root() -> PathBuf {
    default_ui_root_with(
        std::env::var("WYVERN_UI_ROOT").ok().as_deref(),
        std::env::current_dir().ok().as_deref(),
        std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(|p| p.parent()),
        true,
    )
}

/// Resolve the default UI root from injectable inputs (QA-001 — no `set_var` in tests).
#[must_use]
pub fn default_ui_root_with(
    ui_root_var: Option<&str>,
    cwd: Option<&Path>,
    exe_dir: Option<&Path>,
    use_embedded_cache: bool,
) -> PathBuf {
    if let Some(path) = ui_root_var {
        return PathBuf::from(path);
    }
    if let Some(cwd) = cwd {
        let cwd_ui = cwd.join("ui");
        if cwd_ui.is_dir() {
            return cwd_ui;
        }
        let cwd_share = cwd.join("share/wyvern/ui");
        if cwd_share.is_dir() {
            return cwd_share;
        }
    } else {
        let cwd_ui = PathBuf::from("ui");
        if cwd_ui.is_dir() {
            return cwd_ui;
        }
        let cwd_share = PathBuf::from("share/wyvern/ui");
        if cwd_share.is_dir() {
            return cwd_share;
        }
    }
    if let Some(exe_dir) = exe_dir {
        let share = exe_dir.join("share/wyvern/ui");
        if share.is_dir() {
            return share;
        }
        let sibling_ui = exe_dir.join("ui");
        if sibling_ui.is_dir() {
            return sibling_ui;
        }
    }
    if use_embedded_cache {
        if let Some(cached) = crate::embedded_ui::extract_to_cache() {
            return cached;
        }
    }
    PathBuf::from("ui")
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
        "  --ui-root <PATH>           Packaged UI root (default: share/wyvern/ui beside binary)\n",
        "  --viewer <MODE>            embedded|none|system|chrome|safari|edge|firefox\n",
        "                             (default: embedded; CI: WYVERN_VIEWER=none)\n",
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
    fn viewer_from_env_parses_embedded() {
        assert_eq!(
            viewer_from_env_with(Some("embedded")),
            Some(ViewerMode::Embedded)
        );
    }

    #[test]
    fn default_viewer_mode_when_env_unset() {
        assert_eq!(viewer_from_env_with(None), None);
        assert_eq!(
            viewer_from_env_with(None).unwrap_or(ViewerMode::Embedded),
            ViewerMode::Embedded
        );
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
    fn parse_bind_rejects_invalid_with_recovery_hint() {
        let err = parse_cli_args(&args(&["--bind", "not-an-addr"])).expect_err("bind");
        let LoadError::Usage { message } = err else {
            panic!("expected Usage");
        };
        assert!(message.contains("invalid --bind"), "{message}");
        assert!(message.contains("Recovery:"), "{message}");
        assert!(message.contains("--allow-non-loopback"), "{message}");
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let err = parse_cli_args(&args(&["--nope"])).expect_err("flag");
        assert!(matches!(err, LoadError::Usage { .. }));
    }

    #[test]
    fn default_ui_root_prefers_env_override() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let custom = tmp.path().join("custom-ui");
        std::fs::create_dir_all(&custom).expect("mkdir");
        let root = default_ui_root_with(Some(custom.to_str().expect("utf8")), None, None, false);
        assert_eq!(root, custom);
    }

    #[test]
    fn default_ui_root_falls_back_to_ui_when_nothing_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = default_ui_root_with(None, Some(tmp.path()), None, false);
        assert_eq!(root, PathBuf::from("ui"));
    }
}
