//! CLI / one-shot host options.

use std::net::SocketAddr;
use std::path::PathBuf;

/// How the dialog URL is opened after bind.
///
/// c.10 implements [`ViewerMode::None`] only. Other variants are parsed by the
/// CLI and rejected at run time until c.15.
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
            Self::Named(BrowserId::Chrome) => "chrome",
            Self::Named(BrowserId::Safari) => "safari",
            Self::Named(BrowserId::Edge) => "edge",
            Self::Named(BrowserId::Firefox) => "firefox",
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

/// CLI / one-shot invocation options (from `--bind`, `--ui-root`, `--viewer`).
#[derive(Debug, Clone)]
pub struct HostOptions {
    /// TCP bind address (default `127.0.0.1:0`).
    pub bind: SocketAddr,
    /// Static UI root directory (default `ui/` in the workspace / install tree).
    pub ui_root: PathBuf,
    /// Viewer launch mode (c.10: only [`ViewerMode::None`] is supported).
    pub viewer: ViewerMode,
    /// When true, set `WYVERN_DIALOG_URL` after bind (typical for `none`).
    pub dialog_url_env: bool,
    /// Optional path to write the dialog URL (preferred over process-wide env for tests).
    pub dialog_url_file: Option<PathBuf>,
}

impl Default for HostOptions {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([127, 0, 0, 1], 0)),
            ui_root: PathBuf::from("ui"),
            viewer: ViewerMode::None,
            dialog_url_env: true,
            dialog_url_file: None,
        }
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
