//! `wyvern browsers list|refresh` — inspect / rebuild the local browser registry.

use wyvern_host::{
    browser_registry_path, list_browser_entries, refresh_browser_registry, BrowserRegistryEntry,
    HostError,
};

use crate::error::{emit_host_error, EmitError, UsageErrorKind};

/// Usage text for `wyvern browsers --help` / `-h`.
#[must_use]
pub fn browsers_usage_message() -> String {
    concat!(
        "Usage: wyvern browsers list|refresh\n",
        "       wyvern browsers --help\n",
        "\n",
        "Commands:\n",
        "  list      List browsers in the local registry\n",
        "  refresh   Rebuild the local browser registry\n",
        "\n",
        "See also: wyvern --help\n",
    )
    .to_string()
}

/// Run a `browsers` subcommand; returns stdout text on success.
///
/// # Errors
///
/// Returns structured stderr + exit code on registry failure.
pub fn run_browsers_command(args: &[String]) -> Result<String, BrowsersError> {
    if args
        .first()
        .is_some_and(|token| token == "--help" || token == "-h")
    {
        return Ok(browsers_usage_message());
    }
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => list(),
        "refresh" => refresh(),
        other => Err(BrowsersError::Usage {
            kind: UsageErrorKind::UnknownSubcommand {
                domain: "browsers".into(),
                token: other.to_string(),
            },
            message: format!(
                "unknown browsers subcommand '{other}'\n{}",
                browsers_usage_message()
            ),
        }),
    }
}

/// CLI browsers subcommand failure.
#[derive(Debug)]
pub enum BrowsersError {
    /// Bad argv.
    Usage {
        /// Discriminated usage class for structured stderr recovery.
        kind: UsageErrorKind,
        /// Plain-text usage.
        message: String,
    },
    /// Registry / host failure with stderr JSON.
    Stage {
        /// Stderr JSON.
        stderr: String,
        /// Process exit code.
        exit_code: i32,
    },
    /// Emit-boundary serialize failure.
    Emit(EmitError),
}

fn list() -> Result<String, BrowsersError> {
    let path = browser_registry_path();
    let entries = list_browser_entries(&path).map_err(map_host)?;
    Ok(format_entries(&entries))
}

fn refresh() -> Result<String, BrowsersError> {
    let path = browser_registry_path();
    let file = refresh_browser_registry(&path).map_err(map_host)?;
    Ok(format!(
        "Refreshed {} ({} entries)\n{}",
        path.display(),
        file.entries.len(),
        format_entries(&file.entries)
    ))
}

fn format_entries(entries: &[BrowserRegistryEntry]) -> String {
    if entries.is_empty() {
        return "No browsers found in registry.\nRun: wyvern browsers refresh".into();
    }
    let mut out = String::new();
    for e in entries {
        out.push_str(&format!(
            "{:<10}  {:<20}  {}\n",
            e.id,
            e.name,
            e.executable.display()
        ));
    }
    out
}

fn map_host(err: HostError) -> BrowsersError {
    match emit_host_error(&err) {
        Ok(stderr) => {
            let exit_code = match &err {
                HostError::Bind { .. } => wyvern_schema::ErrorCode::HostBindError.exit_code(),
                HostError::ViewerNotFound { .. } | HostError::ViewerUnsupported { .. } => {
                    wyvern_schema::ErrorCode::HostViewerError.exit_code()
                }
                _ => wyvern_schema::ErrorCode::HostError.exit_code(),
            };
            BrowsersError::Stage { stderr, exit_code }
        }
        Err(e) => BrowsersError::Emit(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browsers_help_mentions_list_and_refresh() {
        let text = browsers_usage_message();
        assert!(text.contains("list"), "{text}");
        assert!(text.contains("refresh"), "{text}");
    }

    #[test]
    fn unknown_browsers_subcommand_is_discriminated() {
        let err = run_browsers_command(&["nope".into()]).expect_err("usage");
        match err {
            BrowsersError::Usage { kind, message } => {
                assert!(matches!(
                    kind,
                    UsageErrorKind::UnknownSubcommand { ref domain, ref token }
                        if domain == "browsers" && token == "nope"
                ));
                assert!(message.contains("unknown browsers subcommand"), "{message}");
            }
            other => panic!("expected Usage, got {other:?}"),
        }
    }
}
