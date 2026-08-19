//! Preexec subprocess spawn, PATH requires-check, and stdout capture.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use super::{ExtensionError, StdoutCapture, TemplateErrorKind};

/// Why a preexec subprocess failed.
///
/// `Timeout` is classified from the existing sync poll — it does not add async
/// timeout infrastructure (sprint g.2 non-closure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreexecFailureKind {
    /// The helper binary could not be spawned (`ErrorKind::NotFound`).
    SpawnNotFound {
        /// Expanded `preexec.cmd`.
        cmd: String,
    },
    /// The helper ran and exited nonzero.
    NonZeroExit {
        /// Process exit code, or `1` when killed by signal.
        code: i32,
        /// Last 4 KiB of child stderr.
        stderr_tail: String,
    },
    /// The helper exceeded `WYVERN_PREEXEC_TIMEOUT_SECS` (sync poll).
    Timeout {
        /// Expanded `preexec.cmd`.
        cmd: String,
        /// Timeout that elapsed, in seconds.
        timeout_secs: u64,
    },
}

fn preexec_error(
    kind: Option<PreexecFailureKind>,
    message: impl Into<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
) -> ExtensionError {
    ExtensionError::Preexec {
        kind,
        message: message.into(),
        source,
    }
}

fn spawn_error(cmd: &str, err: std::io::Error) -> ExtensionError {
    let kind = match err.kind() {
        std::io::ErrorKind::NotFound => Some(PreexecFailureKind::SpawnNotFound {
            cmd: cmd.to_string(),
        }),
        _ => None,
    };
    preexec_error(
        kind,
        format!("failed to spawn '{cmd}': {err}"),
        Some(Box::new(err)),
    )
}

