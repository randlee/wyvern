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
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rfd::FileDialog;

/// Test-only env var that bypasses the native picker UI (CLI / e2e children).
pub const MOCK_PICKER_ENV: &str = "WYVERN_MOCK_PICKER_PATH";

/// Ordered enter/exit events for mock picker bodies (serialization tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockPickerSlotEvent {
    /// Mock picker body started (holds the session picker permit).
    Enter,
    /// Mock picker body finished.
    Exit,
}

/// Shared log of [`MockPickerSlotEvent`] for deterministic slot-order assertions.
#[derive(Debug, Clone, Default)]
pub struct MockPickerSlotLog {
    events: Arc<Mutex<Vec<MockPickerSlotEvent>>>,
}

impl MockPickerSlotLog {
    /// Create an empty event log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of recorded enter/exit events in order.
    pub fn events(&self) -> Vec<MockPickerSlotEvent> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn record(&self, event: MockPickerSlotEvent) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(event);
    }
}

/// In-process mock for the native picker (integration tests / harnesses).
///
/// Prefer this over mutating [`MOCK_PICKER_ENV`] process-wide.
#[derive(Debug, Clone, Default)]
pub struct MockPickerConfig {
    /// Path list joined with the OS path separator; empty string cancels.
    pub path: String,
    /// Artificial delay before returning (exercises permit-hold / timeout).
    pub delay: Option<Duration>,
    /// Optional enter/exit log for observing picker-slot serialization.
    pub slot_log: Option<MockPickerSlotLog>,
}

impl MockPickerConfig {
    /// Successful selection of a single path (no delay).
    pub fn path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            delay: None,
            slot_log: None,
        }
    }

    /// Simulate user cancellation.
    pub fn cancel() -> Self {
        Self {
            path: String::new(),
            delay: None,
            slot_log: None,
        }
    }

    /// Successful selection with an artificial delay.
    pub fn path_with_delay(path: impl Into<String>, delay: Duration) -> Self {
        Self {
            path: path.into(),
            delay: Some(delay),
            slot_log: None,
        }
    }

    /// Attach a slot event log (for serialization tests).
    pub fn with_slot_log(mut self, log: MockPickerSlotLog) -> Self {
        self.slot_log = Some(log);
        self
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

    #[cfg(target_os = "macos")]
    if !crate::picker_dispatch::is_main_thread() {
        return crate::picker_dispatch::dispatch_file(filter, multiple, start_path);
    }

    pick_file_rfd(filter, multiple, start_path)
}

/// Open a native folder picker (or mock) and return the selected directory.
///
/// Returns [`None`] when the user cancels (or the mock is empty).
pub fn pick_folder(start_path: Option<&Path>, mock: Option<&MockPickerConfig>) -> Option<PathBuf> {
    if let Some(mocked) = resolve_mock(mock) {
        return mocked.and_then(|mut paths| paths.pop());
    }

    #[cfg(target_os = "macos")]
    if !crate::picker_dispatch::is_main_thread() {
        return crate::picker_dispatch::dispatch_folder(start_path);
    }

    pick_folder_rfd(start_path)
}

/// Native `rfd` file picker — must run on the process main thread on macOS.
pub(crate) fn pick_file_rfd(
    filter: &[String],
    multiple: bool,
    start_path: Option<&Path>,
) -> Option<Vec<PathBuf>> {
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

/// Native `rfd` folder picker — must run on the process main thread on macOS.
pub(crate) fn pick_folder_rfd(start_path: Option<&Path>) -> Option<PathBuf> {
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
        if let Some(log) = &cfg.slot_log {
            log.record(MockPickerSlotEvent::Enter);
        }
        if let Some(delay) = cfg.delay {
            std::thread::sleep(delay);
        }
        let result = Some(parse_mock_value(&cfg.path));
        if let Some(log) = &cfg.slot_log {
            log.record(MockPickerSlotEvent::Exit);
        }
        return result;
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
    #[ignore = "requires native picker UI"]
    fn real_file_picker_smoke() {
        let _ = pick_file(&["*.txt".into()], false, None, None);
    }
}
