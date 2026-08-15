//! Preexec subprocess spawn, PATH requires-check, and stdout capture.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use super::ExtensionError;

/// Default preexec timeout in seconds. Override with `WYVERN_PREEXEC_TIMEOUT_SECS`.
/// 30s covers compose/csv helpers without leaving a hung child unbounded.
const DEFAULT_PREEXEC_TIMEOUT_SECS: u64 = 30;

/// Max captured preexec stdout. 1 MiB is enough for markdown capture and
/// prevents a runaway child from exhausting CLI memory.
const MAX_PREEXEC_STDOUT_BYTES: usize = 1024 * 1024;

/// Poll interval while waiting for a preexec child. Short enough that typical
/// helpers appear instantaneous; long enough to avoid a hot loop.
const PREEXEC_WAIT_POLL: Duration = Duration::from_millis(20);

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

/// Runs the extension preexec command. On timeout the child is killed so a
/// piped stdout reader cannot keep buffering after the CLI has moved on.
/// See `WYVERN_PREEXEC_TIMEOUT_SECS`.
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
    let deadline = Instant::now() + preexec_timeout();
    let status = wait_until(&mut child, cmd, deadline)?;
    if status.success() {
        Ok(())
    } else {
        Err(ExtensionError::Preexec {
            message: format!("'{cmd}' exited with {status}"),
        })
    }
}

fn run_capture_stdout(cmd: &str, args: &[String]) -> Result<String, ExtensionError> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to spawn '{cmd}': {err}"),
        })?;
    let stdout = child.stdout.take().ok_or_else(|| ExtensionError::Preexec {
        message: format!("failed to capture '{cmd}' stdout"),
    })?;
    let timeout = preexec_timeout();
    let cmd_owned = cmd.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::Builder::new()
        .name("preexec-stdout".into())
        .spawn(move || {
            let _ = tx.send(read_capped_stdout(&cmd_owned, stdout));
        })
        .map_err(|err| ExtensionError::Preexec {
            message: format!("thread spawn failed: {err}"),
        })?;

    match rx.recv_timeout(timeout) {
        Ok(Ok(raw)) => {
            // Child closed stdout and should exit promptly; do not reuse the
            // pre-spawn deadline, which may already be nearly exhausted.
            let grace_deadline = Instant::now() + Duration::from_millis(500);
            let status = wait_until(&mut child, cmd, grace_deadline)?;
            let _ = reader.join();
            if !status.success() {
                return Err(ExtensionError::Preexec {
                    message: format!("'{cmd}' exited with {status}"),
                });
            }
            String::from_utf8(raw).map_err(|err| ExtensionError::Preexec {
                message: format!("{cmd} stdout is not valid UTF-8: {err}"),
            })
        }
        Ok(Err(err)) => {
            reap_killed(&mut child, reader);
            Err(err)
        }
        Err(_) => {
            reap_killed(&mut child, reader);
            Err(ExtensionError::Preexec {
                message: format!("{cmd} timed out after {}s", timeout.as_secs()),
            })
        }
    }
}

/// Read stdout with a hard byte cap so a runaway child cannot fill memory.
fn read_capped_stdout(cmd: &str, stdout: ChildStdout) -> Result<Vec<u8>, ExtensionError> {
    let mut buf = Vec::new();
    let mut reader = stdout.take(MAX_PREEXEC_STDOUT_BYTES as u64 + 1);
    reader
        .read_to_end(&mut buf)
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to read stdout: {err}"),
        })?;
    if buf.len() > MAX_PREEXEC_STDOUT_BYTES {
        return Err(ExtensionError::Preexec {
            message: format!("{cmd} stdout exceeded {MAX_PREEXEC_STDOUT_BYTES} bytes"),
        });
    }
    Ok(buf)
}

fn wait_until(
    child: &mut Child,
    cmd: &str,
    deadline: Instant,
) -> Result<ExitStatus, ExtensionError> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ExtensionError::Preexec {
                        message: format!("{cmd} timed out after {}s", preexec_timeout().as_secs()),
                    });
                }
                std::thread::sleep(PREEXEC_WAIT_POLL);
            }
            Err(err) => {
                return Err(ExtensionError::Preexec {
                    message: format!("{cmd} wait failed: {err}"),
                });
            }
        }
    }
}

fn reap_killed(child: &mut Child, reader: std::thread::JoinHandle<()>) {
    let _ = child.kill();
    let _ = child.wait();
    let _ = reader.join();
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

    #[cfg(unix)]
    #[test]
    fn preexec_stdout_cap_rejects_oversize() {
        if !binary_on_path("dd") {
            return; // dd not available on this platform
        }
        let err = run_preexec(
            "dd",
            &["if=/dev/zero".into(), "bs=1024".into(), "count=2048".into()],
            Some("markdown"),
        )
        .expect_err("oversize stdout");
        assert!(
            matches!(err, ExtensionError::Preexec { ref message } if message.contains("exceeded")),
            "{err:?}"
        );
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
