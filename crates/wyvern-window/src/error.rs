//! Window / event-loop failures surfaced to the CLI in sprint a.5.

/// Failure while creating a native window or running its event loop.
///
/// CLI mapping (a.5): `WindowCreate` → `window_create`, `EventLoop` → `event_loop`.
#[derive(Debug)]
pub enum RunError {
    /// Native window or webview construction failed.
    WindowCreate { message: String },
    /// Event loop creation or run failed.
    EventLoop { message: String },
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowCreate { message } => {
                write!(f, "window create failed: {message}")
            }
            Self::EventLoop { message } => write!(f, "event loop failed: {message}"),
        }
    }
}

impl std::error::Error for RunError {}
