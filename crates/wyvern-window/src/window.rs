//! Blank native window + wry webview (Phase A interim; a.5 adds `run`).

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use wry::WebViewBuilder;

use crate::error::RunError;

/// Default blank-window size — matches a.5 chrome open size (480×360).
const BLANK_WIDTH: f64 = 480.0;
const BLANK_HEIGHT: f64 = 360.0;

/// Opens a blank `winit`+`wry` window and runs until dismissed.
///
/// When `auto_dismiss` is true (crate tests), the window closes via the same
/// dismiss path as OS chrome close once creation succeeds, returning [`Ok(())`].
///
/// Not part of the long-term public API — superseded by `run` in sprint a.5.
#[doc(hidden)]
pub fn open_blank_window(auto_dismiss: bool) -> Result<(), RunError> {
    init_platform()?;

    let event_loop = EventLoop::new().map_err(|err| RunError::EventLoop {
        message: err.to_string(),
    })?;

    let mut app = BlankApp {
        window: None,
        webview: None,
        auto_dismiss,
        pending_dismiss: false,
        outcome: None,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|err| RunError::EventLoop {
            message: err.to_string(),
        })?;

    app.outcome.unwrap_or(Ok(()))
}

fn init_platform() -> Result<(), RunError> {
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
fn blank_window_attributes() -> WindowAttributes {
    let attrs = Window::default_attributes()
        .with_title("wyvern")
        .with_inner_size(LogicalSize::new(BLANK_WIDTH, BLANK_HEIGHT));

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

struct BlankApp {
    window: Option<Window>,
    webview: Option<wry::WebView>,
    auto_dismiss: bool,
    pending_dismiss: bool,
    outcome: Option<Result<(), RunError>>,
}

impl BlankApp {
    fn dismiss(&mut self, event_loop: &ActiveEventLoop) {
        // Drop webview before the winit window so WebKit can release GL/X resources
        // without GLXBadWindow under xvfb.
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        self.outcome = Some(Ok(()));
        event_loop.exit();
    }

    fn fail_create(&mut self, event_loop: &ActiveEventLoop, message: String) {
        self.webview.take();
        self.window.take();
        self.outcome = Some(Err(RunError::WindowCreate { message }));
        event_loop.exit();
    }
}

impl ApplicationHandler for BlankApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match event_loop.create_window(blank_window_attributes()) {
            Ok(window) => window,
            Err(err) => {
                self.fail_create(event_loop, err.to_string());
                return;
            }
        };

        let webview = match WebViewBuilder::new(&window)
            .with_html("<!DOCTYPE html><html><body></body></html>")
            .build()
        {
            Ok(webview) => webview,
            Err(err) => {
                self.fail_create(event_loop, err.to_string());
                return;
            }
        };

        self.window = Some(window);
        self.webview = Some(webview);

        if self.auto_dismiss {
            // Close on the next about_to_wait tick so creation fully settles.
            self.pending_dismiss = true;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            self.dismiss(event_loop);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        pump_gtk_events();

        if self.pending_dismiss {
            self.pending_dismiss = false;
            self.dismiss(event_loop);
        }
    }
}

fn pump_gtk_events() {
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
