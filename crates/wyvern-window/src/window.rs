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

/// Platform chrome window attributes (ADR-0010 / ADR-0010a):
/// - macOS: transparent title bar + full-size content
/// - Windows/Linux: `decorations: false` + HTML window controls
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

#[cfg(test)]
mod tests {
    //! ADR-0010a: Win/Linux use `decorations: false` (HTML chrome).
    //! M11: dialog min/max and chrome default open size.

    use winit::dpi::{LogicalSize, Size};

    use crate::{
        CHROME_DEFAULT_HEIGHT, CHROME_DEFAULT_WIDTH, DIALOG_MAX_HEIGHT, DIALOG_MAX_WIDTH,
        DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH,
    };

    fn logical_inner(size: Option<Size>) -> (f64, f64) {
        match size.expect("inner size must be set") {
            Size::Logical(LogicalSize { width, height }) => (width, height),
            Size::Physical(_) => panic!("expected logical size"),
        }
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn non_macos_chrome_and_modal_attrs_disable_decorations() {
        let chrome = super::chrome_window_attributes("decorations-test");
        assert!(
            !chrome.decorations,
            "Win/Linux chrome must use decorations(false) per ADR-0010a"
        );

        let modal = super::modal_window_attributes("decorations-modal", 400.0, 300.0);
        assert!(
            !modal.decorations,
            "Win/Linux modal must use decorations(false) per ADR-0010a"
        );
    }

    #[test]
    fn chrome_default_open_size_is_480x360() {
        let chrome = super::chrome_window_attributes("chrome-size");
        let (w, h) = logical_inner(chrome.inner_size);
        assert!(
            (w - CHROME_DEFAULT_WIDTH).abs() < f64::EPSILON,
            "chrome default width must be {CHROME_DEFAULT_WIDTH}, got {w}"
        );
        assert!(
            (h - CHROME_DEFAULT_HEIGHT).abs() < f64::EPSILON,
            "chrome default height must be {CHROME_DEFAULT_HEIGHT}, got {h}"
        );
    }

    #[test]
    fn modal_attrs_clamp_below_min_and_above_max() {
        let too_small = super::modal_window_attributes("modal-min", 100.0, 50.0);
        let (w_min, h_min) = logical_inner(too_small.inner_size);
        assert!((w_min - DIALOG_MIN_WIDTH).abs() < f64::EPSILON);
        assert!((h_min - DIALOG_MIN_HEIGHT).abs() < f64::EPSILON);

        let too_large = super::modal_window_attributes("modal-max", 2000.0, 1500.0);
        let (w_max, h_max) = logical_inner(too_large.inner_size);
        assert!((w_max - DIALOG_MAX_WIDTH).abs() < f64::EPSILON);
        assert!((h_max - DIALOG_MAX_HEIGHT).abs() < f64::EPSILON);

        let (min_w, min_h) = logical_inner(too_small.min_inner_size);
        assert!((min_w - DIALOG_MIN_WIDTH).abs() < f64::EPSILON);
        assert!((min_h - DIALOG_MIN_HEIGHT).abs() < f64::EPSILON);

        let (max_w, max_h) = logical_inner(too_large.max_inner_size);
        assert!((max_w - DIALOG_MAX_WIDTH).abs() < f64::EPSILON);
        assert!((max_h - DIALOG_MAX_HEIGHT).abs() < f64::EPSILON);
    }
}
