//! Markdown dialog run loop and event handler.

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use wyvern_schema::{
    ButtonLabel, ButtonsPreset, ChromeStatus, ChromeTitle, CommandResult, MarkdownResult,
};

use crate::chrome::{parse_chrome_ipc, ChromeIpc};
use crate::error::RunError;
use crate::markdown::{
    estimate_markdown_window_size, parse_markdown_page_ipc, render_markdown_html, MarkdownPageIpc,
    MarkdownRenderInput,
};
use crate::window::{init_platform, modal_window_attributes, pump_gtk_events};

use super::{DialogEvent, AUTO_DISMISS_ENV, INJECT_IPC_ENV};

pub(super) struct MarkdownRunArgs {
    pub(super) title: Option<ChromeTitle>,
    pub(super) source: String,
    pub(super) status: Option<ChromeStatus>,
    pub(super) buttons: ButtonsPreset,
}

pub(super) fn run_markdown(args: MarkdownRunArgs) -> Result<CommandResult, RunError> {
    init_platform()?;

    let MarkdownRunArgs {
        title,
        source,
        status,
        buttons,
    } = args;

    let title_text = title
        .as_ref()
        .map(|t| t.as_str().to_string())
        .unwrap_or_else(|| "Markdown".into());

    let html = render_markdown_html(&MarkdownRenderInput {
        title: &title_text,
        source: &source,
        status: status.as_ref().map(|s| s.as_str()),
        buttons,
    });
    let button_count = buttons.button_count(None);
    let (width, height) = estimate_markdown_window_size(&source, button_count, status.is_some());

    let auto_dismiss = std::env::var_os(AUTO_DISMISS_ENV).is_some();
    let inject_ipc = std::env::var(INJECT_IPC_ENV).ok();

    let event_loop = EventLoop::<DialogEvent>::with_user_event()
        .build()
        .map_err(|err| RunError::EventLoop {
            message: err.to_string(),
        })?;
    let proxy = event_loop.create_proxy();

    let mut app = MarkdownApp {
        title: title_text,
        html,
        width,
        height,
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
        Ok(CommandResult::Markdown(MarkdownResult {
            button: ButtonLabel::dismissed(),
        }))
    })
}

struct MarkdownApp {
    title: String,
    html: String,
    width: f64,
    height: f64,
    proxy: EventLoopProxy<DialogEvent>,
    window: Option<Window>,
    webview: Option<wry::WebView>,
    auto_dismiss: bool,
    inject_ipc: Option<String>,
    pending_auto: bool,
    pending_inject: bool,
    outcome: Option<Result<CommandResult, RunError>>,
}

impl MarkdownApp {
    fn finish_with_label(&mut self, event_loop: &ActiveEventLoop, label: ButtonLabel) {
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        self.outcome = Some(Ok(CommandResult::Markdown(MarkdownResult {
            button: label,
        })));
        event_loop.exit();
    }

    fn dismiss(&mut self, event_loop: &ActiveEventLoop) {
        self.finish_with_label(event_loop, ButtonLabel::dismissed());
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
                ChromeIpc::WindowMinimize => return, // modal: no-op — must NOT dismiss
                ChromeIpc::WindowClose => {
                    self.dismiss(event_loop);
                    return;
                }
            }
        }
        match parse_markdown_page_ipc(raw) {
            Some(MarkdownPageIpc::ButtonPressed { label }) => {
                self.finish_with_label(event_loop, ButtonLabel::new(label));
            }
            Some(MarkdownPageIpc::Dismissed) => {
                self.dismiss(event_loop);
            }
            None => {
                eprintln!("wyvern-window: malformed or unknown IPC; dismissing: {raw}");
                self.dismiss(event_loop);
            }
        }
    }
}

impl ApplicationHandler<DialogEvent> for MarkdownApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match event_loop.create_window(modal_window_attributes(
            &self.title,
            self.width,
            self.height,
        )) {
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
            // After a non-completing inject (e.g. modal window_minimize no-op),
            // finish via auto-dismiss so integration tests can observe no early exit.
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
