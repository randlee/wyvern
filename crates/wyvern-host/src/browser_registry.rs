//! Local `browsers.json` cache — read, refresh, lookup.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::browser_catalog::{self, CatalogBrowser};
use crate::error::HostError;
use crate::options::BrowserId;

/// On-disk registry schema version.
pub const REGISTRY_VERSION: u32 = 1;

/// Validated catalog browser id (`chrome`, `safari`, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CatalogId(String);

impl CatalogId {
    /// Construct from a catalog wire id.
    ///
    /// # Errors
    ///
    /// Returns [`HostError::Registry`] when `id` is not in the hardcoded catalog.
    pub fn new(id: impl Into<String>) -> Result<Self, HostError> {
        let id = id.into();
        if browser_catalog::find(&id).is_none() {
            return Err(HostError::Registry {
                message: format!("unknown browser catalog id '{id}'"),
            });
        }
        Ok(Self(id))
    }

    /// Borrow the wire id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CatalogId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for CatalogId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

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
    pub id: CatalogId,
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

/// Load the registry from disk.
///
/// Returns `Ok(None)` when the file is missing or fails soft validation (version /
/// platform) so callers can refresh. Corrupt or unreadable files return
/// [`HostError::Registry`].
///
/// # Errors
///
/// Returns [`HostError::Registry`] when the file exists but cannot be read or parsed.
pub fn load(path: &Path) -> Result<Option<BrowserRegistryFile>, HostError> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(HostError::Registry {
                message: format!(
                    "failed to read browser registry {}: {e} (delete the file or run `wyvern browsers refresh`)",
                    path.display()
                ),
            });
        }
    };
    let file: BrowserRegistryFile =
        serde_json::from_slice(&bytes).map_err(|e| HostError::Registry {
            message: format!(
            "corrupt browser registry {}: {e} (delete the file or run `wyvern browsers refresh`)",
            path.display()
        ),
        })?;
    if !registry_shape_ok(&file) {
        return Ok(None);
    }
    Ok(Some(file))
}

fn registry_shape_ok(file: &BrowserRegistryFile) -> bool {
    if file.version != REGISTRY_VERSION {
        return false;
    }
    if file.platform.trim().is_empty() {
        return false;
    }
    file.entries.iter().all(|e| {
        !e.executable.as_os_str().is_empty() && browser_catalog::find(e.id.as_str()).is_some()
    })
}

/// Scan the catalog, write the registry file, and return it.
///
/// # Errors
///
/// Returns [`HostError::Registry`] when the cache directory or file cannot be written.
pub fn refresh(path: &Path) -> Result<BrowserRegistryFile, HostError> {
    refresh_with_discover(path, browser_catalog::discover)
}

/// Test/injection entry: refresh using a custom discover function (no process env).
pub(crate) fn refresh_with_discover<F>(
    path: &Path,
    discover: F,
) -> Result<BrowserRegistryFile, HostError>
where
    F: Fn(&CatalogBrowser) -> Option<PathBuf>,
{
    let file = scan_catalog(discover);
    write_registry(path, &file)?;
    Ok(file)
}

/// Ensure a registry exists (refresh if missing / soft-invalid), then return it.
///
/// # Errors
///
/// Propagates refresh / write / corrupt-load failures.
pub fn load_or_refresh(path: &Path) -> Result<BrowserRegistryFile, HostError> {
    if let Some(existing) = load(path)? {
        return Ok(existing);
    }
    refresh(path)
}

/// Look up a named browser; refresh once on miss / stale executable, then error.
///
/// # Errors
///
/// Returns [`HostError::ViewerNotFound`] when the browser is not installed, or
/// [`HostError::Registry`] on registry I/O failure.
pub fn resolve_named(id: BrowserId, path: &Path) -> Result<BrowserRegistryEntry, HostError> {
    resolve_named_with_discover(id, path, browser_catalog::discover)
}

pub(crate) fn resolve_named_with_discover<F>(
    id: BrowserId,
    path: &Path,
    discover: F,
) -> Result<BrowserRegistryEntry, HostError>
where
    F: Fn(&CatalogBrowser) -> Option<PathBuf>,
{
    let catalog = browser_catalog::for_browser_id(id);
    let mut file = match load(path)? {
        Some(existing) => existing,
        None => refresh_with_discover(path, &discover)?,
    };
    if let Some(entry) = find_usable_entry(&file, catalog.id) {
        return Ok(entry);
    }
    // Miss or stale executable → force re-scan once.
    file = refresh_with_discover(path, &discover)?;
    if let Some(entry) = find_usable_entry(&file, catalog.id) {
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

fn find_usable_entry(file: &BrowserRegistryFile, id: &str) -> Option<BrowserRegistryEntry> {
    file.entries
        .iter()
        .find(|e| e.id.as_str() == id && executable_usable(&e.executable))
        .cloned()
}

fn executable_usable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(meta) => meta.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn scan_catalog<F>(discover: F) -> BrowserRegistryFile
where
    F: Fn(&CatalogBrowser) -> Option<PathBuf>,
{
    let mut entries = Vec::new();
    for browser in browser_catalog::catalog() {
        if let Some(executable) = discover(browser) {
            if let Ok(entry) = entry_from_catalog(browser, executable) {
                entries.push(entry);
            }
        }
    }
    BrowserRegistryFile {
        version: REGISTRY_VERSION,
        updated_at: now_rfc3339(),
        platform: platform_tag(),
        entries,
    }
}

fn entry_from_catalog(
    browser: &CatalogBrowser,
    executable: PathBuf,
) -> Result<BrowserRegistryEntry, HostError> {
    Ok(BrowserRegistryEntry {
        id: CatalogId::new(browser.id)?,
        name: browser.name.to_string(),
        executable,
    })
}

fn write_registry(path: &Path, file: &BrowserRegistryFile) -> Result<(), HostError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| HostError::Registry {
            message: format!(
                "failed to create browser registry dir {}: {e}",
                parent.display()
            ),
        })?;
    }
    let json = serde_json::to_vec_pretty(file).map_err(|e| HostError::Registry {
        message: format!("failed to serialize browser registry: {e}"),
    })?;
    std::fs::write(path, json).map_err(|e| HostError::Registry {
        message: format!("failed to write browser registry {}: {e}", path.display()),
    })?;
    Ok(())
}

