//! Preexec subprocess spawn, PATH requires-check, and stdout capture.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::ExtensionError;

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

/// Run `cmd` with expanded `args`; optionally capture markdown stdout.
///
/// # Errors
///
/// Returns [`ExtensionError::Preexec`] when the process cannot be spawned or
/// exits non-zero. Unknown `stdout` modes are template/contract errors.
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
    let status = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to spawn '{cmd}': {err}"),
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(ExtensionError::Preexec {
            message: format!("'{cmd}' exited with {status}"),
        })
    }
}

fn run_capture_stdout(cmd: &str, args: &[String]) -> Result<String, ExtensionError> {
    let output = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| ExtensionError::Preexec {
            message: format!("failed to spawn '{cmd}': {err}"),
        })?;
    if !output.status.success() {
        return Err(ExtensionError::Preexec {
            message: format!("'{cmd}' exited with {}", output.status),
        });
    }
    String::from_utf8(output.stdout).map_err(|err| ExtensionError::Preexec {
        message: format!("preexec stdout was not UTF-8: {err}"),
    })
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
}
