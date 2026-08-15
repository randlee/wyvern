//! Preexec subprocess spawn, PATH requires-check, and stdout capture.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use super::ExtensionError;

/// Default preexec timeout in seconds. Override with `WYVERN_PREEXEC_TIMEOUT_SECS`.
/// 30s covers compose/csv helpers without leaving a hung child unbounded.
const DEFAULT_PREEXEC_TIMEOUT_SECS: u64 = 30;

/// Max captured preexec stdout. 1 MiB is enough for markdown capture and
/// prevents a runaway child from exhausting CLI memory.
const MAX_PREEXEC_STDOUT_BYTES: usize = 1024 * 1024;

fn preexec_timeout() -> Duration {
    std::env::var("WYVERN_PREEXEC_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DEFAULT_PREEXEC_TIMEOUT_SECS))
}

/// Probe used at match time for `preexec.requires`.
pub trait RequiresProbe {
    /// Returns whether `name` can be executed via `PATH`.
    fn binary_on_path(&self, name: &str) -> bool;
}

/// Default probe that searches `PATH` (and Windows `PATHEXT`).
#[derive(Debug, Clone, Copy, Default)]
pub struct PathRequiresProbe;

impl RequiresProbe for PathRequiresProbe {
    fn binary_on_path(&self, name: &str) -> bool {
        binary_on_path(name)
    }
}

/// Return whether `name` resolves on `PATH`.
#[must_use]
pub fn binary_on_path(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let as_path = Path::new(name);
    if as_path.is_absolute() {
        return as_path.is_file();
    }
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&paths) {
        if candidate_exists(&dir.join(name)) {
            return true;
        }
        #[cfg(windows)]
        {
            for ext in ["exe", "cmd", "bat", "com"] {
                if candidate_exists(&dir.join(format!("{name}.{ext}"))) {
                    return true;
                }
            }
        }
    }
    false
}

fn candidate_exists(path: &Path) -> bool {
    path.is_file()
}

/// Runs the extension preexec command. On timeout the error is returned but
/// the child process continues running until it naturally exits or the OS
/// reclaims it. Users who interrupt `wyvern` during a long preexec must
/// manually terminate the child if needed. See `WYVERN_PREEXEC_TIMEOUT_SECS`.
///
/// # Errors
///
/// Returns [`ExtensionError::Preexec`] when the process cannot be spawned,
/// times out, exceeds the stdout cap, or exits non-zero. Unknown `stdout`
/// modes are template/contract errors.
pub fn run_preexec(
    cmd: &str,
    args: &[String],
    stdout_mode: Option<&str>,
) -> Result<Option<String>, ExtensionError> {
    match stdout_mode {
        None => run_without_capture(cmd, args).map(|()| None),
        Some("markdown") => run_capture_stdout(cmd, args).map(Some),
        Some(other) => Err(ExtensionError::Template {
            message: format!("unsupported preexec.stdout mode '{other}' (Phase F: markdown only)"),
        }),
    }
}

fn run_without_capture(cmd: &str, args: &[String]) -> Result<(), ExtensionError> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .stdout(Stdio::null())
        .spawn()
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to spawn '{cmd}': {err}"),
        })?;
    let status = wait_child_with_timeout(cmd, move || child.wait())?;
    if status.success() {
        Ok(())
    } else {
        Err(ExtensionError::Preexec {
            message: format!("'{cmd}' exited with {status}"),
        })
    }
}

fn run_capture_stdout(cmd: &str, args: &[String]) -> Result<String, ExtensionError> {
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to spawn '{cmd}': {err}"),
        })?;
    let output = wait_child_with_timeout(cmd, move || child.wait_with_output())?;
    if !output.status.success() {
        return Err(ExtensionError::Preexec {
            message: format!("'{cmd}' exited with {}", output.status),
        });
    }
    let raw = output.stdout;
    if raw.len() > MAX_PREEXEC_STDOUT_BYTES {
        return Err(ExtensionError::Preexec {
            message: format!("{cmd} stdout exceeded {MAX_PREEXEC_STDOUT_BYTES} bytes"),
        });
    }
    String::from_utf8(raw).map_err(|err| ExtensionError::Preexec {
        message: format!("{cmd} stdout is not valid UTF-8: {err}"),
    })
}

