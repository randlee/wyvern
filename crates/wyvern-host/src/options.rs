//! CLI / one-shot host options.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use crate::picker::MockPickerConfig;

/// Default one-shot session idle timeout before dismissed semantics (REQ-0097).
pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(600);

/// Minimum allowed [`HostOptions::session_timeout`] (rejects zero / sub-second).
pub const MIN_SESSION_TIMEOUT: Duration = Duration::from_secs(1);

/// How the dialog URL is opened after bind.
///
/// c.15 implements all modes. Product CLI default is [`ViewerMode::Embedded`];
/// CI / headless uses [`ViewerMode::None`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerMode {
    /// Spawn `wyvern-viewer` (c.15 — product default).
    Embedded,
    /// No launch; set `WYVERN_DIALOG_URL` for headless e2e.
    None,
    /// OS default browser via `webbrowser` (c.15).
    System,
    /// Named browser from the Wyvern registry (c.15).
    Named(BrowserId),
}

impl ViewerMode {
    /// Wire name for errors and logging.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::None => "none",
            Self::System => "system",
            Self::Named(id) => id.as_str(),
        }
    }

    /// Parse a `--viewer` / `WYVERN_VIEWER` value.
    ///
    /// Accepts `browser` as a deprecated alias for [`ViewerMode::System`].
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "embedded" => Some(Self::Embedded),
            "none" => Some(Self::None),
            "system" | "browser" => Some(Self::System),
            "chrome" => Some(Self::Named(BrowserId::Chrome)),
            "safari" => Some(Self::Named(BrowserId::Safari)),
            "edge" => Some(Self::Named(BrowserId::Edge)),
            "firefox" => Some(Self::Named(BrowserId::Firefox)),
            _ => None,
        }
    }
}

/// Named browser catalog id (c.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserId {
    /// Google Chrome.
    Chrome,
    /// Apple Safari.
    Safari,
    /// Microsoft Edge.
    Edge,
    /// Mozilla Firefox.
    Firefox,
}

impl BrowserId {
    /// Wire / catalog id for this browser.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Safari => "safari",
            Self::Edge => "edge",
            Self::Firefox => "firefox",
        }
    }
}

impl std::fmt::Display for BrowserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// CLI / one-shot invocation options (from `--bind`, `--ui-root`, `--viewer`).
#[derive(Debug, Clone)]
pub struct HostOptions {
    /// TCP bind address (default `127.0.0.1:0`).
    pub bind: SocketAddr,
    /// Static UI root directory (default `ui/` in the workspace / install tree).
    pub ui_root: PathBuf,
    /// Viewer launch mode (c.15: `embedded` / `none` / `system` / named).
    pub viewer: ViewerMode,
    /// When true, publish dialog URL via stderr / optional file after bind (typical for `none`).
    pub dialog_url_env: bool,
    /// Optional path to write the dialog URL (preferred over process-wide env for tests).
    pub dialog_url_file: Option<PathBuf>,
    /// Allow non-loopback binds (`0.0.0.0`, LAN IPs). Default false (ADR-0016).
    pub allow_non_loopback: bool,
    /// Idle timeout waiting for `POST /api/result`; expiry → dismissed (REQ-0097).
    pub session_timeout: Duration,
    /// In-process picker mock for tests/harnesses (preferred over process-global env).
    pub mock_picker: Option<MockPickerConfig>,
}

impl Default for HostOptions {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([127, 0, 0, 1], 0)),
            ui_root: PathBuf::from("ui"),
            viewer: ViewerMode::None,
            dialog_url_env: true,
            dialog_url_file: None,
            allow_non_loopback: false,
            session_timeout: DEFAULT_SESSION_TIMEOUT,
            mock_picker: None,
        }
    }
}

impl HostOptions {
    /// Reject zero / sub-minimum [`Self::session_timeout`] at startup (RSH-005).
    ///
    /// # Errors
    ///
    /// Returns [`crate::HostError::Internal`] when `session_timeout` is below
    /// [`MIN_SESSION_TIMEOUT`].
    pub fn validate(&self) -> Result<(), crate::HostError> {
        if self.session_timeout < MIN_SESSION_TIMEOUT {
            return Err(crate::HostError::Internal {
                message: format!(
                    "session_timeout {:?} is below minimum {:?} (must be >= 1s; zero disables idle dismiss incorrectly)",
                    self.session_timeout, MIN_SESSION_TIMEOUT
                ),
            });
        }
        Ok(())
    }
}

/// Optional window hints for an embedded viewer (c.15).
#[derive(Debug, Clone, Default)]
pub struct ViewerLaunchOptions {
    /// Preferred window width in CSS pixels.
    pub width: Option<u32>,
    /// Preferred window height in CSS pixels.
    pub height: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn validate_rejects_zero_session_timeout() {
        let mut opts = HostOptions::default();
        opts.session_timeout = Duration::ZERO;
        let err = opts.validate().expect_err("zero");
        assert!(err.to_string().contains("session_timeout"));
    }

    #[test]
    fn validate_rejects_sub_minimum_session_timeout() {
        let mut opts = HostOptions::default();
        opts.session_timeout = Duration::from_millis(500);
        assert!(opts.validate().is_err());
    }

    #[test]
    fn validate_accepts_default_session_timeout() {
        HostOptions::default().validate().expect("default ok");
    }
}
