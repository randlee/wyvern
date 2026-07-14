//! Platform init (GTK on Linux) and chrome window attributes.

use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{UserAttentionType, Window, WindowAttributes, WindowButtons, WindowLevel};

use crate::run::ViewerError;

/// Bootstrap size before page `resize:` IPC; 4:3 placeholder until auto-size IPC.
pub(crate) const DEFAULT_WIDTH: f64 = 320.0;
/// Bootstrap height before auto-size (4:3 with [`DEFAULT_WIDTH`]).
pub(crate) const DEFAULT_HEIGHT: f64 = 240.0;

/// Initialize platform prerequisites (GTK on Linux).
pub(crate) fn init_platform() -> Result<(), ViewerError> {
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
            ViewerError::EventLoop {
                message: format!("gtk init failed (DISPLAY={display}): {err}"),
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

/// Build the viewer event loop (regular app on macOS — not accessory/modal panel).
pub(crate) fn build_event_loop() -> Result<EventLoop<()>, ViewerError> {
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder
            .with_activation_policy(ActivationPolicy::Regular)
            // Bring Wyvern Browser to front on launch; user can still alt-tab away.
            .with_activate_ignoring_other_apps(true);
    }
    builder.build().map_err(|err| ViewerError::EventLoop {
        message: err.to_string(),
    })
}

/// Present the viewer window: visible, focused, normal level (not always-on-top / modal).
pub(crate) fn present_viewer_window(window: &Window) {
    window.set_visible(true);
    window.set_window_level(WindowLevel::Normal);
    window.focus_window();
    window.request_user_attention(Some(UserAttentionType::Informational));
}

/// macOS: standard decorated window — native title bar drag, stable alt-tab.
#[cfg(target_os = "macos")]
pub(crate) fn viewer_window_attributes(title: &str, width: f64, height: f64) -> WindowAttributes {
    Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_resizable(true)
        .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
        .with_window_level(WindowLevel::Normal)
}

/// Win/Linux: undecorated frame; HTML chrome supplies controls where needed.
///
/// `with_window_level(WindowLevel::Normal)` ensures no always-on-top trapping.
#[cfg(not(target_os = "macos"))]
pub(crate) fn viewer_window_attributes(title: &str, width: f64, height: f64) -> WindowAttributes {
    Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_resizable(true)
        .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
        .with_decorations(false)
        .with_window_level(WindowLevel::Normal)
}

/// Drain pending GTK events so WebKit can release resources cleanly.
pub(crate) fn pump_gtk_events() {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    {
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }
}
