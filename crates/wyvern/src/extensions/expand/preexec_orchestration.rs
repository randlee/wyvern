//! Create tmpdir, run preexec, expand, and validate the resulting command.

use std::path::{Path, PathBuf};

use crate::extensions::preexec::{create_tmpdir, first_rendered_html, run_preexec, tmpdir_path};
use crate::extensions::{ExtensionDef, ExtensionError, TemplateErrorKind};

use super::template::{references_rendered_basename, references_tmpdir};
use super::{expand_command_host, expand_preexec_args, ExpandedInvocation, MatchContext};

// Thread-local storage for the last created tmpdir path.
// Used only in tests via `last_created_tmpdir()` to verify cleanup behaviour
// without exposing `TempDir` handles across API boundaries.
// Interior mutability is required because the test hook must write to this
// slot inside `expand_and_validate` which takes `&ExtensionDef` (non-mut).
// Production code never reads this slot; it is populated only when preexec
// creates a tmpdir, and tests call `last_created_tmpdir()` after the fact.
// Kept `pub` (not `#[cfg(test)]`) so integration-test binaries can call it.
thread_local! {
    static LAST_CREATED_TMPDIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Path of the temp dir created by the last [`expand_and_validate`] on this thread.
///
/// Integration tests use this hook; it is not part of the supported public API.
#[doc(hidden)]
#[must_use]
pub fn last_created_tmpdir() -> Option<PathBuf> {
    LAST_CREATED_TMPDIR.with(|cell| cell.borrow().clone())
}

fn ensure_preexec_output_parents(args: &[String]) -> Result<(), ExtensionError> {
    for window in args.windows(2) {
        if window[0] == "--output" {
            if let Some(parent) = Path::new(&window[1]).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|err| ExtensionError::Io {
                        message: format!(
                            "could not create preexec output parent '{}': {err}",
                            parent.display()
                        ),
                        source: Some(Box::new(err)),
                    })?;
                }
            }
        }
    }
    Ok(())
}

/// Create tmpdir if needed, run preexec, expand, and validate.
///
/// On preexec failure the temp dir is dropped immediately (no host launch).
/// On success `temp_guard` is held until the caller drops [`ExpandedInvocation`].
/// CLI `--help` / `-h` never reach this function — [`crate::extensions::match_extension_help`]
/// handles skill cards before match and expand.
///
/// # Errors
///
/// Returns [`ExtensionError`] for preexec, template, I/O, or validation failure.
pub fn expand_and_validate(
    ext: &ExtensionDef,
    ctx: &MatchContext<'_>,
) -> Result<ExpandedInvocation, ExtensionError> {
    let mut ctx = ctx.clone();
    let temp_guard = if references_tmpdir(ext) {
        let dir = create_tmpdir()?;
        ctx.tmpdir = Some(tmpdir_path(&dir));
        LAST_CREATED_TMPDIR.with(|cell| {
            *cell.borrow_mut() = ctx.tmpdir.clone();
        });
        Some(dir)
    } else {
        None
    };

    if let Some(pre) = ext.preexec.as_ref() {
        let (cmd, args) = match expand_preexec_args(pre, ext, &ctx) {
            Ok(pair) => pair,
            Err(err) => {
                drop(temp_guard);
                return Err(err);
            }
        };
        ensure_preexec_output_parents(&args)?;
        let stdout_capture = ext.preexec.as_ref().and_then(|p| p.stdout);
        match run_preexec(&cmd, &args, stdout_capture) {
            Ok(stdout) => ctx.preexec_stdout = stdout,
            Err(err) => {
                drop(temp_guard);
                return Err(err);
            }
        }
        if references_rendered_basename(ext) {
            let tmp = ctx.tmpdir.as_deref().ok_or_else(|| {
                ExtensionError::template(
                    TemplateErrorKind::Unavailable,
                    "{rendered_basename} requires {tmpdir}",
                )
            })?;
            match first_rendered_html(tmp) {
                Ok(name) => ctx.rendered_basename = Some(name),
                Err(err) => {
                    drop(temp_guard);
                    return Err(err);
                }
            }
        }
    }

    let (command, host_overrides) = match expand_command_host(ext, &ctx) {
        Ok(pair) => pair,
        Err(err) => {
            drop(temp_guard);
            return Err(err);
        }
    };
    if let Err(source) = wyvern_schema::validate(&command) {
        drop(temp_guard);
        return Err(ExtensionError::InvalidCommand { source });
    }
    Ok(ExpandedInvocation {
        command,
        host_overrides,
        temp_guard,
    })
}