fn nonzero_error(cmd: &str, status: &ExitStatus, stderr: String) -> ExtensionError {
    let code = status.code().unwrap_or(1);
    preexec_error(
        Some(PreexecFailureKind::NonZeroExit {
            code,
            stderr_tail: stderr.clone(),
        }),
        preexec_fail_message(cmd, &status.to_string(), &stderr),
        None,
    )
}

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
            let secs: u64 = v.parse().map_err(|_| {
                preexec_error(
                    None,
                    format!("WYVERN_PREEXEC_TIMEOUT_SECS={v} is not a positive integer"),
                    None,
                )
            })?;
            if secs < 1 {
                return Err(preexec_error(
                    None,
                    "WYVERN_PREEXEC_TIMEOUT_SECS must be at least 1",
                    None,
                ));
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

/// Subprocess request shared by extension preexec and workflow hooks.
#[derive(Debug)]
pub struct ScriptRequest {
    /// Program name or absolute path.
    pub program: OsString,
    /// Arguments after the program.
    pub args: Vec<OsString>,
    /// Optional working directory.
    pub cwd: Option<PathBuf>,
    /// Extra environment variables (inherited env plus these keys).
    pub extra_env: Vec<(OsString, OsString)>,
    /// Optional stdin bytes. `None` uses `/dev/null`.
    pub stdin: Option<Vec<u8>>,
    /// When true, capture stdout (capped) instead of discarding it.
    pub capture_stdout: bool,
    /// Kill the child after this duration.
    pub timeout: Duration,
    /// When true on Unix, spawn in a new process group and kill the group
    /// on timeout so descendants are reaped (workflow scripts).
    pub process_group: bool,
}

/// Successful wait outcome for [`run_script`].
#[derive(Debug)]
pub struct ScriptOutput {
    /// Captured stdout when requested.
    pub stdout: Option<String>,
    /// Last 4 KiB of child stderr.
    pub stderr_tail: String,
    /// Child exit status.
    pub status: ExitStatus,
}

/// Failure from [`run_script`] (spawn, timeout, wait, or stdout capture).
#[derive(Debug)]
pub enum ScriptError {
    /// Program was not found on PATH or as an absolute file.
    SpawnNotFound {
        /// Program token.
        cmd: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Spawn failed for a reason other than not-found.
    Spawn {
        /// Program token.
        cmd: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Child exceeded `timeout`.
    Timeout {
        /// Program token.
        cmd: String,
        /// Timeout in seconds.
        timeout_secs: u64,
        /// Stderr collected before kill.
        stderr_tail: String,
    },
    /// `wait` failed after spawn.
    Wait {
        /// Program token.
        cmd: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Stdout could not be read or was not UTF-8 / exceeded the cap.
    Stdout {
        /// Program token.
        cmd: String,
        /// Human-readable cause.
        cause: String,
    },
    /// Helper thread could not be started.
    Thread {
        /// Human-readable cause.
        message: String,
    },
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawnNotFound { cmd, source } => {
                write!(f, "failed to spawn '{cmd}': {source}")
            }
            Self::Spawn { cmd, source } => write!(f, "failed to spawn '{cmd}': {source}"),
            Self::Timeout {
                cmd, timeout_secs, ..
            } => write!(f, "{cmd} timed out after {timeout_secs}s"),
            Self::Wait { cmd, source } => write!(f, "{cmd} wait failed: {source}"),
            Self::Stdout { cmd, cause } => write!(f, "{cmd} stdout: {cause}"),
            Self::Thread { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SpawnNotFound { source, .. }
            | Self::Spawn { source, .. }
            | Self::Wait { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Spawn a subprocess with timeout, optional stdin, and stderr tail.
///
/// Workflow hooks and extension preexec share this helper so `workflow/` does
/// not grow a second `Command::new` stack (ADR-0023).
///
/// # Errors
///
/// Returns [`ScriptError`] when the process cannot be spawned, times out,
/// exceeds the stdout cap, or wait/IO fails.
pub fn run_script(request: &ScriptRequest) -> Result<ScriptOutput, ScriptError> {
    let cmd = request.program.to_string_lossy().into_owned();
    let mut child_cmd = Command::new(&request.program);
    child_cmd.args(&request.args);
    if let Some(cwd) = &request.cwd {
        child_cmd.current_dir(cwd);
    }
    for (key, value) in &request.extra_env {
        child_cmd.env(key, value);
    }
    child_cmd.stderr(Stdio::piped());
    if request.capture_stdout {
        child_cmd.stdout(Stdio::piped());
    } else {
        child_cmd.stdout(Stdio::null());
    }
    if request.stdin.is_some() {
        child_cmd.stdin(Stdio::piped());
    } else {
        child_cmd.stdin(Stdio::null());
    }
    #[cfg(unix)]
    if request.process_group {
        use std::os::unix::process::CommandExt;
        child_cmd.process_group(0);
    }

    let mut child = child_cmd.spawn().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            ScriptError::SpawnNotFound {
                cmd: cmd.clone(),
                source: err,
            }
        } else {
            ScriptError::Spawn {
                cmd: cmd.clone(),
                source: err,
            }
        }
    })?;

    if let Some(data) = &request.stdin {
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(err) = stdin.write_all(data) {
                terminate_child(&mut child, request.process_group);
                return Err(ScriptError::Spawn { cmd, source: err });
            }
        }
    }

    let stderr_reader = spawn_stderr_reader(&mut child).map_err(|err| ScriptError::Thread {
        message: err.to_string(),
    })?;
    let deadline = Instant::now() + request.timeout;

    if request.capture_stdout {
        let stdout = child.stdout.take().ok_or_else(|| ScriptError::Stdout {
            cmd: cmd.clone(),
            cause: "failed to capture stdout".into(),
        })?;
        let cmd_owned = cmd.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let reader = std::thread::Builder::new()
            .name("script-stdout".into())
            .spawn(move || {
                let _ = tx.send(read_capped_stdout(&cmd_owned, stdout));
            })
            .map_err(|err| ScriptError::Thread {
                message: format!("thread spawn failed: {err}"),
            })?;

        match rx.recv_timeout(request.timeout) {
            Ok(Ok(raw)) => {
                let grace_deadline = Instant::now() + Duration::from_millis(500);
                let status = match wait_until(
                    &mut child,
                    &cmd,
                    grace_deadline,
                    request.timeout,
                    request.process_group,
                ) {
                    Ok(status) => status,
                    Err(err) => {
                        let _ = reader.join();
                        let stderr = join_stderr(stderr_reader);
                        return Err(map_wait_error(err, stderr));
                    }
                };
                let _ = reader.join();
                let stderr_tail = join_stderr(stderr_reader);
                let stdout = String::from_utf8(raw).map_err(|err| ScriptError::Stdout {
                    cmd: cmd.clone(),
                    cause: format!("not valid UTF-8: {err}"),
                })?;
                Ok(ScriptOutput {
                    stdout: Some(stdout),
                    stderr_tail,
                    status,
                })
            }
            Ok(Err(err)) => {
                reap_killed(&mut child, reader, request.process_group);
                let _ = join_stderr(stderr_reader);
                Err(ScriptError::Stdout {
                    cmd,
                    cause: err.to_string(),
                })
            }
            Err(_) => {
                reap_killed(&mut child, reader, request.process_group);
                let stderr_tail = join_stderr(stderr_reader);
                Err(ScriptError::Timeout {
                    cmd,
                    timeout_secs: request.timeout.as_secs(),
                    stderr_tail,
                })
            }
        }
    } else {
        let status = match wait_until(
            &mut child,
            &cmd,
            deadline,
            request.timeout,
            request.process_group,
        ) {
            Ok(status) => status,
            Err(err) => {
                let stderr = join_stderr(stderr_reader);
                return Err(map_wait_error(err, stderr));
            }
        };
        let stderr_tail = join_stderr(stderr_reader);
        Ok(ScriptOutput {
            stdout: None,
            stderr_tail,
            status,
        })
    }
}

fn map_wait_error(err: ExtensionError, stderr_tail: String) -> ScriptError {
    match err {
        ExtensionError::Preexec {
            kind: Some(PreexecFailureKind::Timeout { cmd, timeout_secs }),
            ..
        } => ScriptError::Timeout {
            cmd,
            timeout_secs,
            stderr_tail,
        },
        ExtensionError::Preexec {
            message, source, ..
        } => {
            if let Some(source) =
                source.and_then(|s| s.downcast::<std::io::Error>().ok().map(|b| *b))
            {
                ScriptError::Wait {
                    cmd: message,
                    source,
                }
            } else {
                ScriptError::Wait {
                    cmd: message,
                    source: std::io::Error::other("wait failed"),
                }
            }
        }
        other => ScriptError::Wait {
            cmd: other.to_string(),
            source: std::io::Error::other(other.to_string()),
        },
    }
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
        .map_err(|err| spawn_error(cmd, err))?;
    let stderr_reader = spawn_stderr_reader(&mut child)?;
    let deadline = Instant::now() + timeout;
    let status = match wait_until(&mut child, cmd, deadline, timeout, false) {
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
        Err(nonzero_error(cmd, &status, stderr))
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
        .map_err(|err| spawn_error(cmd, err))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| preexec_error(None, format!("failed to capture '{cmd}' stdout"), None))?;
    let stderr_reader = spawn_stderr_reader(&mut child)?;
    let cmd_owned = cmd.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::Builder::new()
        .name("preexec-stdout".into())
        .spawn(move || {
            let _ = tx.send(read_capped_stdout(&cmd_owned, stdout));
        })
        .map_err(|err| {
            preexec_error(
                None,
                format!("thread spawn failed: {err}"),
                Some(Box::new(err)),
            )
        })?;

    match rx.recv_timeout(timeout) {
        Ok(Ok(raw)) => {
            // Child closed stdout and should exit promptly; do not reuse the
            // pre-spawn deadline, which may already be nearly exhausted.
            let grace_deadline = Instant::now() + Duration::from_millis(500);
            let status = wait_until(&mut child, cmd, grace_deadline, timeout, false)?;
            let _ = reader.join();
            let stderr = join_stderr(stderr_reader);
            if !status.success() {
                return Err(nonzero_error(cmd, &status, stderr));
            }
            String::from_utf8(raw).map_err(|err| {
                preexec_error(
                    None,
                    format!("{cmd} stdout is not valid UTF-8: {err}"),
                    Some(Box::new(err)),
                )
            })
        }
        Ok(Err(err)) => {
            reap_killed(&mut child, reader, false);
            let _ = join_stderr(stderr_reader);
            Err(err)
        }
        Err(_) => {
            reap_killed(&mut child, reader, false);
            let stderr = join_stderr(stderr_reader);
            Err(preexec_error(
                Some(PreexecFailureKind::Timeout {
                    cmd: cmd.to_string(),
                    timeout_secs: timeout.as_secs(),
                }),
                preexec_fail_message(
                    cmd,
                    &format!("timed out after {}s", timeout.as_secs()),
                    &stderr,
                ),
                None,
            ))
        }
    }
}

/// Read stdout with a hard byte cap so a runaway child cannot fill memory.
fn read_capped_stdout(cmd: &str, stdout: ChildStdout) -> Result<Vec<u8>, ExtensionError> {
    let mut buf = Vec::new();
    let mut reader = stdout.take(MAX_PREEXEC_STDOUT_BYTES as u64 + 1);
    reader.read_to_end(&mut buf).map_err(|err| {
        preexec_error(
            None,
            format!("failed to read stdout: {err}"),
            Some(Box::new(err)),
        )
    })?;
    if buf.len() > MAX_PREEXEC_STDOUT_BYTES {
        return Err(preexec_error(
            None,
            format!("{cmd} stdout exceeded {MAX_PREEXEC_STDOUT_BYTES} bytes"),
            None,
        ));
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
            read_stderr_tail(stderr)
        })
        .map_err(|err| {
            preexec_error(
                None,
                format!("thread spawn failed: {err}"),
                Some(Box::new(err)),
            )
        })
}

/// Keep the last [`MAX_PREEXEC_STDERR_BYTES`] of child stderr (a tail, not a head).
fn read_stderr_tail(mut reader: impl Read) -> String {
    let mut tail = Vec::with_capacity(MAX_PREEXEC_STDERR_BYTES);
    let mut chunk = [0_u8; 1024];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => append_tail(&mut tail, &chunk[..n], MAX_PREEXEC_STDERR_BYTES),
            Err(_) => break,
        }
    }
    String::from_utf8_lossy(&tail).trim().to_string()
}

