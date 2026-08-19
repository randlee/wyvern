//! Wizard workflow hooks and `next_wizard` chain loop (REQ-0124–0126).
//!
//! Spawn, timeout, and stderr-tail go through [`crate::extensions::run_script`]
//! only — this module must not call `std::process::Command::new` (ADR-0023).

mod chain;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::extensions::{binary_on_path, run_script, ScriptError, ScriptRequest};

#[doc(inline)]
pub use chain::{merge_wizard_config, resolve_next_wizard, NextInvocation};

/// Timeout for every workflow pre/post script (REQ-0124 / REQ-0125).
pub const WORKFLOW_SCRIPT_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum wizard sessions in one `next_wizard` chain (REQ-0126).
pub const NEXT_WIZARD_MAX_DEPTH: u32 = 16;

/// Allowlisted roots for workflow script and `next_wizard` paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allowlist {
    /// Resolved `{wyvern_share}` directory.
    pub share_root: PathBuf,
    /// Process working directory.
    pub cwd: PathBuf,
    /// Directory of the current `wizard.json`.
    pub wizard_dir: PathBuf,
}

impl Allowlist {
    /// Expand `{wyvern_share}`, canonicalize, and reject `..` / symlink escape.
    ///
    /// Relative paths try `{wyvern_share}`, then cwd, then the current wizard
    /// directory.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError::PathDenied`] when the resolved path escapes the
    /// allowlist, or [`WorkflowError::Resolve`] when expansion / lookup fails.
    pub fn resolve_allowed(&self, raw: &str) -> Result<PathBuf, WorkflowError> {
        let expanded = expand_wyvern_share(raw, &self.share_root);
        let candidate = PathBuf::from(&expanded);
        let roots = self.canonical_roots();

        let tries: Vec<PathBuf> = if candidate.is_absolute() {
            vec![candidate]
        } else {
            vec![
                self.share_root.join(&expanded),
                self.cwd.join(&expanded),
                self.wizard_dir.join(&expanded),
            ]
        };

        let mut saw_escape = false;
        for try_path in tries {
            let lexical = lexical_normalize(&try_path);
            if !is_under_any(&lexical, &self.lexical_roots()) && !is_under_any(&lexical, &roots) {
                saw_escape = true;
                continue;
            }
            match std::fs::canonicalize(&try_path) {
                Ok(canon) => {
                    if is_under_any(&canon, &roots) {
                        return Ok(canon);
                    }
                    saw_escape = true;
                }
                Err(_) => {
                    if is_under_any(&lexical, &self.lexical_roots()) {
                        return Err(WorkflowError::Resolve {
                            path: raw.to_string(),
                            cause: format!("path does not exist: {}", try_path.display()),
                        });
                    }
                    saw_escape = true;
                }
            }
        }

        if saw_escape {
            Err(WorkflowError::PathDenied {
                path: PathBuf::from(expanded),
            })
        } else {
            Err(WorkflowError::Resolve {
                path: raw.to_string(),
                cause: "could not resolve path against share, cwd, or wizard directory".into(),
            })
        }
    }

    fn canonical_roots(&self) -> Vec<PathBuf> {
        [&self.share_root, &self.cwd, &self.wizard_dir]
            .into_iter()
            .filter_map(|p| std::fs::canonicalize(p).ok())
            .collect()
    }

    fn lexical_roots(&self) -> Vec<PathBuf> {
        vec![
            lexical_normalize(&self.share_root),
            lexical_normalize(&self.cwd),
            lexical_normalize(&self.wizard_dir),
        ]
    }
}

/// Runs `workflow.pre` / `workflow.post` through Phase F preexec helpers.
#[derive(Debug, Clone)]
pub struct WorkflowRunner {
    /// Path allowlist for this hop.
    pub allowlist: Allowlist,
    /// Script timeout (normally [`WORKFLOW_SCRIPT_TIMEOUT`]).
    pub timeout: Duration,
}

impl WorkflowRunner {
    /// Run `spec.pre` if present and deep-merge `config_patch` into `config`.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError`] on allowlist, spawn, timeout, nonzero, or
    /// invalid stdout.
    pub fn run_pre(
        &self,
        spec: &wyvern_schema::WorkflowSpec,
        config: &mut Value,
        dry_run: bool,
    ) -> Result<(), WorkflowError> {
        let Some(raw) = spec.pre.as_deref() else {
            return Ok(());
        };
        let stdout = self.spawn_script(raw, None, true, dry_run)?;
        let patch = parse_config_patch(&stdout)?;
        *config = merge_wizard_config(
            config.clone(),
            Value::Object(Default::default()),
            Some(patch),
        )?;
        Ok(())
    }

