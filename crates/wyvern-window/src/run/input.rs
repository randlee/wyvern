//! Input dialog run loop and event handler.

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use wyvern_schema::{
    ButtonLabel, ButtonsPreset, ChromeStatus, ChromeTitle, CommandResult, InputMode, InputResult,
    InputValue,
};

use crate::chrome::{parse_chrome_ipc, ChromeIpc};
use crate::error::RunError;
use crate::input::{
    estimate_input_window_size, parse_input_page_ipc, pick_file, pick_folder, render_input_html,
    InputPageIpc, InputRenderInput,
};
use crate::window::{init_platform, modal_window_attributes, pump_gtk_events};

use super::{DialogEvent, AUTO_DISMISS_ENV, INJECT_IPC_ENV};

pub(super) struct InputRunArgs {
    pub(super) title: ChromeTitle,
    pub(super) message: String,
    pub(super) status: Option<ChromeStatus>,
    pub(super) icon: Option<String>,
    pub(super) markdown: bool,
    pub(super) multiline: bool,
    pub(super) placeholder: Option<String>,
    pub(super) default: Option<String>,
    pub(super) mode: InputMode,
    pub(super) filter: Option<Vec<String>>,
    pub(super) multiple: bool,
    pub(super) start_path: Option<String>,
    pub(super) buttons: ButtonsPreset,
}

pub(super) fn run_input(args: InputRunArgs) -> Result<CommandResult, RunError> {
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
        mode,
        filter,
        multiple,
        start_path,
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
        mode,
        buttons,
    })?;
    let button_count = buttons.button_count(None);
    let picker_mode = matches!(mode, InputMode::File | InputMode::Folder);
    let (width, height) = estimate_input_window_size(
        &message,
        button_count,
        status.is_some(),
        icon.is_some(),
        multiline,
        picker_mode,
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
        mode,
        filter: filter.unwrap_or_default(),
        multiple,
        start_path,
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

struct InputApp {
    title: String,
    html: String,
    width: f64,
    height: f64,
    mode: InputMode,
    filter: Vec<String>,
    multiple: bool,
    start_path: Option<String>,
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

    fn open_picker(&self) -> Option<InputValue> {
        let start = self.start_path.as_deref().map(std::path::Path::new);
        match self.mode {
            InputMode::File => {
                let paths = pick_file(&self.filter, self.multiple, start)?;
                let strings: Vec<String> = paths
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                if self.multiple {
                    Some(InputValue::Paths(strings))
                } else {
                    Some(InputValue::Text(
                        strings.into_iter().next().unwrap_or_default(),
                    ))
                }
            }
            InputMode::Folder => {
                let path = pick_folder(start)?;
                Some(InputValue::Text(path.to_string_lossy().into_owned()))
            }
            InputMode::Text => None,
        }
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
        match parse_input_page_ipc(raw) {
            Some(InputPageIpc::InputSubmitted { button, value }) => {
                let label = ButtonLabel::new(button);
                // Cancel omits input and never opens the picker.
                if label.as_str() == "cancel" {
                    self.finish(event_loop, label, None);
                    return;
                }

                match self.mode {
                    InputMode::Text => {
                        // Confirm buttons include the text value (empty string allowed).
                        let input = Some(InputValue::Text(value.unwrap_or_default()));
                        self.finish(event_loop, label, input);
                    }
                    InputMode::File | InputMode::Folder => {
                        // Picker-on-OK: open rfd synchronously; cancel leaves dialog open.
                        match self.open_picker() {
                            Some(input) => self.finish(event_loop, label, Some(input)),
                            None => {
                                // Picker cancelled — keep dialog open; no stdout yet.
                            }
                        }
                    }
                }
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
