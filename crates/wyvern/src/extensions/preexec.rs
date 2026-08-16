//! Preexec subprocess spawn, PATH requires-check, and stdout capture.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use super::{ExtensionError, StdoutCapture, TemplateErrorKind};

/// Default preexec timeout in seconds. Override with `WYVERN_PREEXEC_TIMEOUT_SECS`.
/// 30s covers compose/csv helpers without leaving a hung child unbounded.
const DEFAULT_PREEXEC_TIMEOUT_SECS: u64 = 30;

/// Max captured preexec stdout. 1 MiB is enough for markdown capture and
/// prevents a runaway child from exhausting CLI memory.
const MAX_PREEXEC_STDOUT_BYTES: usize = 1024 * 1024;

/// Max preexec stderr included in [`ExtensionError::Preexec`] (PLAN-CRIT-009).
const MAX_PREEXEC_STDERR_BYTES: usize = 4 * 1024;

/// Poll interval while waiting for a preexec child. Short enough that typical
/// helpers appear instantaneous; long enough to avoid a hot loop.
const PREEXEC_WAIT_POLL: Duration = Duration::from_millis(20);

/// Parse `WYVERN_PREEXEC_TIMEOUT_SECS`. Values below 1 second are rejected.
fn parse_preexec_timeout_secs(raw: Option<&str>) -> Result<u64, ExtensionError> {
    match raw {
        None => Ok(DEFAULT_PREEXEC_TIMEOUT_SECS),
        Some(v) => {
            let secs: u64 = v.parse().map_err(|_| ExtensionError::Preexec {
                message: format!("WYVERN_PREEXEC_TIMEOUT_SECS={v} is not a positive integer"),
                source: None,
            })?;
            if secs < 1 {
                return Err(ExtensionError::Preexec {
                    message: "WYVERN_PREEXEC_TIMEOUT_SECS must be at least 1".into(),
                    source: None,
                });
            }
            Ok(secs)
        }
    }
}

fn preexec_timeout() -> Result<Duration, ExtensionError> {
    parse_preexec_timeout_secs(std::env::var("WYVERN_PREEXEC_TIMEOUT_SECS").ok().as_deref())
        .map(Duration::from_secs)
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
/// times out, exceeds the stdout cap, or exits non-zero.
pub fn run_preexec(
    cmd: &str,
    args: &[String],
    stdout_mode: Option<StdoutCapture>,
) -> Result<Option<String>, ExtensionError> {
    match stdout_mode {
        None => run_without_capture(cmd, args).map(|()| None),
        Some(StdoutCapture::Markdown) => run_capture_stdout(cmd, args).map(Some),
    }
}

fn run_without_capture(cmd: &str, args: &[String]) -> Result<(), ExtensionError> {
    let timeout = preexec_timeout()?;
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to spawn '{cmd}': {err}"),
            source: Some(Box::new(err)),
        })?;
    let stderr_reader = spawn_stderr_reader(&mut child)?;
    let deadline = Instant::now() + timeout;
    let status = match wait_until(&mut child, cmd, deadline, timeout) {
        Ok(status) => status,
        Err(err) => {
            let _ = join_stderr(stderr_reader);
            return Err(err);
        }
    };
    let stderr = join_stderr(stderr_reader);
    if status.success() {
        Ok(())
    } else {
        Err(ExtensionError::Preexec {
            message: preexec_fail_message(cmd, &status.to_string(), &stderr),
            source: None,
        })
    }
}

