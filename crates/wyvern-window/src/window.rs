//! Shared platform init and window attributes (chrome + modal dialogs).

use winit::dpi::LogicalSize;
use winit::window::{Window, WindowAttributes, WindowButtons};

use crate::error::RunError;
use crate::{
    CHROME_DEFAULT_HEIGHT, CHROME_DEFAULT_WIDTH, CHROME_MAX_HEIGHT, CHROME_MAX_WIDTH,
    DIALOG_MAX_HEIGHT, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH,
};

/// Initialize platform prerequisites (GTK on Linux; soft-GL under CI).
pub(crate) fn init_platform() -> Result<(), RunError> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    {
        // Headless CI (xvfb) has no usable GPU; WebKitGTK compositing then trips
        // GLXBadWindow inside winit's X11 error handler. Soften the GPU path when
        // CI is set and the caller has not already chosen values.
        if std::env::var_os("CI").is_some() {
            set_env_if_unset("GDK_BACKEND", "x11");
            set_env_if_unset("LIBGL_ALWAYS_SOFTWARE", "1");
            set_env_if_unset("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            set_env_if_unset("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }

        gtk::init().map_err(|err| {
            let display = std::env::var("DISPLAY").unwrap_or_else(|_| "<unset>".into());
            RunError::EventLoop {
                message: format!("gtk init failed (DISPLAY={display}): {err}"),
            }
        })?;
    }
    Ok(())
}

/// Sets `key=value` only when unset. Used before GTK/wry init on the main thread.
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn set_env_if_unset(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        // SAFETY: called once from the main thread before gtk::init / wry start;
        // no concurrent env readers yet.
        unsafe { std::env::set_var(key, value) };
    }
}

/// Phase A platform interim policy (see phase-A README):
/// - macOS: transparent title bar + full-size content (ADR-0010)
/// - Windows/Linux: native OS decorations (custom chrome deferred to Phase C)
pub(crate) fn chrome_window_attributes(title: &str) -> WindowAttributes {
    let attrs = Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(
            CHROME_DEFAULT_WIDTH,
            CHROME_DEFAULT_HEIGHT,
        ))
        .with_max_inner_size(LogicalSize::new(CHROME_MAX_WIDTH, CHROME_MAX_HEIGHT));

    apply_platform_chrome(attrs)
}

/// Modal dialog attributes (REQ-0083 + REQ-0041).
///
/// Minimize/maximize disabled; size clamped to Phase B min/max.
pub(crate) fn modal_window_attributes(title: &str, width: f64, height: f64) -> WindowAttributes {
    let width = width.clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);
    let height = height.clamp(DIALOG_MIN_HEIGHT, DIALOG_MAX_HEIGHT);

    let attrs = Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_min_inner_size(LogicalSize::new(DIALOG_MIN_WIDTH, DIALOG_MIN_HEIGHT))
        .with_max_inner_size(LogicalSize::new(DIALOG_MAX_WIDTH, DIALOG_MAX_HEIGHT))
        .with_resizable(false)
        // REQ-0083: modal types disable minimize and maximize/fullscreen.
        .with_enabled_buttons(WindowButtons::CLOSE);

    apply_platform_chrome(attrs)
}

fn apply_platform_chrome(attrs: WindowAttributes) -> WindowAttributes {
    #[cfg(target_os = "macos")]
    let attrs = {
        use winit::platform::macos::WindowAttributesExtMacOS;
        attrs
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true)
    };

    #[cfg(not(target_os = "macos"))]
    let attrs = attrs.with_decorations(true);

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
