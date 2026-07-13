//! Public `run` entry: open chrome and return a protocol result.

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use wyvern_schema::{ChromeResult, Command, CommandResult};

use crate::chrome::render_chrome_html;
use crate::error::RunError;
use crate::window::{chrome_window_attributes, init_platform, pump_gtk_events};

/// Env var that auto-dismisses the chrome window after successful creation.
///
/// Used by CI and crate tests so GUI paths do not block on interactive close.
const AUTO_DISMISS_ENV: &str = "WYVERN_AUTO_DISMISS";

/// Open a native window for `command` and return the protocol result.
///
/// Phase A supports only [`Command::Chrome`]. OS close yields
/// `{"button":"dismissed"}` via [`CommandResult::Chrome`].
///
/// # Errors
///
/// Returns [`RunError::EventLoop`] if the event loop cannot start, or
/// [`RunError::WindowCreate`] if the native window / webview fails to build.
///
/// # Panics
///
/// Does not panic under normal operation.
pub fn run(command: Command) -> Result<CommandResult, RunError> {
    match command {
        Command::Chrome { title, status } => run_chrome(title, status),
    }
}

fn run_chrome(title: String, status: Option<String>) -> Result<CommandResult, RunError> {
    init_platform()?;

    let html = render_chrome_html(&title, status.as_deref());
    // wry does not support data: URLs via `with_url`; `with_html` is the
    // supported inline-HTML path equivalent to loading a data URL.
    let auto_dismiss = std::env::var_os(AUTO_DISMISS_ENV).is_some();

    let event_loop = EventLoop::new().map_err(|err| RunError::EventLoop {
        message: err.to_string(),
    })?;

    let mut app = ChromeApp {
        title,
        html,
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

    app.outcome.unwrap_or_else(|| {
        Ok(CommandResult::Chrome(ChromeResult {
            button: "dismissed".into(),
        }))
    })
}

struct ChromeApp {
    title: String,
    html: String,
    window: Option<Window>,
    webview: Option<wry::WebView>,
    auto_dismiss: bool,
    pending_dismiss: bool,
    outcome: Option<Result<CommandResult, RunError>>,
}

impl ChromeApp {
    fn dismiss(&mut self, event_loop: &ActiveEventLoop) {
        // Drop webview before the winit window so WebKit can release GL/X resources
        // without GLXBadWindow under xvfb.
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        self.outcome = Some(Ok(CommandResult::Chrome(ChromeResult {
            button: "dismissed".into(),
        })));
        event_loop.exit();
    }

    fn fail_create(&mut self, event_loop: &ActiveEventLoop, message: String) {
        self.webview.take();
        self.window.take();
        self.outcome = Some(Err(RunError::WindowCreate { message }));
        event_loop.exit();
    }
}

impl ApplicationHandler for ChromeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match event_loop.create_window(chrome_window_attributes(&self.title)) {
            Ok(window) => window,
            Err(err) => {
                self.fail_create(event_loop, err.to_string());
                return;
            }
        };

        let webview = match WebViewBuilder::new(&window)
            .with_html(self.html.clone())
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