fn run_capture_stdout(cmd: &str, args: &[String]) -> Result<String, ExtensionError> {
    let timeout = preexec_timeout()?;
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to spawn '{cmd}': {err}"),
            source: Some(Box::new(err)),
        })?;
    let stdout = child.stdout.take().ok_or_else(|| ExtensionError::Preexec {
        message: format!("failed to capture '{cmd}' stdout"),
        source: None,
    })?;
    let stderr_reader = spawn_stderr_reader(&mut child)?;
    let cmd_owned = cmd.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::Builder::new()
        .name("preexec-stdout".into())
        .spawn(move || {
            let _ = tx.send(read_capped_stdout(&cmd_owned, stdout));
        })
        .map_err(|err| ExtensionError::Preexec {
            message: format!("thread spawn failed: {err}"),
            source: Some(Box::new(err)),
        })?;

    match rx.recv_timeout(timeout) {
        Ok(Ok(raw)) => {
            // Child closed stdout and should exit promptly; do not reuse the
            // pre-spawn deadline, which may already be nearly exhausted.
            let grace_deadline = Instant::now() + Duration::from_millis(500);
            let status = wait_until(&mut child, cmd, grace_deadline, timeout)?;
            let _ = reader.join();
            let stderr = join_stderr(stderr_reader);
            if !status.success() {
                return Err(ExtensionError::Preexec {
                    message: preexec_fail_message(cmd, &status.to_string(), &stderr),
                    source: None,
                });
            }
            String::from_utf8(raw).map_err(|err| ExtensionError::Preexec {
                message: format!("{cmd} stdout is not valid UTF-8: {err}"),
                source: Some(Box::new(err)),
            })
        }
        Ok(Err(err)) => {
            reap_killed(&mut child, reader);
            let _ = join_stderr(stderr_reader);
            Err(err)
        }
        Err(_) => {
            reap_killed(&mut child, reader);
            let stderr = join_stderr(stderr_reader);
            Err(ExtensionError::Preexec {
                message: preexec_fail_message(
                    cmd,
                    &format!("timed out after {}s", timeout.as_secs()),
                    &stderr,
                ),
                source: None,
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
            source: Some(Box::new(err)),
        })?;
    if buf.len() > MAX_PREEXEC_STDOUT_BYTES {
        return Err(ExtensionError::Preexec {
            message: format!("{cmd} stdout exceeded {MAX_PREEXEC_STDOUT_BYTES} bytes"),
            source: None,
        });
    }
    Ok(buf)
}

fn spawn_stderr_reader(
    child: &mut Child,
) -> Result<std::thread::JoinHandle<String>, ExtensionError> {
    let stderr = child.stderr.take();
    std::thread::Builder::new()
        .name("preexec-stderr".into())
        .spawn(move || {
            let Some(stderr) = stderr else {
                return String::new();
            };
            let mut buf = Vec::new();
            let mut reader = stderr.take(MAX_PREEXEC_STDERR_BYTES as u64);
            let _ = reader.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).trim().to_string()
        })
        .map_err(|err| ExtensionError::Preexec {
            message: format!("thread spawn failed: {err}"),
            source: Some(Box::new(err)),
        })
}

fn join_stderr(reader: std::thread::JoinHandle<String>) -> String {
    reader.join().unwrap_or_default()
}

fn preexec_fail_message(cmd: &str, status: &str, stderr: &str) -> String {
    if stderr.is_empty() {
        format!("'{cmd}' exited with {status}")
    } else {
        format!("'{cmd}' exited with {status}: {stderr}")
    }
}

fn wait_until(
    child: &mut Child,
    cmd: &str,
    deadline: Instant,
    timeout: Duration,
) -> Result<ExitStatus, ExtensionError> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ExtensionError::Preexec {
                        message: format!("{cmd} timed out after {}s", timeout.as_secs()),
                        source: None,
                    });
                }
                std::thread::sleep(PREEXEC_WAIT_POLL);
            }
            Err(err) => {
                return Err(ExtensionError::Preexec {
                    message: format!("{cmd} wait failed: {err}"),
                    source: Some(Box::new(err)),
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
        .map_err(|err| {
            ExtensionError::template(
                TemplateErrorKind::Unavailable,
                format!(
                    "{{rendered_basename}} requires {{tmpdir}}/pages ({}): {err}",
                    pages.display()
                ),
            )
        })?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().into_string().ok()?;
            name.to_ascii_lowercase().ends_with(".html").then_some(name)
        })
        .collect();
    names.sort();
    names.into_iter().next().ok_or_else(|| {
        ExtensionError::template(
            TemplateErrorKind::Unavailable,
            format!(
                "{{rendered_basename}} found no *.html under {}",
                pages.display()
            ),
        )
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
        source: Some(Box::new(err)),
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
        let out =
            run_preexec("printf", &["# hi".into()], Some(StdoutCapture::Markdown)).expect("printf");
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
            Some(StdoutCapture::Markdown),
        )
        .expect_err("oversize stdout");
        assert!(
            matches!(err, ExtensionError::Preexec { ref message, .. } if message.contains("exceeded")),
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

    #[test]
    fn preexec_timeout_zero_is_rejected() {
        let err = parse_preexec_timeout_secs(Some("0")).expect_err("zero");
        assert!(
            matches!(err, ExtensionError::Preexec { ref message, .. } if message.contains("at least 1")),
            "{err}"
        );
        assert_eq!(
            parse_preexec_timeout_secs(None).expect("default"),
            DEFAULT_PREEXEC_TIMEOUT_SECS
        );
    }

    #[cfg(unix)]
    #[test]
    fn preexec_stderr_appears_in_error() {
        let err = run_preexec(
            "sh",
            &["-c".into(), "echo known-stderr-line >&2; exit 1".into()],
            None,
        )
        .expect_err("nonzero");
        let text = format!("{err}");
        assert!(
            text.contains("known-stderr-line"),
            "preexec error must include stderr snippet: {text}"
        );
    }
}
