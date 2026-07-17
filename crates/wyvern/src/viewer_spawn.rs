//! Discover and spawn the `wyvern-viewer` subprocess for `--viewer embedded`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
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

/// Resolve `wyvern-viewer` via sibling → `CARGO_BIN_EXE` → `WYVERN_VIEWER_BIN` → `PATH`.
pub fn resolve_viewer_bin() -> Result<PathBuf, ViewerSpawnError> {
    let cargo_bin = std::env::var("CARGO_BIN_EXE_wyvern-viewer").ok();
    let wyvern_bin = std::env::var("WYVERN_VIEWER_BIN").ok();
    let path = std::env::var_os("PATH");
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_deref().and_then(|p| p.parent());

    resolve_viewer_bin_with(&ViewerResolveEnv {
        exe_dir,
        cargo_bin_exe: cargo_bin.as_deref(),
        wyvern_viewer_bin: wyvern_bin.as_deref(),
        path: path.as_deref(),
    })
}

/// Injectable viewer discovery inputs (QA-002 — no `set_var` in unit tests).
#[derive(Debug, Clone, Default)]
pub struct ViewerResolveEnv<'a> {
    /// Directory containing the running executable (sibling probe).
    pub exe_dir: Option<&'a Path>,
    /// `CARGO_BIN_EXE_wyvern-viewer` when set.
    pub cargo_bin_exe: Option<&'a str>,
    /// `WYVERN_VIEWER_BIN` when set.
    pub wyvern_viewer_bin: Option<&'a str>,
    /// `PATH` directories for `which` lookup.
    pub path: Option<&'a OsStr>,
}

/// Resolve `wyvern-viewer` from injectable discovery inputs.
pub fn resolve_viewer_bin_with(env: &ViewerResolveEnv<'_>) -> Result<PathBuf, ViewerSpawnError> {
    if let Some(dir) = env.exe_dir {
        let sibling = dir.join(viewer_bin_name());
        if is_executable_file(&sibling) {
            return Ok(sibling);
        }
    }

    if let Some(path) = env.cargo_bin_exe {
        let p = PathBuf::from(path);
        if is_executable_file(&p) {
            return Ok(p);
        }
    }

    if let Some(path) = env.wyvern_viewer_bin {
        let p = PathBuf::from(path);
        if is_executable_file(&p) {
            return Ok(p);
        }
        if p.is_file() {
            return Err(ViewerSpawnError::NotFound {
                hint: format!(
                    "WYVERN_VIEWER_BIN='{path}' exists but is not executable; chmod +x or fix the path"
                ),
            });
        }
        return Err(ViewerSpawnError::NotFound {
            hint: format!(
                "WYVERN_VIEWER_BIN='{path}' is not an executable file; install wyvern-viewer or fix the path"
            ),
        });
    }

    if let Some(path_var) = env.path {
        if let Some(path) = which_in_path(path_var, viewer_bin_name()) {
            if is_executable_file(&path) {
                return Ok(path);
            }
        }
    }

    Err(ViewerSpawnError::NotFound {
        hint: "install wyvern-viewer next to wyvern, set WYVERN_VIEWER_BIN, or add it to PATH (do not silently fall back to --viewer none)".into(),
    })
}

/// Spawn `wyvern-viewer` for `dialog_url` with optional size hints.
///
/// Stdin is piped so the CLI can send `exit\n` after the host accepts
/// `POST /api/result` (parent-controlled shutdown — page does not close the window).
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
        .stdin(Stdio::piped())
        .stdout(Stdio::null());
    // Panics during macOS teardown must not leak Rust stack traces to the agent/user.
    if std::env::var_os("WYVERN_VIEWER_LOG").is_some() {
        cmd.stderr(Stdio::inherit());
    } else {
        cmd.stderr(Stdio::null());
    }
    if let Some(w) = options.width {
        cmd.env("WYVERN_VIEWER_WIDTH", w.to_string());
    }
    if let Some(h) = options.height {
        cmd.env("WYVERN_VIEWER_HEIGHT", h.to_string());
    }
    if let Some(title) = &options.title {
        cmd.env("WYVERN_VIEWER_TITLE", title);
    }
    cmd.env("WYVERN_DIALOG_URL", dialog_url);
    cmd.spawn().map_err(|e| ViewerSpawnError::Io {
        message: format!("{}: {e}", bin.display()),
    })
}

