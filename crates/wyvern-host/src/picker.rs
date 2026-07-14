//! Native file and folder pickers via `rfd` (REQ-0113 / ADR-0014).
//!
//! Headless CI strategy: when the test-only environment variable
//! [`MOCK_PICKER_ENV`] (`WYVERN_MOCK_PICKER_PATH`) is set, this module skips the
//! native `rfd` UI. A non-empty value is returned as a successful path selection
//! (OS path-list separator may join multiple paths); an empty value simulates
//! picker cancellation (`None`). Real picker UI tests without the mock must be
//! `#[ignore]` with reason `requires native picker UI`.

use std::path::{Path, PathBuf};

use rfd::FileDialog;

/// Test-only env var that bypasses the native picker UI.
pub const MOCK_PICKER_ENV: &str = "WYVERN_MOCK_PICKER_PATH";

/// Open a native file picker (or mock) and return selected path(s).
///
/// `filter` entries are extension patterns such as `*.json` or `txt`. When
/// `multiple` is true, the OS multi-select dialog is used.
///
/// Returns [`None`] when the user cancels (or the mock env is empty).
pub fn pick_file(
    filter: &[String],
    multiple: bool,
    start_path: Option<&Path>,
) -> Option<Vec<PathBuf>> {
    if let Some(mocked) = try_mock_paths() {
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
/// Returns [`None`] when the user cancels (or the mock env is empty).
pub fn pick_folder(start_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(mocked) = try_mock_paths() {
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

/// When `WYVERN_MOCK_PICKER_PATH` is set: `Some(None)` = cancel, `Some(Some(paths))`
/// = selection. When unset: `None` (use real `rfd`).
fn try_mock_paths() -> Option<Option<Vec<PathBuf>>> {
    match std::env::var(MOCK_PICKER_ENV) {
        Err(_) => None,
        Ok(s) if s.is_empty() => Some(None),
        Ok(s) => {
            let paths: Vec<PathBuf> = std::env::split_paths(&s).collect();
            if paths.is_empty() {
                Some(None)
            } else {
                Some(Some(paths))
            }
        }
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
    #[serial]
    fn mock_env_returns_path_without_rfd() {
        let fixture = std::env::temp_dir().join("wyvern-picker-fixture.txt");
        let _guard = MockPickerEnvGuard::set(&fixture);
        let picked = pick_file(&[], false, None);
        assert_eq!(picked, Some(vec![fixture]));
    }

    #[test]
    #[serial]
    fn mock_env_empty_simulates_cancel() {
        let _guard = MockPickerEnvGuard::set("");
        let picked = pick_file(&[], false, None);
        assert_eq!(picked, None);
    }

    #[test]
    #[serial]
    fn mock_folder_returns_path() {
        let picked_dir = std::env::temp_dir().join("wyvern-picked-dir");
        let _guard = MockPickerEnvGuard::set(&picked_dir);
        let picked = pick_folder(None);
        assert_eq!(picked, Some(picked_dir));
    }

    #[test]
    #[ignore = "requires native picker UI"]
    fn real_file_picker_smoke() {
        let _ = pick_file(&["*.txt".into()], false, None);
    }
}