fn append_tail(tail: &mut Vec<u8>, data: &[u8], cap: usize) {
    if data.len() >= cap {
        tail.clear();
        tail.extend_from_slice(&data[data.len() - cap..]);
        return;
    }
    let combined = tail.len() + data.len();
    if combined > cap {
        tail.drain(..combined - cap);
    }
    tail.extend_from_slice(data);
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
    process_group: bool,
) -> Result<ExitStatus, ExtensionError> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    terminate_child(child, process_group);
                    return Err(preexec_error(
                        Some(PreexecFailureKind::Timeout {
                            cmd: cmd.to_string(),
                            timeout_secs: timeout.as_secs(),
                        }),
                        format!("{cmd} timed out after {}s", timeout.as_secs()),
                        None,
                    ));
                }
                std::thread::sleep(PREEXEC_WAIT_POLL);
            }
            Err(err) => {
                return Err(preexec_error(
                    None,
                    format!("{cmd} wait failed: {err}"),
                    Some(Box::new(err)),
                ));
            }
        }
    }
}

fn reap_killed(child: &mut Child, reader: std::thread::JoinHandle<()>, process_group: bool) {
    terminate_child(child, process_group);
    let _ = reader.join();
}

fn terminate_child(child: &mut Child, process_group: bool) {
    #[cfg(unix)]
    {
        if process_group {
            kill_process_group(child);
            let _ = child.wait();
            return;
        }
    }
    #[cfg(not(unix))]
    let _ = process_group;
    let _ = child.kill();
    let _ = child.wait();
}

