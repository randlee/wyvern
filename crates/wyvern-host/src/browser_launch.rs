//! System + named browser launch (not embedded — CLI owns `wyvern-viewer`).

use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use crate::browser_registry::{self, BrowserRegistryEntry};
use crate::error::HostError;
use crate::options::{BrowserId, ViewerMode};

/// Brief window to detect instant-exit named browser launches.
const NAMED_LAUNCH_PROBE: Duration = Duration::from_millis(150);

/// Open `url` for `system` or a named registry browser.
///
/// # Errors
///
/// Returns [`HostError::ViewerNotFound`] when a named browser is missing, or
/// [`HostError::Internal`] / [`HostError::Registry`] when the OS open / spawn fails.
pub fn launch(mode: &ViewerMode, url: &str) -> Result<(), HostError> {
    match mode {
        ViewerMode::System => launch_system(url),
        ViewerMode::Named(id) => launch_named(*id, url),
        ViewerMode::None | ViewerMode::Embedded => Ok(()),
    }
}

fn launch_system(url: &str) -> Result<(), HostError> {
    tracing::info!(
        viewer_mode = "system",
        url_host = %redact_url_host(url),
        "launching system browser"
    );
    match webbrowser::open(url) {
        Ok(()) => {
            tracing::info!(
                viewer_mode = "system",
                outcome = "ok",
                "system browser launched"
            );
            Ok(())
        }
        Err(e) => {
            tracing::error!(
                viewer_mode = "system",
                outcome = "error",
                error = %e,
                "system browser launch failed"
            );
            Err(HostError::Internal {
                message: format!("failed to open system browser: {e}"),
            })
        }
    }
}

fn launch_named(id: BrowserId, url: &str) -> Result<(), HostError> {
    let path = browser_registry::registry_path();
    tracing::info!(
        viewer_mode = "named",
        browser_id = %id,
        url_host = %redact_url_host(url),
        "launching named browser"
    );
    let entry = browser_registry::resolve_named(id, &path)?;
    match spawn_browser_supervised(&entry, url) {
        Ok(()) => {
            tracing::info!(
                viewer_mode = "named",
                browser_id = %id,
                outcome = "ok",
                "named browser launched"
            );
            Ok(())
        }
        Err(first_err) => {
            tracing::warn!(
                viewer_mode = "named",
                browser_id = %id,
                outcome = "retry",
                error = %first_err,
                "named browser exited immediately; refreshing registry and retrying once"
            );
            let _ = browser_registry::refresh(&path)?;
            let entry = browser_registry::resolve_named(id, &path)?;
            match spawn_browser_supervised(&entry, url) {
                Ok(()) => {
                    tracing::info!(
                        viewer_mode = "named",
                        browser_id = %id,
                        outcome = "ok_after_retry",
                        "named browser launched after refresh"
                    );
                    Ok(())
                }
                Err(err) => {
                    tracing::error!(
                        viewer_mode = "named",
                        browser_id = %id,
                        outcome = "error",
                        error = %err,
                        "named browser launch failed after retry"
                    );
                    Err(err)
                }
            }
        }
    }
}

fn spawn_browser_supervised(entry: &BrowserRegistryEntry, url: &str) -> Result<(), HostError> {
    let mut child = spawn_browser_child(entry, url)?;
    thread::sleep(NAMED_LAUNCH_PROBE);
    match child.try_wait() {
        Ok(Some(status)) => Err(HostError::Internal {
            message: format!(
                "browser '{}' ({}) exited immediately with {status}; run `wyvern browsers refresh` or use --viewer system",
                entry.id,
                entry.executable.display()
            ),
        }),
        Ok(None) => {
            // Still running — detach (caller does not own long-lived browser lifetime).
            std::mem::forget(child);
            Ok(())
        }
        Err(e) => Err(HostError::Internal {
            message: format!(
                "failed to poll browser '{}' ({}): {e}",
                entry.id,
                entry.executable.display()
            ),
        }),
    }
}

fn spawn_browser_child(entry: &BrowserRegistryEntry, url: &str) -> Result<Child, HostError> {
    Command::new(&entry.executable)
        .arg(url)
        .spawn()
        .map_err(|e| HostError::Internal {
            message: format!(
                "failed to launch '{}' ({}): {e}",
                entry.id,
                entry.executable.display()
            ),
        })
}

fn redact_url_host(url: &str) -> String {
    url.split('/')
        .nth(2)
        .unwrap_or("unknown")
        .split(':')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_none_and_embedded_are_noops() {
        launch(&ViewerMode::None, "http://127.0.0.1/").expect("none");
        launch(&ViewerMode::Embedded, "http://127.0.0.1/").expect("embedded");
    }
}
