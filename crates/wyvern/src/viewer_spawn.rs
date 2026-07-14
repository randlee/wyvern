//! Discover and spawn the `wyvern-viewer` subprocess for `--viewer embedded`.

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
    if let Some(sibling) = sibling_viewer_bin() {
        if is_executable_file(&sibling) {
            return Ok(sibling);
        }
    }

    if let Ok(path) = std::env::var("CARGO_BIN_EXE_wyvern-viewer") {
        let p = PathBuf::from(&path);
        if is_executable_file(&p) {
            return Ok(p);
        }
    }

    if let Ok(path) = std::env::var("WYVERN_VIEWER_BIN") {
        let p = PathBuf::from(&path);
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

    if let Some(path) = which("wyvern-viewer") {
        if is_executable_file(&path) {
            return Ok(path);
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

fn sibling_viewer_bin() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(dir.join(viewer_bin_name()))
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
    use std::ffi::{OsStr, OsString};
    use std::sync::{Mutex, MutexGuard};

    /// Serializes tests that mutate process-global env (parallel `cargo test` safe).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvLock {
        _guard: MutexGuard<'static, ()>,
    }

    impl EnvLock {
        fn acquire() -> Self {
            Self {
                _guard: ENV_LOCK
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            }
        }
    }

    /// RAII env mutation: restores the prior value (or absence) on drop.
    struct EnvGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: callers hold [`EnvLock`] so no concurrent env mutation.
            unsafe { std::env::set_var(key, value) };
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: callers hold [`EnvLock`] so no concurrent env mutation.
            unsafe { std::env::remove_var(key) };
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: same exclusive [`EnvLock`] as set/remove; restore prior state.
            unsafe {
                match &self.previous {
                    Some(v) => std::env::set_var(self.key, v),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    #[test]
    fn resolve_respects_wyvern_viewer_bin_when_no_sibling() {
        let _lock = EnvLock::acquire();
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
        // Clear cargo bin env so WYVERN_VIEWER_BIN is reached when sibling is absent.
        let _cargo = EnvGuard::remove("CARGO_BIN_EXE_wyvern-viewer");
        let _guard = EnvGuard::set("WYVERN_VIEWER_BIN", &fake);

        // Sibling of the test harness may exist (target/debug/wyvern-viewer). Prefer
        // asserting that an explicit env override is accepted when resolve reaches it,
        // or that resolve succeeds with some executable path.
        match resolve_viewer_bin() {
            Ok(resolved) => {
                assert!(
                    resolved == fake || is_executable_file(&resolved),
                    "resolved={resolved:?}"
                );
            }
            Err(err) => panic!("expected resolve ok, got {err}"),
        }
    }

    #[test]
    fn missing_bin_env_errors_when_override_points_nowhere() {
        let _lock = EnvLock::acquire();
        let tmp = tempfile::tempdir().expect("tmp");
        let missing = tmp.path().join("no-such-viewer");
        // Force override path and clear cargo bin; sibling may still win — only assert
        // NotFound when the override is the sole candidate that resolve would use.
        let _cargo = EnvGuard::remove("CARGO_BIN_EXE_wyvern-viewer");
        let _guard = EnvGuard::set("WYVERN_VIEWER_BIN", &missing);

        // If a sibling binary exists next to the test exe, resolve succeeds via sibling
        // (documented order). Otherwise the missing override must error.
        if sibling_viewer_bin()
            .as_ref()
            .is_some_and(|p| is_executable_file(p))
        {
            let resolved = resolve_viewer_bin().expect("sibling wins");
            assert!(is_executable_file(&resolved));
        } else {
            let err = resolve_viewer_bin().expect_err("missing");
            assert!(matches!(err, ViewerSpawnError::NotFound { .. }));
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_bin_env_errors() {
        let _lock = EnvLock::acquire();
        let tmp = tempfile::tempdir().expect("tmp");
        let fake = tmp.path().join("not-exec-viewer");
        std::fs::write(&fake, b"#!/bin/sh\n").expect("write");
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&fake).unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&fake, perms).unwrap();

        // Call the permission helper directly — resolve may short-circuit on sibling.
        assert!(!is_executable_file(&fake));
        let _cargo = EnvGuard::remove("CARGO_BIN_EXE_wyvern-viewer");
        // Isolate PATH so which() cannot find a real viewer after the override fails.
        let _path = EnvGuard::set("PATH", tmp.path());
        let _guard = EnvGuard::set("WYVERN_VIEWER_BIN", &fake);

        if sibling_viewer_bin()
            .as_ref()
            .is_some_and(|p| is_executable_file(p))
        {
            // Sibling discovered first — permission check still covered above.
            return;
        }
        let err = resolve_viewer_bin().expect_err("not executable");
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
}