    /// Run `spec.post` if present, sending `finish` JSON on stdin.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError`] on allowlist, spawn, timeout, or nonzero exit.
    pub fn run_post(
        &self,
        spec: &wyvern_schema::WorkflowSpec,
        finish: &Value,
        dry_run: bool,
    ) -> Result<(), WorkflowError> {
        let Some(raw) = spec.post.as_deref() else {
            return Ok(());
        };
        let stdin = serde_json::to_vec(finish).map_err(|err| WorkflowError::InvalidStdout {
            cause: format!("could not serialize finish JSON for post stdin: {err}"),
        })?;
        self.spawn_script(raw, Some(stdin), false, dry_run)?;
        Ok(())
    }

    fn spawn_script(
        &self,
        raw: &str,
        stdin: Option<Vec<u8>>,
        capture_stdout: bool,
        dry_run: bool,
    ) -> Result<String, WorkflowError> {
        let canonical = self.allowlist.resolve_allowed(raw)?;
        let mut argv = script_argv(&canonical)?;
        if dry_run {
            argv.push(OsString::from("--dry-run"));
        }
        let program = argv
            .first()
            .cloned()
            .ok_or_else(|| WorkflowError::Resolve {
                path: raw.to_string(),
                cause: "script argv was empty".into(),
            })?;
        let args = argv.into_iter().skip(1).collect::<Vec<_>>();
        let extra_env = workflow_env(&self.allowlist)?;
        let request = ScriptRequest {
            program,
            args,
            cwd: Some(self.allowlist.cwd.clone()),
            extra_env,
            stdin,
            capture_stdout,
            timeout: self.timeout,
            process_group: true,
        };
        let output = run_script(&request).map_err(map_script_error)?;
        if !output.status.success() {
            return Err(WorkflowError::NonZero {
                status: output.status.code().unwrap_or(1),
                stderr_tail: output.stderr_tail,
            });
        }
        Ok(output.stdout.unwrap_or_default())
    }
}

/// Fail when `hop` exceeds [`NEXT_WIZARD_MAX_DEPTH`].
///
/// # Errors
///
/// Returns [`WorkflowError::ChainDepth`] when `hop` is 17 or greater.
pub fn check_chain_depth(hop: u32) -> Result<(), WorkflowError> {
    if hop > NEXT_WIZARD_MAX_DEPTH {
        Err(WorkflowError::ChainDepth {
            max: NEXT_WIZARD_MAX_DEPTH,
        })
    } else {
        Ok(())
    }
}

/// Workflow / chain failure (stderr via [`wyvern_schema::ErrorCode::WorkflowError`]).
#[derive(Debug)]
pub enum WorkflowError {
    /// Path escaped `{wyvern_share}`, cwd, or the current wizard directory.
    PathDenied {
        /// Offending path after expansion.
        path: PathBuf,
    },
    /// Script exceeded 30s.
    Timeout {
        /// Last 4 KiB of child stderr collected before kill.
        stderr_tail: String,
    },
    /// Script exit status was not zero.
    NonZero {
        /// Process exit code (or `1` when killed by signal).
        status: i32,
        /// Last 4 KiB of child stderr.
        stderr_tail: String,
    },
    /// Pre stdout was not one JSON object with an object `config_patch`.
    InvalidStdout {
        /// Parse / shape failure detail.
        cause: String,
    },
    /// A 17th hop was requested.
    ChainDepth {
        /// Configured maximum (`16`).
        max: u32,
    },
    /// `{wyvern_share}` / relative path could not be resolved.
    Resolve {
        /// Original path string.
        path: String,
        /// Why resolution failed.
        cause: String,
    },
    /// `.py` script and `python3` is not on PATH.
    MissingPython3,
    /// `input` or `config_patch` was not a JSON object.
    Merge {
        /// Merge failure detail.
        cause: String,
    },
}

