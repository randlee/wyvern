//! Public `run` entry: open chrome/message/input windows and return protocol results.

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use wyvern_schema::{
    ButtonLabel, ButtonsPreset, ChromeResult, ChromeStatus, ChromeTitle, Command, CommandResult,
    InputMode, InputResult, InputValue, MessageLevel, MessageResult,
};

use crate::chrome::render_chrome_html;
use crate::error::RunError;
use crate::input::{
    estimate_input_window_size, parse_input_page_ipc, render_input_html, InputPageIpc,
    InputRenderInput,
};
use crate::message::{
    estimate_message_window_size, parse_page_ipc, render_message_html, MessageRenderInput, PageIpc,
};
use crate::window::{
    chrome_window_attributes, init_platform, modal_window_attributes, pump_gtk_events,
};

/// Env var that auto-dismisses the window after successful creation.
///
/// Used by CI and crate tests so GUI paths do not block on interactive close.
const AUTO_DISMISS_ENV: &str = "WYVERN_AUTO_DISMISS";

/// Env var that injects a raw IPC JSON body after the window opens (tests).
///
/// Example: `{"kind":"button_pressed","label":"ok"}`.
const INJECT_IPC_ENV: &str = "WYVERN_INJECT_IPC";

/// User events from IPC handler / test inject / auto-dismiss.
#[derive(Debug)]
enum DialogEvent {
    Ipc(String),
    AutoDismiss,
}

/// Open a native window for `command` and return the protocol result.
///
/// # Errors
///
/// Returns [`RunError::EventLoop`] if the event loop cannot start, or
/// [`RunError::WindowCreate`] if the native window / webview fails to build.
pub fn run(command: Command) -> Result<CommandResult, RunError> {
    match command {
        Command::Chrome { title, status } => run_chrome(title, status),
        Command::Message {
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
        } => run_message(MessageRunArgs {
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
        }),
        Command::Input {
            title,
            message,
            status,
            icon,
            markdown,
            multiline,
            placeholder,
            default,
            mode,
            buttons,
        } => {
            debug_assert_eq!(
                mode,
                InputMode::Text,
                "file/folder modes must be rejected at validate()"
            );
            run_input(InputRunArgs {
                title,
                message,
                status,
                icon,
                markdown,
                multiline,
                placeholder,
                default,
                buttons,
            })
        }
    }
}

fn run_chrome(title: ChromeTitle, status: Option<ChromeStatus>) -> Result<CommandResult, RunError> {
    init_platform()?;

    let html = render_chrome_html(title.as_str(), status.as_ref().map(|s| s.as_str()));
    let auto_dismiss = std::env::var_os(AUTO_DISMISS_ENV).is_some();

    let event_loop = EventLoop::new().map_err(|err| RunError::EventLoop {
        message: err.to_string(),
    })?;

    let mut app = ChromeApp {
        title: title.into_inner(),
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
            button: ButtonLabel::dismissed(),
        }))
    })
}

struct MessageRunArgs {
    title: ChromeTitle,
    message: String,
    status: Option<ChromeStatus>,
    buttons: ButtonsPreset,
    custom_buttons: Option<Vec<String>>,
    default_button: Option<u32>,
    level: Option<MessageLevel>,
    icon: Option<String>,
    image: Option<String>,
    markdown: bool,
}

fn run_message(args: MessageRunArgs) -> Result<CommandResult, RunError> {
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

struct InputRunArgs {
    title: ChromeTitle,
    message: String,
    status: Option<ChromeStatus>,
    icon: Option<String>,
    markdown: bool,
    multiline: bool,
    placeholder: Option<String>,
    default: Option<String>,
    buttons: ButtonsPreset,
}

fn run_input(args: InputRunArgs) -> Result<CommandResult, RunError> {
    init_platform()?;

    let InputRunArgs {
        title,
        message,
        status,
        icon,
        markdown,
        multiline,
        placeholder,
        default,
        buttons,
    } = args;

    let html = render_input_html(&InputRenderInput {
        title: title.as_str(),
        message: &message,
        status: status.as_ref().map(|s| s.as_str()),
        icon: icon.as_deref(),
        markdown,
        multiline,
        placeholder: placeholder.as_deref(),
        default: default.as_deref(),
        buttons,
    })?;
    let button_count = buttons.button_count(None);
    let (width, height) = estimate_input_window_size(
        &message,
        button_count,
        status.is_some(),
        icon.is_some(),
        multiline,
    );

    let auto_dismiss = std::env::var_os(AUTO_DISMISS_ENV).is_some();
    let inject_ipc = std::env::var(INJECT_IPC_ENV).ok();

    let event_loop = EventLoop::<DialogEvent>::with_user_event()
        .build()
        .map_err(|err| RunError::EventLoop {
            message: err.to_string(),
        })?;
    let proxy = event_loop.create_proxy();

    let mut app = InputApp {
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
        Ok(CommandResult::Input(InputResult {
            button: ButtonLabel::dismissed(),
            input: None,
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
            return;
        }

        if self.pending_auto {
            self.pending_auto = false;
            let _ = self.proxy.send_event(DialogEvent::AutoDismiss);
        }
    }
}

struct InputApp {
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

impl InputApp {
    fn finish(
        &mut self,
        event_loop: &ActiveEventLoop,
        button: ButtonLabel,
        input: Option<InputValue>,
    ) {
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        self.outcome = Some(Ok(CommandResult::Input(InputResult { button, input })));
        event_loop.exit();
    }

    fn dismiss(&mut self, event_loop: &ActiveEventLoop) {
        self.finish(event_loop, ButtonLabel::dismissed(), None);
    }

    fn fail_create(&mut self, event_loop: &ActiveEventLoop, message: String) {
        self.webview.take();
        self.window.take();
        self.outcome = Some(Err(RunError::WindowCreate { message }));
        event_loop.exit();
    }

    fn handle_ipc(&mut self, event_loop: &ActiveEventLoop, raw: &str) {
        match parse_input_page_ipc(raw) {
            Some(InputPageIpc::InputSubmitted { button, value }) => {
                let label = ButtonLabel::new(button);
                // Cancel omits input; confirm buttons include the text value
                // (empty string allowed when the field is blank).
                let input = if label.as_str() == "cancel" {
                    None
                } else {
                    Some(InputValue::Text(value.unwrap_or_default()))
                };
                self.finish(event_loop, label, input);
            }
            Some(InputPageIpc::Dismissed) => {
                self.dismiss(event_loop);
            }
            None => {
                eprintln!("wyvern-window: malformed or unknown IPC; dismissing: {raw}");
                self.dismiss(event_loop);
            }
        }
    }
}

impl ApplicationHandler<DialogEvent> for InputApp {
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
            return;
        }

        if self.pending_auto {
            self.pending_auto = false;
            let _ = self.proxy.send_event(DialogEvent::AutoDismiss);
        }
    }
}
