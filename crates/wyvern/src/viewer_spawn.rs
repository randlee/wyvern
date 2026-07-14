//! Discover and spawn the `wyvern-viewer` subprocess for `--viewer embedded`.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use wyvern_host::ViewerLaunchOptions;

/// Failure locating or launching `wyvern-viewer`.
#[derive(Debug)]
pub enum ViewerSpawnError {
    /// Binary not found (`HOST_VIEWER_ERROR`).
    NotFound {
        /// Install / path hint.
        hint: String,
    },
    /// Spawn I/O failure.
    Io {
        /// Failure detail.
        message: String,
    },
}

impl std::fmt::Display for ViewerSpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { hint } => write!(f, "wyvern-viewer not found; {hint}"),
            Self::Io { message } => write!(f, "failed to spawn wyvern-viewer: {message}"),
        }
    }
}

impl std::error::Error for ViewerSpawnError {}

/// Resolve `wyvern-viewer` via env → sibling → cargo target → PATH.
pub fn resolve_viewer_bin() -> Result<PathBuf, ViewerSpawnError> {
    if let Ok(path) = std::env::var("WYVERN_VIEWER_BIN") {
        let p = PathBuf::from(&path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(ViewerSpawnError::NotFound {
            hint: format!(
                "WYVERN_VIEWER_BIN='{path}' is not an executable file; install wyvern-viewer or fix the path"
            ),
        });
    }

    if let Some(sibling) = sibling_viewer_bin() {
        if sibling.is_file() {
            return Ok(sibling);
        }
    }

    // Dev workspace: target/{debug,release}/wyvern-viewer next to current exe when run via cargo.
    if let Some(target) = cargo_target_viewer_bin() {
        if target.is_file() {
            return Ok(target);
        }
    }

    if let Some(path) = which("wyvern-viewer") {
        return Ok(path);
    }

    Err(ViewerSpawnError::NotFound {
        hint: "install wyvern-viewer next to wyvern, set WYVERN_VIEWER_BIN, or add it to PATH (do not silently fall back to --viewer none)".into(),
    })
}

/// Spawn `wyvern-viewer` for `dialog_url` with optional size hints.
///
/// # Errors
///
/// Returns [`ViewerSpawnError`] when the binary is missing or spawn fails.
pub fn spawn_embedded_viewer(
    dialog_url: &str,
    options: &ViewerLaunchOptions,
) -> Result<Child, ViewerSpawnError> {
    let bin = resolve_viewer_bin()?;
    let mut cmd = Command::new(&bin);
    cmd.arg(dialog_url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    // SAFETY: single-threaded spawn setup before child runs; env is process-local.
    if let Some(w) = options.width {
        cmd.env("WYVERN_VIEWER_WIDTH", w.to_string());
    }
    if let Some(h) = options.height {
        cmd.env("WYVERN_VIEWER_HEIGHT", h.to_string());
    }
    cmd.env("WYVERN_DIALOG_URL", dialog_url);
    cmd.spawn().map_err(|e| ViewerSpawnError::Io {
        message: format!("{}: {e}", bin.display()),
    })
}

fn sibling_viewer_bin() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(dir.join(viewer_bin_name()))
}

fn cargo_target_viewer_bin() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    // .../target/debug/wyvern → .../target/debug/wyvern-viewer
    let dir = exe.parent()?;
    let candidate = dir.join(viewer_bin_name());
    if candidate.is_file() {
        return Some(candidate);
    }
    // .../target/debug/deps/wyvern-* → ../../debug/wyvern-viewer
    if dir.file_name()?.to_str()? == "deps" {
        let debug_or_release = dir.parent()?;
        let candidate = debug_or_release.join(viewer_bin_name());
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn viewer_bin_name() -> &'static str {
    if cfg!(windows) {
        "wyvern-viewer.exe"
    } else {
        "wyvern-viewer"
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{name}.exe"));
            if with_exe.is_file() {
                return Some(with_exe);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_respects_wyvern_viewer_bin() {
        let tmp = tempfile::tempdir().expect("tmp");
        let fake = tmp.path().join(viewer_bin_name());
        std::fs::write(&fake, b"#!/bin/sh\n").expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&fake).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&fake, perms).unwrap();
        }
        std::env::set_var("WYVERN_VIEWER_BIN", &fake);
        let resolved = resolve_viewer_bin().expect("resolve");
        assert_eq!(resolved, fake);
        std::env::remove_var("WYVERN_VIEWER_BIN");
    }

    #[test]
    fn missing_bin_env_errors() {
        let tmp = tempfile::tempdir().expect("tmp");
        let missing = tmp.path().join("no-such-viewer");
        std::env::set_var("WYVERN_VIEWER_BIN", &missing);
        let err = resolve_viewer_bin().expect_err("missing");
        assert!(matches!(err, ViewerSpawnError::NotFound { .. }));
        std::env::remove_var("WYVERN_VIEWER_BIN");
    }
}
