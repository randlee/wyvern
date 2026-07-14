//! `wyvern browsers list|refresh` — inspect / rebuild the local browser registry.

use wyvern_host::{
    browser_registry_path, list_browser_entries, refresh_browser_registry, BrowserRegistryEntry,
    HostError,
};

use crate::error::{emit_host_error, EmitError};

/// Run a `browsers` subcommand; returns stdout text on success.
///
/// # Errors
///
/// Returns structured stderr + exit code on registry failure.
pub fn run_browsers_command(args: &[String]) -> Result<String, BrowsersError> {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => list(),
        "refresh" => refresh(),
        other => Err(BrowsersError::Usage {
            message: format!(
                "unknown browsers subcommand '{other}'\nUsage: wyvern browsers list|refresh"
            ),
        }),
    }
}

/// CLI browsers subcommand failure.
#[derive(Debug)]
pub enum BrowsersError {
    /// Bad argv.
    Usage {
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
