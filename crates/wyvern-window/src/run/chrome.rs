//! Chrome window run loop and event handler.

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use wyvern_schema::{ButtonLabel, ChromeResult, ChromeStatus, ChromeTitle, CommandResult};

use crate::chrome::{
    parse_chrome_ipc, platform_chrome_for, render_chrome_html, ChromeIpc, CommandKind,
};
use crate::error::RunError;
use crate::window::{chrome_window_attributes, init_platform, pump_gtk_events};

use super::{DialogEvent, AUTO_DISMISS_ENV, INJECT_IPC_ENV};

pub(super) fn run_chrome(
    title: ChromeTitle,
    status: Option<ChromeStatus>,
) -> Result<CommandResult, RunError> {
    init_platform()?;

    let chrome = platform_chrome_for(CommandKind::Chrome);
    let html = render_chrome_html(title.as_str(), status.as_ref().map(|s| s.as_str()), chrome);
    let auto_dismiss = std::env::var_os(AUTO_DISMISS_ENV).is_some();
    let inject_ipc = std::env::var(INJECT_IPC_ENV).ok();

    let event_loop = EventLoop::<DialogEvent>::with_user_event()
        .build()
        .map_err(|err| RunError::EventLoop {
            message: err.to_string(),
        })?;
    let proxy = event_loop.create_proxy();

    let mut app = ChromeApp {
        title: title.into_inner(),
        html,
        proxy,
        window: None,
        webview: None,
        auto_dismiss,
        inject_ipc,
        pending_auto: false,
        pending_inject: false,
        outcome: None,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|err| RunError::EventLoop {
            message: err.to_string(),
        })?;

    app.outcome.unwrap_or_else(|| {
        Ok(CommandResult::Chrome(ChromeResult {
            button: ButtonLabel::dismissed(),
        }))
    })
}

struct ChromeApp {
    title: String,
    html: String,
    proxy: EventLoopProxy<DialogEvent>,
    window: Option<Window>,
    webview: Option<wry::WebView>,
    auto_dismiss: bool,
    inject_ipc: Option<String>,
    pending_auto: bool,
    pending_inject: bool,
    outcome: Option<Result<CommandResult, RunError>>,
}

impl ChromeApp {
    fn dismiss(&mut self, event_loop: &ActiveEventLoop) {
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        self.outcome = Some(Ok(CommandResult::Chrome(ChromeResult {
            button: ButtonLabel::dismissed(),
        })));
        event_loop.exit();
    }

    fn fail_create(&mut self, event_loop: &ActiveEventLoop, message: String) {
        self.webview.take();
        self.window.take();
        self.outcome = Some(Err(RunError::WindowCreate { message }));
        event_loop.exit();
    }

    fn handle_ipc(&mut self, event_loop: &ActiveEventLoop, raw: &str) {
        if let Some(msg) = parse_chrome_ipc(raw) {
            match msg {
                ChromeIpc::WindowClose => {
                    self.dismiss(event_loop);
                }
                ChromeIpc::WindowMinimize => {
                    if let Some(window) = &self.window {
                        // macOS CI: set_minimized + immediate harness teardown races
                        // objc2 WeakId (null deref / SIGABRT). When auto-dismiss will
                        // finish the run, skip the OS minimize — the test asserts that
                        // minimize IPC does not complete stdout by itself (CI-001).
                        if !self.auto_dismiss {
                            window.set_minimized(true);
                        }
                    }
                    // Non-modal: minimize only — no CommandResult / stdout yet.
                }
            }
            return;
        }
        eprintln!("wyvern-window: malformed chrome IPC; dismissing: {raw}");
        self.dismiss(event_loop);
    }
}

impl ApplicationHandler<DialogEvent> for ChromeApp {
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

        let proxy = self.proxy.clone();
        let webview = match WebViewBuilder::new(&window)
            .with_html(self.html.clone())
            .with_ipc_handler(move |req| {
                let body = req.body().clone();
                let _ = proxy.send_event(DialogEvent::Ipc(body));
            })
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

        if self.inject_ipc.is_some() {
            self.pending_inject = true;
        } else if self.auto_dismiss {
            self.pending_auto = true;
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: DialogEvent) {
        match event {
            DialogEvent::Ipc(raw) => self.handle_ipc(event_loop, &raw),
            DialogEvent::AutoDismiss => self.dismiss(event_loop),
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        pump_gtk_events();

        if self.pending_inject {
            self.pending_inject = false;
            if let Some(raw) = self.inject_ipc.take() {
                let _ = self.proxy.send_event(DialogEvent::Ipc(raw));
            }
            // After a non-completing inject (e.g. window_minimize), finish via auto-dismiss.
            if self.auto_dismiss {
                self.pending_auto = true;
            }
            return;
        }

        if self.pending_auto {
            self.pending_auto = false;
            let _ = self.proxy.send_event(DialogEvent::AutoDismiss);
        }
    }
}
