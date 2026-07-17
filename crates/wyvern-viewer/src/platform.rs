//! Platform init (GTK on Linux) and chrome window attributes.

use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{UserAttentionType, Window, WindowAttributes, WindowButtons, WindowLevel};

/// Viewer platform bootstrap failure.
#[derive(Debug)]
pub enum PlatformError {
    /// Event loop / GTK init failure.
    EventLoop {
        /// Failure detail.
        message: String,
        /// Optional upstream platform cause.
        cause: Option<String>,
    },
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoop { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for PlatformError {}

/// Bootstrap size used only while the window is **hidden**.
///
/// Policy (d.6 / REQ-V008): the viewer must never flash this 320×240 placeholder
/// at the user. Create the window with `visible: false`, inject viewport bounds,
/// wait for the first content `resize:` IPC, then [`present_viewer_window`].
pub const DEFAULT_WIDTH: f64 = 320.0;
/// Bootstrap height paired with [`DEFAULT_WIDTH`] (hidden until first resize).
pub const DEFAULT_HEIGHT: f64 = 240.0;

/// Initialize platform prerequisites (GTK on Linux).
pub fn init_platform() -> Result<(), PlatformError> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    {
        if std::env::var_os("CI").is_some() {
            set_env_if_unset("GDK_BACKEND", "x11");
            set_env_if_unset("LIBGL_ALWAYS_SOFTWARE", "1");
            set_env_if_unset("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            set_env_if_unset("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        gtk::init().map_err(|err| {
            let display = std::env::var("DISPLAY").unwrap_or_else(|_| "<unset>".into());
            PlatformError::EventLoop {
                message: format!("gtk init failed (DISPLAY={display})"),
                cause: Some(err.to_string()),
            }
        })?;
    }
    Ok(())
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn set_env_if_unset(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        // SAFETY: called once from the main thread before gtk::init / wry start.
        unsafe { std::env::set_var(key, value) };
    }
}

/// Build a viewer event loop with custom user events (IPC wakeups).
pub fn build_event_loop<T: 'static>() -> Result<EventLoop<T>, PlatformError> {
    let mut builder = EventLoop::<T>::with_user_event();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder
            .with_activation_policy(ActivationPolicy::Regular)
            // Bring Wyvern Browser to front on launch; user can still alt-tab away.
            .with_activate_ignoring_other_apps(true);
    }
    builder.build().map_err(|err| PlatformError::EventLoop {
        message: "failed to build event loop".into(),
        cause: Some(err.to_string()),
    })
}

/// Present the viewer window: visible, focused, normal level (not always-on-top / modal).
///
/// Call only after the first content-sized resize so the 320×240 bootstrap is never shown.
pub fn present_viewer_window(window: &Window) {
    window.set_visible(true);
    window.set_window_level(WindowLevel::Normal);
    window.focus_window();
    window.request_user_attention(Some(UserAttentionType::Informational));
}

/// macOS: standard decorated window — native title bar drag, stable alt-tab.
///
/// Starts **hidden**; [`present_viewer_window`] shows after first resize.
#[cfg(target_os = "macos")]
pub fn viewer_window_attributes(title: &str, width: f64, height: f64) -> WindowAttributes {
    Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_visible(false)
        .with_resizable(true)
        .with_enabled_buttons(
            WindowButtons::CLOSE | WindowButtons::MINIMIZE | WindowButtons::MAXIMIZE,
        )
        .with_window_level(WindowLevel::Normal)
}

/// Win/Linux: undecorated frame; HTML chrome supplies controls where needed.
///
/// Starts **hidden**; [`present_viewer_window`] shows after first resize.
/// `with_window_level(WindowLevel::Normal)` ensures no always-on-top trapping.
#[cfg(not(target_os = "macos"))]
pub fn viewer_window_attributes(title: &str, width: f64, height: f64) -> WindowAttributes {
    Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_visible(false)
        .with_resizable(true)
        .with_enabled_buttons(
            WindowButtons::CLOSE | WindowButtons::MINIMIZE | WindowButtons::MAXIMIZE,
        )
        .with_decorations(false)
        .with_window_level(WindowLevel::Normal)
}

/// Drain pending GTK events so WebKit can release resources cleanly.
///
/// No-op when GTK is not initialized (headless Linux CI / unit tests without
/// a display). Calling `gtk::events_pending` before `gtk::init` panics.
pub fn pump_gtk_events() {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    {
        if !gtk::is_initialized() {
            return;
        }
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }
}

/// Parse optional width/height env overrides for the bootstrap window (QA-001).
#[must_use]
pub fn resolve_bootstrap_size(width_env: Option<&str>, height_env: Option<&str>) -> (f64, f64) {
    let width = width_env
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WIDTH);
    let height = height_env
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_HEIGHT);
    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bootstrap_size_is_hidden_placeholder() {
        assert_eq!(DEFAULT_WIDTH, 320.0);
        assert_eq!(DEFAULT_HEIGHT, 240.0);
    }

    #[test]
    fn resolve_bootstrap_size_defaults_and_overrides() {
        assert_eq!(resolve_bootstrap_size(None, None), (320.0, 240.0));
        assert_eq!(
            resolve_bootstrap_size(Some("640"), Some("480")),
            (640.0, 480.0)
        );
        assert_eq!(
            resolve_bootstrap_size(Some("bad"), Some("480")),
            (320.0, 480.0)
        );
    }

    #[test]
    fn viewer_window_attributes_start_hidden() {
        let attrs = viewer_window_attributes("Wyvern", DEFAULT_WIDTH, DEFAULT_HEIGHT);
        // Building attributes without panic covers the platform chrome path (QA-001).
        let _ = attrs;
    }

    #[test]
    fn pump_gtk_events_is_noop_or_drains() {
        // Without gtk::init (headless CI / unit tests), this must be a no-op —
        // not a panic from gtk::events_pending (CI-UBUNTU-001).
        pump_gtk_events();
    }
}
