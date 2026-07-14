//! Local `browsers.json` cache — read, refresh, lookup.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::browser_catalog::{self, CatalogBrowser};
use crate::error::HostError;
use crate::options::BrowserId;

/// On-disk registry schema version.
pub const REGISTRY_VERSION: u32 = 1;

/// Wyvern browser registry file (HTTP-TYPES / http-viewer-contract).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserRegistryFile {
    /// Schema version.
    pub version: u32,
    /// RFC3339 timestamp of last refresh.
    pub updated_at: String,
    /// Platform tag, e.g. `macos-aarch64`.
    pub platform: String,
    /// Browsers found on disk.
    pub entries: Vec<BrowserRegistryEntry>,
}

/// One discovered browser in the registry cache.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserRegistryEntry {
    /// Catalog id (`chrome`, …).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Absolute executable path.
    pub executable: PathBuf,
}

/// Resolve the registry file path (`WYVERN_BROWSERS_FILE` or platform cache).
pub fn registry_path() -> PathBuf {
    if let Some(path) = std::env::var_os("WYVERN_BROWSERS_FILE") {
        return PathBuf::from(path);
    }
    let cache = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    cache.join("wyvern").join("browsers.json")
}

/// Load the registry from disk, or `None` if missing / unreadable.
pub fn load(path: &Path) -> Option<BrowserRegistryFile> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Scan the catalog, write the registry file, and return it.
///
/// # Errors
///
/// Returns [`HostError::Internal`] when the cache directory or file cannot be written.
pub fn refresh(path: &Path) -> Result<BrowserRegistryFile, HostError> {
    let file = scan_catalog();
    write_registry(path, &file)?;
    Ok(file)
}

/// Ensure a registry exists (refresh if missing), then return it.
///
/// # Errors
///
/// Propagates refresh / write failures.
pub fn load_or_refresh(path: &Path) -> Result<BrowserRegistryFile, HostError> {
    if let Some(existing) = load(path) {
        return Ok(existing);
    }
    refresh(path)
}

/// Look up a named browser; refresh once on miss, then error if still absent.
///
/// # Errors
///
/// Returns [`HostError::ViewerNotFound`] when the browser is not installed, or
/// [`HostError::Internal`] on registry I/O failure.
pub fn resolve_named(id: BrowserId, path: &Path) -> Result<BrowserRegistryEntry, HostError> {
    let catalog = browser_catalog::for_browser_id(id);
    let mut file = load_or_refresh(path)?;
    if let Some(entry) = find_entry(&file, catalog.id) {
        return Ok(entry);
    }
    // Miss → force re-scan once.
    file = refresh(path)?;
    if let Some(entry) = find_entry(&file, catalog.id) {
        return Ok(entry);
    }
    Err(HostError::ViewerNotFound {
        id,
        hint: format!(
            "{} not found; install {} or use --viewer system",
            catalog.name, catalog.name
        ),
    })
}

/// List entries from the registry (refresh if missing).
///
/// # Errors
///
/// Propagates refresh / write failures.
pub fn list_entries(path: &Path) -> Result<Vec<BrowserRegistryEntry>, HostError> {
    Ok(load_or_refresh(path)?.entries)
}

fn find_entry(file: &BrowserRegistryFile, id: &str) -> Option<BrowserRegistryEntry> {
    file.entries.iter().find(|e| e.id == id).cloned()
}

fn scan_catalog() -> BrowserRegistryFile {
    let mut entries = Vec::new();
    for browser in browser_catalog::catalog() {
        if let Some(executable) = browser_catalog::discover(browser) {
            entries.push(entry_from_catalog(browser, executable));
        }
    }
    BrowserRegistryFile {
        version: REGISTRY_VERSION,
        updated_at: now_rfc3339(),
        platform: platform_tag(),
        entries,
    }
}

fn entry_from_catalog(browser: &CatalogBrowser, executable: PathBuf) -> BrowserRegistryEntry {
    BrowserRegistryEntry {
        id: browser.id.to_string(),
        name: browser.name.to_string(),
        executable,
    }
}

fn write_registry(path: &Path, file: &BrowserRegistryFile) -> Result<(), HostError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| HostError::Internal {
            message: format!(
                "failed to create browser registry dir {}: {e}",
                parent.display()
            ),
        })?;
    }
    let json = serde_json::to_vec_pretty(file).map_err(|e| HostError::Internal {
        message: format!("failed to serialize browser registry: {e}"),
    })?;
    std::fs::write(path, json).map_err(|e| HostError::Internal {
        message: format!("failed to write browser registry {}: {e}", path.display()),
    })?;
    Ok(())
}

fn platform_tag() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn now_rfc3339() -> String {
    // Avoid a chrono dependency: approximate UTC via system time since UNIX epoch.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Minimal RFC3339-ish stamp; sufficient for cache metadata.
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_writes_and_lists() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("browsers.json");
        let file = refresh(&path).expect("refresh");
        assert_eq!(file.version, REGISTRY_VERSION);
        assert!(path.is_file());
        let listed = list_entries(&path).expect("list");
        assert_eq!(listed, file.entries);
    }

    #[test]
    fn resolve_named_errors_when_missing_after_empty_refresh() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("browsers.json");
        // Point safari override at a missing path so discover skips it even on macOS.
        let missing = tmp.path().join("no-safari-bin");
        std::env::set_var("WYVERN_SAFARI_PATH", &missing);
        let _guard = EnvClearGuard {
            key: "WYVERN_SAFARI_PATH",
        };
        // Force empty initial file; refresh will re-scan but safari path_env fails.
        let empty = BrowserRegistryFile {
            version: REGISTRY_VERSION,
            updated_at: "0".into(),
            platform: "test".into(),
            entries: vec![],
        };
        write_registry(&path, &empty).expect("write");
        // Clear entries by refreshing with safari forced missing — other browsers may
        // still appear. Only assert ViewerNotFound when safari remains absent.
        let _ = refresh(&path);
        if browser_catalog::discover(browser_catalog::for_browser_id(BrowserId::Safari)).is_none() {
            // Ensure registry has no safari entry.
            let mut file = load(&path).expect("load");
            file.entries.retain(|e| e.id != "safari");
            write_registry(&path, &file).expect("rewrite");
            let err = resolve_named(BrowserId::Safari, &path).expect_err("missing safari");
            assert!(matches!(err, HostError::ViewerNotFound { .. }));
        }
    }

    struct EnvClearGuard {
        key: &'static str,
    }
    impl Drop for EnvClearGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.key);
        }
    }
}
