//! Message dialog run loop and event handler.

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use wyvern_schema::{
    ButtonLabel, ButtonsPreset, ChromeStatus, ChromeTitle, CommandResult, MessageLevel,
    MessageResult,
};

use crate::chrome::{parse_chrome_ipc, ChromeIpc};
use crate::error::RunError;
use crate::message::{
    estimate_message_window_size, parse_page_ipc, render_message_html, MessageRenderInput, PageIpc,
};
use crate::window::{init_platform, modal_window_attributes, pump_gtk_events};

use super::{DialogEvent, AUTO_DISMISS_ENV, INJECT_IPC_ENV};

pub(super) struct MessageRunArgs {
    pub(super) title: ChromeTitle,
    pub(super) message: String,
    pub(super) status: Option<ChromeStatus>,
    pub(super) buttons: ButtonsPreset,
    pub(super) custom_buttons: Option<Vec<String>>,
    pub(super) default_button: Option<u32>,
    pub(super) level: Option<MessageLevel>,
    pub(super) icon: Option<String>,
    pub(super) image: Option<String>,
    pub(super) markdown: bool,
}

pub(super) fn run_message(args: MessageRunArgs) -> Result<CommandResult, RunError> {
    init_platform()?;

    let MessageRunArgs {
        title,
        message,
        status,
        buttons,
        custom_buttons,
        default_button,
        level,
        icon,
        image,
        markdown,
    } = args;

    let custom_ref = custom_buttons.as_deref();
    let html = render_message_html(&MessageRenderInput {
        title: title.as_str(),
        message: &message,
        status: status.as_ref().map(|s| s.as_str()),
        buttons,
        custom_buttons: custom_ref,
        default_button,
        level,
        icon: icon.as_deref(),
        image: image.as_deref(),
        markdown,
    })?;
    let button_count = buttons.button_count(custom_ref);
    let (width, height) = estimate_message_window_size(
        &message,
        button_count,
        status.is_some(),
        level.is_some() || icon.is_some(),
        image.is_some(),
    );

    let auto_dismiss = std::env::var_os(AUTO_DISMISS_ENV).is_some();
    let inject_ipc = std::env::var(INJECT_IPC_ENV).ok();

    let event_loop = EventLoop::<DialogEvent>::with_user_event()
        .build()
        .map_err(|err| RunError::EventLoop {
            message: err.to_string(),
        })?;
    let proxy = event_loop.create_proxy();

    let mut app = MessageApp {
        title: title.into_inner(),
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
        Ok(CommandResult::Message(MessageResult {
            button: ButtonLabel::dismissed(),
        }))
    })
}

struct MessageApp {
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

impl MessageApp {
    fn finish_with_label(&mut self, event_loop: &ActiveEventLoop, label: ButtonLabel) {
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        self.outcome = Some(Ok(CommandResult::Message(MessageResult { button: label })));
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
        match parse_page_ipc(raw) {
            Some(PageIpc::ButtonPressed { label }) => {
                self.finish_with_label(event_loop, ButtonLabel::new(label));
            }
            Some(PageIpc::Dismissed) => {
                self.dismiss(event_loop);
            }
            None => {
                // Contract: malformed / unknown kind → log + fail-safe dismissed.
                // Observability lives in the CLI crate; log locally here.
                eprintln!("wyvern-window: malformed or unknown IPC; dismissing: {raw}");
                self.dismiss(event_loop);
            }
        }
    }
}

impl ApplicationHandler<DialogEvent> for MessageApp {
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
