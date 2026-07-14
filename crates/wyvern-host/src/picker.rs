//! Native file and folder pickers via `rfd` (REQ-0113 / ADR-0014).
//!
//! Headless CI strategy (in priority order):
//! 1. [`MockPickerConfig`] injected via [`crate::HostOptions::mock_picker`]
//!    (preferred for in-process tests — no process-global env mutation).
//! 2. Environment variable [`MOCK_PICKER_ENV`] (`WYVERN_MOCK_PICKER_PATH`) for
//!    CLI / Playwright child processes.
//!
//! A non-empty mock path is returned as a successful path selection (OS
//! path-list separator may join multiple paths); an empty value simulates
//! picker cancellation (`None`). Real picker UI tests without the mock must be
//! `#[ignore]` with reason `requires native picker UI`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rfd::FileDialog;

/// Test-only env var that bypasses the native picker UI (CLI / e2e children).
pub const MOCK_PICKER_ENV: &str = "WYVERN_MOCK_PICKER_PATH";

/// In-process mock for the native picker (integration tests / harnesses).
///
/// Prefer this over mutating [`MOCK_PICKER_ENV`] process-wide.
#[derive(Debug, Clone, Default)]
pub struct MockPickerConfig {
    /// Path list joined with the OS path separator; empty string cancels.
    pub path: String,
    /// Artificial delay before returning (exercises permit-hold / timeout).
    pub delay: Option<Duration>,
}

impl MockPickerConfig {
    /// Successful selection of a single path (no delay).
    pub fn path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            delay: None,
        }
    }

    /// Simulate user cancellation.
    pub fn cancel() -> Self {
        Self {
            path: String::new(),
            delay: None,
        }
    }

    /// Successful selection with an artificial delay.
    pub fn path_with_delay(path: impl Into<String>, delay: Duration) -> Self {
        Self {
            path: path.into(),
            delay: Some(delay),
        }
    }
}

/// Open a native file picker (or mock) and return selected path(s).
///
/// `filter` entries are extension patterns such as `*.json` or `txt`. When
/// `multiple` is true, the OS multi-select dialog is used.
///
/// Returns [`None`] when the user cancels (or the mock is empty).
pub fn pick_file(
    filter: &[String],
    multiple: bool,
    start_path: Option<&Path>,
    mock: Option<&MockPickerConfig>,
) -> Option<Vec<PathBuf>> {
    if let Some(mocked) = resolve_mock(mock) {
        return mocked;
    }

    let mut dialog = FileDialog::new();
    if let Some(dir) = start_path {
        dialog = dialog.set_directory(dir);
    }

    let extensions = filter_extensions(filter);
    if !extensions.is_empty() {
        dialog = dialog.add_filter("Files", &extensions);
    }

    if multiple {
        dialog.pick_files()
    } else {
        dialog.pick_file().map(|p| vec![p])
    }
}

/// Open a native folder picker (or mock) and return the selected directory.
///
/// Returns [`None`] when the user cancels (or the mock is empty).
pub fn pick_folder(start_path: Option<&Path>, mock: Option<&MockPickerConfig>) -> Option<PathBuf> {
    if let Some(mocked) = resolve_mock(mock) {
        return mocked.and_then(|mut paths| paths.pop());
    }

    let mut dialog = FileDialog::new();
    if let Some(dir) = start_path {
        dialog = dialog.set_directory(dir);
    }
    dialog.pick_folder()
}

/// Convert wire filter patterns (`*.json`, `.txt`, `rs`) into bare extensions.
fn filter_extensions(filter: &[String]) -> Vec<String> {
    filter
        .iter()
        .map(|pat| {
            let s = pat.trim();
            let s = s.strip_prefix("*.").unwrap_or(s);
            let s = s.strip_prefix('*').unwrap_or(s);
            let s = s.strip_prefix('.').unwrap_or(s);
            s.to_string()
        })
        .filter(|ext| !ext.is_empty())
        .collect()
}

/// Resolve mock paths from an injected config, else [`MOCK_PICKER_ENV`].
///
/// Returns `Some(None)` = cancel, `Some(Some(paths))` = selection, `None` = use `rfd`.
fn resolve_mock(mock: Option<&MockPickerConfig>) -> Option<Option<Vec<PathBuf>>> {
    if let Some(cfg) = mock {
        if let Some(delay) = cfg.delay {
            std::thread::sleep(delay);
        }
        return Some(parse_mock_value(&cfg.path));
    }
    try_mock_env_paths()
}

fn parse_mock_value(s: &str) -> Option<Vec<PathBuf>> {
    if s.is_empty() {
        return None;
    }
    let paths: Vec<PathBuf> = std::env::split_paths(s).collect();
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

/// When `WYVERN_MOCK_PICKER_PATH` is set: `Some(None)` = cancel, `Some(Some(paths))`
/// = selection. When unset: `None` (use real `rfd`).
fn try_mock_env_paths() -> Option<Option<Vec<PathBuf>>> {
    match std::env::var(MOCK_PICKER_ENV) {
        Err(_) => None,
        Ok(s) => Some(parse_mock_value(&s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::ffi::OsStr;

    /// RAII guard: sets [`MOCK_PICKER_ENV`] and removes it on drop.
    struct MockPickerEnvGuard;

    impl MockPickerEnvGuard {
        fn set(value: impl AsRef<OsStr>) -> Self {
            // SAFETY: callers are `#[serial]` so no concurrent env mutation.
            unsafe { std::env::set_var(MOCK_PICKER_ENV, value) };
            Self
        }
    }

    impl Drop for MockPickerEnvGuard {
        fn drop(&mut self) {
            unsafe { std::env::remove_var(MOCK_PICKER_ENV) };
        }
    }

    #[test]
    fn filter_extensions_strips_glob_prefix() {
        let exts =
            filter_extensions(&["*.json".into(), "*.txt".into(), "rs".into(), ".toml".into()]);
        assert_eq!(exts, ["json", "txt", "rs", "toml"]);
    }

    #[test]
    fn injected_mock_returns_path_without_env() {
        let fixture = std::env::temp_dir().join("wyvern-picker-injected.txt");
        let cfg = MockPickerConfig::path(fixture.to_string_lossy());
        let picked = pick_file(&[], false, None, Some(&cfg));
        assert_eq!(picked, Some(vec![fixture]));
    }

    #[test]
    fn injected_mock_empty_simulates_cancel() {
        let cfg = MockPickerConfig::cancel();
        let picked = pick_file(&[], false, None, Some(&cfg));
        assert_eq!(picked, None);
    }

    #[test]
    #[serial]
    fn mock_env_returns_path_without_rfd() {
        let fixture = std::env::temp_dir().join("wyvern-picker-fixture.txt");
        let _guard = MockPickerEnvGuard::set(&fixture);
        let picked = pick_file(&[], false, None, None);
        assert_eq!(picked, Some(vec![fixture]));
    }

    #[test]
    #[serial]
    fn mock_env_empty_simulates_cancel() {
        let _guard = MockPickerEnvGuard::set("");
        let picked = pick_file(&[], false, None, None);
        assert_eq!(picked, None);
    }

    #[test]
    #[serial]
    fn mock_folder_returns_path() {
        let picked_dir = std::env::temp_dir().join("wyvern-picked-dir");
        let _guard = MockPickerEnvGuard::set(&picked_dir);
        let picked = pick_folder(None, None);
        assert_eq!(picked, Some(picked_dir));
    }

    #[test]
    #[ignore = "requires native picker UI"]
    fn real_file_picker_smoke() {
        let _ = pick_file(&["*.txt".into()], false, None, None);
    }
}