fn platform_tag() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    // Manual UTC RFC3339 without chrono/time crates.
    const SECS_PER_DAY: u64 = 86_400;
    let days = secs / SECS_PER_DAY;
    let tod = secs % SECS_PER_DAY;
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}.{nanos:09}Z")
}

/// Howard Hinnant civil_from_days (proleptic Gregorian) for days since 1970-01-01.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn refresh_writes_rfc3339_updated_at() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("browsers.json");
        let file = refresh_with_discover(&path, |_| None).expect("refresh");
        assert_eq!(file.version, REGISTRY_VERSION);
        assert!(path.is_file());
        // YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ
        let ts = &file.updated_at;
        assert!(
            ts.len() >= 20 && ts.ends_with('Z') && ts.chars().nth(10) == Some('T'),
            "updated_at not RFC3339: {ts}"
        );
        let date = &ts[..10];
        let parts: Vec<_> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[0].parse::<u32>().is_ok());
        assert!(parts[1].parse::<u32>().is_ok());
        assert!(parts[2].parse::<u32>().is_ok());
        let listed = list_entries(&path).expect("list");
        assert_eq!(listed, file.entries);
    }

    #[test]
    fn resolve_named_errors_when_missing_after_empty_refresh() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("browsers.json");
        // Injected discover never finds safari — no process-global env mutation.
        let discover = |_b: &CatalogBrowser| None;
        let empty = BrowserRegistryFile {
            version: REGISTRY_VERSION,
            updated_at: now_rfc3339(),
            platform: "test".into(),
            entries: vec![],
        };
        write_registry(&path, &empty).expect("write");
        let err =
            resolve_named_with_discover(BrowserId::Safari, &path, discover).expect_err("missing");
        assert!(matches!(err, HostError::ViewerNotFound { .. }));
    }

    #[test]
    fn load_distinguishes_missing_vs_corrupt() {
        let tmp = tempfile::tempdir().expect("tmp");
        let missing = tmp.path().join("nope.json");
        assert!(load(&missing).expect("missing ok").is_none());

        let corrupt = tmp.path().join("bad.json");
        std::fs::write(&corrupt, b"{not-json").expect("write");
        let err = load(&corrupt).expect_err("corrupt");
        assert!(matches!(err, HostError::Registry { .. }));
    }

    #[test]
    fn version_mismatch_is_cache_miss() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("browsers.json");
        let bad = BrowserRegistryFile {
            version: REGISTRY_VERSION + 99,
            updated_at: now_rfc3339(),
            platform: "test".into(),
            entries: vec![],
        };
        write_registry(&path, &bad).expect("write");
        assert!(load(&path).expect("load").is_none());
    }

    #[test]
    fn stale_executable_triggers_refresh_retry() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("browsers.json");
        let missing_bin = tmp.path().join("gone-chrome");
        let good_bin = tmp.path().join("chrome-bin");
        std::fs::write(&good_bin, b"#!/bin/sh\n").expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&good_bin).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&good_bin, perms).unwrap();
        }
        let stale = BrowserRegistryFile {
            version: REGISTRY_VERSION,
            updated_at: now_rfc3339(),
            platform: "test".into(),
            entries: vec![BrowserRegistryEntry {
                id: CatalogId::new("chrome").unwrap(),
                name: "Google Chrome".into(),
                executable: missing_bin,
            }],
        };
        write_registry(&path, &stale).expect("write");

        let good = Arc::new(good_bin);
        let discover = {
            let good = Arc::clone(&good);
            move |b: &CatalogBrowser| {
                if b.id == "chrome" {
                    Some((*good).clone())
                } else {
                    None
                }
            }
        };
        let entry = resolve_named_with_discover(BrowserId::Chrome, &path, discover).expect("retry");
        assert_eq!(entry.executable, *good);
    }

    #[test]
    fn catalog_id_rejects_unknown() {
        let err = CatalogId::new("not-a-browser").expect_err("unknown");
        assert!(matches!(err, HostError::Registry { .. }));
    }
}
