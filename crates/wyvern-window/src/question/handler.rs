//! Question dialog event-loop handler (`run_question` + `QuestionApp`).

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use wyvern_schema::{CommandResult, QuestionCard, QuestionResult};

use crate::chrome::{parse_chrome_ipc, ChromeIpc};
use crate::error::RunError;
use crate::window::{init_platform, modal_window_attributes, pump_gtk_events};

use super::{
    estimate_question_window_size, parse_question_page_ipc, render_question_html, QuestionPageIpc,
    QuestionRenderInput,
};

/// Env var that auto-dismisses the window after successful creation.
const AUTO_DISMISS_ENV: &str = "WYVERN_AUTO_DISMISS";

/// Env var that injects a raw IPC JSON body after the window opens (tests).
const INJECT_IPC_ENV: &str = "WYVERN_INJECT_IPC";

/// User events from IPC handler / test inject / auto-dismiss.
#[derive(Debug)]
enum DialogEvent {
    Ipc(String),
    AutoDismiss,
}

/// Open a question dialog and return the protocol result.
pub(crate) fn run_question(
    questions: Vec<QuestionCard>,
    questions_raw: Vec<serde_json::Value>,
) -> Result<CommandResult, RunError> {
    init_platform()?;

    let title = "Question";
    let html = render_question_html(&QuestionRenderInput {
        title,
        questions: &questions,
    });
    let (width, height) = estimate_question_window_size(&questions);

    let auto_dismiss = std::env::var_os(AUTO_DISMISS_ENV).is_some();
    let inject_ipc = std::env::var(INJECT_IPC_ENV).ok();

    let event_loop = EventLoop::<DialogEvent>::with_user_event()
        .build()
        .map_err(|err| RunError::EventLoop {
            message: err.to_string(),
        })?;
    let proxy = event_loop.create_proxy();

    let mut app = QuestionApp {
        title: title.to_string(),
        html,
        width,
        height,
        questions_raw,
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
        Ok(CommandResult::Question(QuestionResult::dismissed(
            app.questions_raw,
        )))
    })
}

struct QuestionApp {
    title: String,
    html: String,
    width: f64,
    height: f64,
    questions_raw: Vec<serde_json::Value>,
    proxy: EventLoopProxy<DialogEvent>,
    window: Option<Window>,
    webview: Option<wry::WebView>,
    auto_dismiss: bool,
    inject_ipc: Option<String>,
    pending_auto: bool,
    pending_inject: bool,
    outcome: Option<Result<CommandResult, RunError>>,
}

impl QuestionApp {
    fn finish_submitted(
        &mut self,
        event_loop: &ActiveEventLoop,
        answers: std::collections::HashMap<String, String>,
    ) {
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        let questions = self.questions_raw.clone();
        self.outcome = Some(Ok(CommandResult::Question(QuestionResult::submitted(
            questions, answers,
        ))));
        event_loop.exit();
    }

    fn dismiss(&mut self, event_loop: &ActiveEventLoop) {
        self.webview.take();
        pump_gtk_events();
        self.window.take();
        let questions = self.questions_raw.clone();
        self.outcome = Some(Ok(CommandResult::Question(QuestionResult::dismissed(
            questions,
        ))));
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
                ChromeIpc::WindowMinimize => return, // modal: no-op — must NOT dismiss
                ChromeIpc::WindowClose => {
                    self.dismiss(event_loop);
                    return;
                }
            }
        }
        match parse_question_page_ipc(raw) {
            Some(QuestionPageIpc::QuestionSubmitted { answers }) => {
                if answers.is_empty() {
                    // REQ-0068 fail-safe for empty answers.
                    self.dismiss(event_loop);
                } else {
                    self.finish_submitted(event_loop, answers);
                }
            }
            Some(QuestionPageIpc::Dismissed) => {
                self.dismiss(event_loop);
            }
            None => {
                eprintln!("wyvern-window: malformed or unknown IPC; dismissing: {raw}");
                self.dismiss(event_loop);
            }
        }
    }
}

impl ApplicationHandler<DialogEvent> for QuestionApp {
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
