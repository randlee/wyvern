//! Platform init (GTK on Linux) and chrome window attributes.

use winit::dpi::LogicalSize;
use winit::window::{Window, WindowAttributes, WindowButtons};

use crate::run::ViewerError;

/// Default embedded dialog size when width/height are omitted.
pub(crate) const DEFAULT_WIDTH: f64 = 480.0;
/// Default embedded dialog height.
pub(crate) const DEFAULT_HEIGHT: f64 = 360.0;

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

/// Transparent / undecorated chrome attrs matching prior wyvern-window dialogs.
pub(crate) fn viewer_window_attributes(title: &str, width: f64, height: f64) -> WindowAttributes {
    let attrs = Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_resizable(true)
        .with_enabled_buttons(WindowButtons::CLOSE);

    #[cfg(target_os = "macos")]
    let attrs = {
        use winit::platform::macos::WindowAttributesExtMacOS;
        attrs
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true)
    };

    #[cfg(not(target_os = "macos"))]
    let attrs = attrs.with_decorations(false);

    attrs
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