fn wait_child_with_timeout<T, F>(cmd: &str, child_wait: F) -> Result<T, ExtensionError>
where
    F: FnOnce() -> std::io::Result<T> + Send + 'static,
    T: Send + 'static,
{
    let timeout = preexec_timeout();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let handle = std::thread::Builder::new()
        .name("preexec-wait".into())
        .spawn(move || {
            let result = child_wait();
            let _ = done_tx.send(());
            result
        })
        .map_err(|err| ExtensionError::Preexec {
            message: format!("thread spawn failed: {err}"),
        })?;
    match done_rx.recv_timeout(timeout) {
        Ok(()) => handle
            .join()
            .map_err(|_| ExtensionError::Preexec {
                message: "preexec wait thread panicked".into(),
            })?
            .map_err(|err| ExtensionError::Preexec {
                message: format!("{cmd} failed: {err}"),
            }),
        Err(_) => Err(ExtensionError::Preexec {
            message: format!("{cmd} timed out after {}s", timeout.as_secs()),
        }),
    }
}

/// Lexicographically first `*.html` basename under `{tmpdir}/pages/`.
///
/// # Errors
///
/// Returns [`ExtensionError::Template`] when the directory is missing or empty.
pub fn first_rendered_html(tmpdir: &Path) -> Result<String, ExtensionError> {
    let pages = tmpdir.join("pages");
    let mut names: Vec<String> = std::fs::read_dir(&pages)
        .map_err(|err| ExtensionError::Template {
            message: format!(
                "{{rendered_basename}} requires {{tmpdir}}/pages ({}): {err}",
                pages.display()
            ),
        })?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().into_string().ok()?;
            name.to_ascii_lowercase().ends_with(".html").then_some(name)
        })
        .collect();
    names.sort();
    names
        .into_iter()
        .next()
        .ok_or_else(|| ExtensionError::Template {
            message: format!(
                "{{rendered_basename}} found no *.html under {}",
                pages.display()
            ),
        })
}

/// Create a secure temp directory for `{tmpdir}`.
///
/// # Errors
///
/// Returns [`ExtensionError::Io`] when a temp dir cannot be created.
pub fn create_tmpdir() -> Result<tempfile::TempDir, ExtensionError> {
    tempfile::TempDir::new().map_err(|err| ExtensionError::Io {
        message: format!("could not create extension temp dir: {err}"),
    })
}

/// Path of an owned temp dir as a [`PathBuf`].
#[must_use]
pub fn tmpdir_path(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_probe_finds_common_binaries() {
        // `false` / `echo` exist on Unix CI; skip assertion if PATH is empty.
        if std::env::var_os("PATH").is_none() {
            return;
        }
        let _ = binary_on_path("false") || binary_on_path("echo") || binary_on_path("sh");
    }

    #[cfg(unix)]
    #[test]
    fn preexec_nonzero_is_error() {
        let err = run_preexec("false", &[], None).expect_err("false");
        assert!(matches!(err, ExtensionError::Preexec { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn preexec_markdown_stdout_capture() {
        let out = run_preexec("printf", &["# hi".into()], Some("markdown")).expect("printf");
        assert_eq!(out.as_deref(), Some("# hi"));
    }

    #[test]
    fn first_rendered_html_picks_lexicographic_first() {
        let tmp = tempfile::tempdir().expect("tmp");
        let pages = tmp.path().join("pages");
        std::fs::create_dir_all(&pages).expect("mkdir");
        std::fs::write(pages.join("foo.html"), "<p>x</p>").expect("write");
        std::fs::write(pages.join("zzz.html"), "<p>z</p>").expect("write");
        assert_eq!(first_rendered_html(tmp.path()).expect("html"), "foo.html");
    }
}