impl std::fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathDenied { path } => {
                write!(f, "workflow path denied: {}", path.display())
            }
            Self::Timeout { stderr_tail } => {
                if stderr_tail.is_empty() {
                    f.write_str("workflow script timed out after 30s")
                } else {
                    write!(f, "workflow script timed out after 30s: {stderr_tail}")
                }
            }
            Self::NonZero {
                status,
                stderr_tail,
            } => {
                if stderr_tail.is_empty() {
                    write!(f, "workflow script exited with status {status}")
                } else {
                    write!(
                        f,
                        "workflow script exited with status {status}: {stderr_tail}"
                    )
                }
            }
            Self::InvalidStdout { cause } => write!(f, "invalid workflow pre stdout: {cause}"),
            Self::ChainDepth { max } => {
                write!(f, "next_wizard chain exceeded maximum depth of {max}")
            }
            Self::Resolve { path, cause } => {
                write!(f, "could not resolve workflow path '{path}': {cause}")
            }
            Self::MissingPython3 => f.write_str("python3 is required to run .py workflow scripts"),
            Self::Merge { cause } => write!(f, "workflow config merge failed: {cause}"),
        }
    }
}

impl std::error::Error for WorkflowError {}

impl WorkflowError {
    /// Stable recovery steps for stderr JSON (RBP-001).
    #[must_use]
    pub fn recovery(&self) -> Vec<String> {
        match self {
            Self::PathDenied { .. } => vec![
                "Use a path under {wyvern_share}, the process cwd, or the current wizard.json directory".into(),
            ],
            Self::Timeout { .. } => vec![
                "Shorten the workflow script or raise the timeout only via a later ADR".into(),
            ],
            Self::NonZero { .. } => vec![
                "Fix the script; stderr_tail is in the JSON cause".into(),
            ],
            Self::InvalidStdout { .. } => vec![
                r#"Print { "config_patch": { ... } } only"#.into(),
            ],
            Self::ChainDepth { max } => vec![format!("Keep chains ≤ {max}")],
            Self::Resolve { .. } => vec!["Fix the path string".into()],
            Self::MissingPython3 => vec!["Install Python 3".into()],
            Self::Merge { .. } => vec!["Pass JSON objects for input and config_patch".into()],
        }
    }

    /// Short cause string for the stderr envelope.
    #[must_use]
    pub fn cause(&self) -> String {
        match self {
            Self::PathDenied { path } => {
                format!("path escaped the workflow allowlist: {}", path.display())
            }
            Self::Timeout { stderr_tail } => {
                if stderr_tail.is_empty() {
                    "script exceeded 30s".into()
                } else {
                    format!("script exceeded 30s: {stderr_tail}")
                }
            }
            Self::NonZero { stderr_tail, .. } => {
                if stderr_tail.is_empty() {
                    "script exit was not 0".into()
                } else {
                    stderr_tail.clone()
                }
            }
            Self::InvalidStdout { cause } => cause.clone(),
            Self::ChainDepth { max } => format!("17th hop requested; max is {max}"),
            Self::Resolve { cause, .. } => cause.clone(),
            Self::MissingPython3 => "python3 not found on PATH".into(),
            Self::Merge { cause } => cause.clone(),
        }
    }

    /// Stable sub-discriminator for machine branching under `WORKFLOW_ERROR`.
    #[must_use]
    pub fn subcode(&self) -> &'static str {
        match self {
            Self::PathDenied { .. } => "path_denied",
            Self::Timeout { .. } => "timeout",
            Self::NonZero { .. } => "nonzero",
            Self::InvalidStdout { .. } => "invalid_stdout",
            Self::ChainDepth { .. } => "chain_depth",
            Self::Resolve { .. } => "resolve",
            Self::MissingPython3 => "missing_python3",
            Self::Merge { .. } => "merge",
        }
    }
}

/// Build argv for a canonical script path. `.py` → `python3 <path>`.
fn script_argv(canonical: &Path) -> Result<Vec<OsString>, WorkflowError> {
    if canonical.extension() == Some(OsStr::new("py")) {
        if !binary_on_path("python3") {
            return Err(WorkflowError::MissingPython3);
        }
        Ok(vec![
            OsString::from("python3"),
            canonical.as_os_str().to_os_string(),
        ])
    } else {
        Ok(vec![canonical.as_os_str().to_os_string()])
    }
}

fn workflow_env(allowlist: &Allowlist) -> Result<Vec<(OsString, OsString)>, WorkflowError> {
    let wyvern_bin = resolve_wyvern_bin();
    let repo_root = std::env::var_os("WYVERN_REPO_ROOT")
        .unwrap_or_else(|| allowlist.cwd.clone().into_os_string());
    Ok(vec![
        (
            OsString::from("WYVERN_SHARE"),
            allowlist.share_root.clone().into_os_string(),
        ),
        (OsString::from("WYVERN_REPO_ROOT"), repo_root),
        (OsString::from("WYVERN_BIN"), wyvern_bin),
    ])
}