/// Ask an embedded viewer to exit after the host session completes.
///
/// Writes `exit\n` to the child's stdin (see `spawn_embedded_viewer`). The viewer
/// hides and tears down on its own — avoids page-initiated close racing macOS focus.
pub fn request_viewer_exit(child: &mut Child) {
    use std::io::Write;
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(b"exit\n");
        let _ = stdin.flush();
    }
}

/// Block until the embedded viewer exits after [`request_viewer_exit`].
pub fn wait_for_viewer_exit(child: &mut Child) {
    use std::thread;
    use std::time::{Duration, Instant};

    request_viewer_exit(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(_) => return,
        }
        if Instant::now() >= deadline {
            request_viewer_exit(child);
            let _ = child.wait();
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn viewer_bin_name() -> &'static str {
    if cfg!(windows) {
        "wyvern-viewer.exe"
    } else {
        "wyvern-viewer"
    }
}

fn which_in_path(path_var: &OsStr, name: &str) -> Option<PathBuf> {
    for dir in std::env::split_paths(path_var) {
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{name}.exe"));
            if is_executable_file(&with_exe) {
                return Some(with_exe);
            }
        }
    }
    None
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(meta) => meta.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_executable(path: &Path) {
        std::fs::write(path, b"#!/bin/sh\n").expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms).unwrap();
        }
    }

    #[test]
    fn resolve_prefers_wyvern_viewer_bin_override() {
        let tmp = tempfile::tempdir().expect("tmp");
        let fake = tmp.path().join(viewer_bin_name());
        make_executable(&fake);
        let env = ViewerResolveEnv {
            exe_dir: None,
            cargo_bin_exe: None,
            wyvern_viewer_bin: Some(fake.to_str().expect("utf8")),
            path: None,
        };
        let resolved = resolve_viewer_bin_with(&env).expect("override");
        assert_eq!(resolved, fake);
    }

    #[test]
    fn resolve_errors_when_override_missing() {
        let tmp = tempfile::tempdir().expect("tmp");
        let missing = tmp.path().join("no-such-viewer");
        let env = ViewerResolveEnv {
            exe_dir: None,
            cargo_bin_exe: None,
            wyvern_viewer_bin: Some(missing.to_str().expect("utf8")),
            path: None,
        };
        let err = resolve_viewer_bin_with(&env).expect_err("missing");
        assert!(matches!(err, ViewerSpawnError::NotFound { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_bin_override_errors() {
        let tmp = tempfile::tempdir().expect("tmp");
        let fake = tmp.path().join("not-exec-viewer");
        std::fs::write(&fake, b"#!/bin/sh\n").expect("write");
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&fake).unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&fake, perms).unwrap();
        assert!(!is_executable_file(&fake));

        let env = ViewerResolveEnv {
            exe_dir: None,
            cargo_bin_exe: None,
            wyvern_viewer_bin: Some(fake.to_str().expect("utf8")),
            path: Some(tmp.path().as_os_str()),
        };
        let err = resolve_viewer_bin_with(&env).expect_err("not executable");
        match err {
            ViewerSpawnError::NotFound { hint } => {
                assert!(
                    hint.contains("not executable") || hint.contains("not an executable"),
                    "hint={hint}"
                );
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn resolve_prefers_sibling_before_path() {
        let tmp = tempfile::tempdir().expect("tmp");
        let sibling = tmp.path().join(viewer_bin_name());
        make_executable(&sibling);
        let other = tmp.path().join("other-viewer");
        make_executable(&other);
        let env = ViewerResolveEnv {
            exe_dir: Some(tmp.path()),
            cargo_bin_exe: None,
            wyvern_viewer_bin: None,
            path: Some(tmp.path().as_os_str()),
        };
        let resolved = resolve_viewer_bin_with(&env).expect("sibling");
        assert_eq!(resolved, sibling);
    }
}
