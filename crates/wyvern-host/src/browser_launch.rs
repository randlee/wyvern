//! System + named browser launch (not embedded — CLI owns `wyvern-viewer`).

use crate::browser_registry::{self, BrowserRegistryEntry};
use crate::error::HostError;
use crate::options::{BrowserId, ViewerMode};

/// Open `url` for `system` or a named registry browser.
///
/// # Errors
///
/// Returns [`HostError::ViewerNotFound`] when a named browser is missing, or
/// [`HostError::Internal`] when the OS open / spawn fails.
pub fn launch(mode: &ViewerMode, url: &str) -> Result<(), HostError> {
    match mode {
        ViewerMode::System => launch_system(url),
        ViewerMode::Named(id) => launch_named(*id, url),
        ViewerMode::None | ViewerMode::Embedded => Ok(()),
    }
}

fn launch_system(url: &str) -> Result<(), HostError> {
    webbrowser::open(url).map_err(|e| HostError::Internal {
        message: format!("failed to open system browser: {e}"),
    })?;
    Ok(())
}

fn launch_named(id: BrowserId, url: &str) -> Result<(), HostError> {
    let path = browser_registry::registry_path();
    let entry = browser_registry::resolve_named(id, &path)?;
    spawn_browser(&entry, url)
}

fn spawn_browser(entry: &BrowserRegistryEntry, url: &str) -> Result<(), HostError> {
    std::process::Command::new(&entry.executable)
        .arg(url)
        .spawn()
        .map_err(|e| HostError::Internal {
            message: format!(
                "failed to launch '{}' ({}): {e}",
                entry.id,
                entry.executable.display()
            ),
        })?;
    Ok(())
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