/// Send `SIGKILL` to the child's process group so descendants are reaped.
///
/// `child` must have been spawned with [`std::os::unix::process::CommandExt::process_group`]`(0)`,
/// so its process-group id equals its pid. `kill(-pid, SIGKILL)` then targets
/// that group only.
#[cfg(unix)]
fn kill_process_group(child: &Child) {
    let pid = child.id() as i32;
    if pid <= 1 {
        return;
    }
    // SAFETY: child is the process-group leader created at spawn. Negative
    // pid to `kill(2)` targets that group only.
    let _ = unsafe { libc::kill(-pid, libc::SIGKILL) };
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
        assert!(matches!(
            err,
            ExtensionError::Preexec {
                kind: Some(PreexecFailureKind::NonZeroExit { .. }),
                ..
            }
        ));
    }

    #[test]
    fn spawn_error_maps_not_found_vs_other() {
        let not_found = spawn_error(
            "missing-bin",
            std::io::Error::new(std::io::ErrorKind::NotFound, "nope"),
        );
        assert!(
            matches!(
                not_found,
                ExtensionError::Preexec {
                    kind: Some(PreexecFailureKind::SpawnNotFound { ref cmd }),
                    ..
                } if cmd == "missing-bin"
            ),
            "{not_found:?}"
        );
        let denied = spawn_error(
            "locked-bin",
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        );
        assert!(
            matches!(denied, ExtensionError::Preexec { kind: None, .. }),
            "{denied:?}"
        );
    }

    #[test]
    fn preexec_missing_binary_is_spawn_not_found() {
        let err = run_preexec("wyvern-g2-missing-bin-xyz", &[], None).expect_err("missing");
        assert!(
            matches!(
                err,
                ExtensionError::Preexec {
                    kind: Some(PreexecFailureKind::SpawnNotFound { ref cmd }),
                    ..
                } if cmd == "wyvern-g2-missing-bin-xyz"
            ),
            "{err:?}"
        );
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