fn resolve_wyvern_bin() -> OsString {
    match std::env::current_exe() {
        Ok(exe) => std::fs::canonicalize(&exe).unwrap_or(exe).into_os_string(),
        Err(_) => OsString::from("wyvern"),
    }
}

fn expand_wyvern_share(raw: &str, share_root: &Path) -> String {
    raw.replace("{wyvern_share}", &share_root.to_string_lossy())
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn is_under_any(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn parse_config_patch(stdout: &str) -> Result<Value, WorkflowError> {
    let trimmed = stdout.trim();
    let value: Value =
        serde_json::from_str(trimmed).map_err(|err| WorkflowError::InvalidStdout {
            cause: format!("pre stdout is not JSON: {err}"),
        })?;
    let obj = value
        .as_object()
        .ok_or_else(|| WorkflowError::InvalidStdout {
            cause: "pre stdout must be one JSON object".into(),
        })?;
    if obj.len() != 1 || !obj.contains_key("config_patch") {
        return Err(WorkflowError::InvalidStdout {
            cause: "pre stdout must be an object with only config_patch".into(),
        });
    }
    let patch = obj
        .get("config_patch")
        .cloned()
        .ok_or_else(|| WorkflowError::InvalidStdout {
            cause: "pre stdout missing config_patch".into(),
        })?;
    if !patch.is_object() {
        return Err(WorkflowError::InvalidStdout {
            cause: "config_patch must be a JSON object".into(),
        });
    }
    Ok(patch)
}

fn map_script_error(err: ScriptError) -> WorkflowError {
    match err {
        ScriptError::Timeout { stderr_tail, .. } => WorkflowError::Timeout { stderr_tail },
        ScriptError::SpawnNotFound { cmd, .. } if cmd == "python3" => WorkflowError::MissingPython3,
        ScriptError::SpawnNotFound { cmd, source } | ScriptError::Spawn { cmd, source } => {
            WorkflowError::Resolve {
                path: cmd,
                cause: source.to_string(),
            }
        }
        ScriptError::Wait { cmd, source } => WorkflowError::Resolve {
            path: cmd,
            cause: source.to_string(),
        },
        ScriptError::Stdout { cause, .. } => WorkflowError::InvalidStdout { cause },
        ScriptError::Thread { message } => WorkflowError::Resolve {
            path: String::new(),
            cause: message,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_allowlist() -> (tempfile::TempDir, Allowlist) {
        let tmp = tempfile::tempdir().expect("tmp");
        let share = tmp.path().join("share");
        let cwd = tmp.path().join("cwd");
        let wizard = tmp.path().join("wizard");
        std::fs::create_dir_all(&share).unwrap();
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(&wizard).unwrap();
        let allowlist = Allowlist {
            share_root: share,
            cwd,
            wizard_dir: wizard,
        };
        (tmp, allowlist)
    }

    #[test]
    fn resolve_allowed_rejects_escape() {
        let (_tmp, allow) = temp_allowlist();
        let err = allow
            .resolve_allowed("../../../../etc/passwd")
            .expect_err("escape");
        assert!(matches!(err, WorkflowError::PathDenied { .. }), "{err:?}");
    }

    #[test]
    fn check_chain_depth_rejects_seventeenth_hop() {
        check_chain_depth(16).expect("16 ok");
        let err = check_chain_depth(17).expect_err("17");
        assert!(matches!(
            err,
            WorkflowError::ChainDepth {
                max: NEXT_WIZARD_MAX_DEPTH
            }
        ));
    }

    #[test]
    fn parse_config_patch_requires_object() {
        let err = parse_config_patch("[]").expect_err("array");
        assert!(matches!(err, WorkflowError::InvalidStdout { .. }));
        let patch = parse_config_patch(r#"{"config_patch":{"k":1}}"#).expect("ok");
        assert_eq!(patch, json!({"k": 1}));
    }

    #[test]
    fn timeout_cause_includes_stderr_tail() {
        let err = WorkflowError::Timeout {
            stderr_tail: "still running child".into(),
        };
        assert_eq!(err.subcode(), "timeout");
        assert!(err.cause().contains("still running child"));
        assert!(err.to_string().contains("still running child"));
    }
}
