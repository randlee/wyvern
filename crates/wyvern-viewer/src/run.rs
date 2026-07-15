//! Event loop: load dialog URL; CLI stdin `exit` or OS close ends the process.

use std::fmt;
use std::io::{BufRead, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::http::Request;
use wry::{Rect, WebView, WebViewBuilder};

use crate::platform::{
    build_event_loop, init_platform, present_viewer_window, pump_gtk_events,
    viewer_window_attributes, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};
use wyvern_viewer::viewport::{HiddenUntilResize, ViewportBounds, FALLBACK_VIEWPORT};

/// Absolute floor for dialog chrome size.
const MIN_DIALOG_WIDTH: u32 = 200;
const MIN_DIALOG_HEIGHT: u32 = 96;
/// Accept refinement `resize:` IPC for this long after the first applied size.
const RESIZE_REFINEMENT_WINDOW: Duration = Duration::from_millis(300);

const DISMISS_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const DISMISS_IO_TIMEOUT: Duration = Duration::from_secs(2);

fn parse_resize_message(msg: &str, max_w: u32, max_h: u32) -> Option<(u32, u32)> {
    let rest = msg.strip_prefix("resize:")?;
    let (w, h) = rest.split_once('x')?;
    let w: u32 = w.parse().ok()?;
    let h: u32 = h.parse().ok()?;
    Some((
        w.clamp(MIN_DIALOG_WIDTH, max_w.max(MIN_DIALOG_WIDTH)),
        h.clamp(MIN_DIALOG_HEIGHT, max_h.max(MIN_DIALOG_HEIGHT)),
    ))
}

fn viewport_bounds_from_event_loop(event_loop: &ActiveEventLoop) -> ViewportBounds {
    let monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next());
    match monitor {
        Some(m) => {
            let size = m.size();
            ViewportBounds::from_physical(size.width, size.height, m.scale_factor())
                .unwrap_or(FALLBACK_VIEWPORT)
        }
        None => FALLBACK_VIEWPORT,
    }
}

/// Viewer process failure.
#[derive(Debug)]
pub enum ViewerError {
    /// Missing or invalid dialog URL.
    Usage {
        /// Human-readable detail.
        message: String,
        /// Optional upstream / parse cause for structured stderr.
        cause: Option<String>,
    },
    /// Window / event-loop failure.
    EventLoop {
        /// Failure detail.
        message: String,
        /// Optional upstream platform cause for structured stderr.
        cause: Option<String>,
    },
}

impl ViewerError {
    /// Optional structured cause string (included in stderr envelope when set).
    #[must_use]
    pub fn cause(&self) -> Option<&str> {
        match self {
            Self::Usage { cause, .. } | Self::EventLoop { cause, .. } => cause.as_deref(),
        }
    }
}

impl fmt::Display for ViewerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage { message, .. } | Self::EventLoop { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for ViewerError {}

/// Launch options resolved from argv / env.
#[derive(Debug, Clone)]
pub struct ViewerArgs {
    /// Dialog page URL (`http://127.0.0.1:PORT/{type}/`).
    pub dialog_url: String,
    /// Window title.
    pub title: String,
    /// Logical width.
    pub width: f64,
    /// Logical height.
    pub height: f64,
}

/// Parse argv + env into [`ViewerArgs`] and run the window.
///
/// Env: `WYVERN_DIALOG_URL` (required if not passed as first positional).
/// Optional: `WYVERN_VIEWER_WIDTH`, `WYVERN_VIEWER_HEIGHT`, `WYVERN_VIEWER_TITLE`.
/// Opt-in non-loopback: `WYVERN_VIEWER_ALLOW_NON_LOOPBACK=1`.
pub fn run_from_env_and_args(args: Vec<String>) -> Result<(), ViewerError> {
    let dialog_url = args
        .first()
        .cloned()
        .or_else(|| std::env::var("WYVERN_DIALOG_URL").ok())
        .ok_or_else(|| ViewerError::Usage {
            message: "missing dialog URL (pass as argv[1] or set WYVERN_DIALOG_URL)".into(),
            cause: None,
        })?;

    let parsed = Url::parse(&dialog_url).map_err(|e| ViewerError::Usage {
        message: format!("invalid dialog URL '{dialog_url}'"),
        cause: Some(e.to_string()),
    })?;
    enforce_dialog_url_policy(&parsed)?;

    let width = std::env::var("WYVERN_VIEWER_WIDTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WIDTH);
    let height = std::env::var("WYVERN_VIEWER_HEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_HEIGHT);
    let title = std::env::var("WYVERN_VIEWER_TITLE").unwrap_or_else(|_| "Wyvern".into());

    run(ViewerArgs {
        dialog_url,
        title,
        width,
        height,
    })
}

/// Reject non-http(s) schemes and non-loopback hosts unless explicitly opted in.
fn enforce_dialog_url_policy(url: &Url) -> Result<(), ViewerError> {
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(ViewerError::Usage {
            message: format!("refusing dialog URL scheme '{scheme}'; only http/https are allowed"),
            cause: None,
        });
    }

    let allow_non_loopback = std::env::var_os("WYVERN_VIEWER_ALLOW_NON_LOOPBACK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if allow_non_loopback {
        return Ok(());
    }

    let host = url.host_str().ok_or_else(|| ViewerError::Usage {
        message: "dialog URL is missing a host".into(),
        cause: None,
    })?;
    if is_loopback_host(host) {
        return Ok(());
    }
    Err(ViewerError::Usage {
        message: format!(
            "refusing non-loopback dialog host '{host}'; set WYVERN_VIEWER_ALLOW_NON_LOOPBACK=1 to opt in"
        ),
        cause: None,
    })
}

fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    // url::Url::host_str keeps brackets for IPv6 literals (`[::1]`).
    let bare = host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(addr) = bare.parse::<std::net::IpAddr>() {
        return addr.is_loopback();
    }
    false
}

/// Open an embedded webview for `args.dialog_url`.
pub fn run(args: ViewerArgs) -> Result<(), ViewerError> {
    init_platform()?;

    let event_loop = build_event_loop()?;
    let close_requested = Arc::new(AtomicBool::new(false));
    spawn_cli_exit_watcher(Arc::clone(&close_requested));

    let mut app = ViewerApp {
        args,
        window: None,
        webview: None,
        posted_dismiss: false,
        pending_resize: Arc::new(Mutex::new(None)),
        pending_navigate: Arc::new(Mutex::new(None)),
        viewport: FALLBACK_VIEWPORT,
        first_resize_at: None,
        present_gate: HiddenUntilResize::new(),
        bounds_injected: false,
        close_requested,
        closing: false,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|err| ViewerError::EventLoop {
            message: "event loop failed".into(),
            cause: Some(err.to_string()),
        })?;

    pump_gtk_events();
    Ok(())
}

/// Background thread: CLI writes `exit\n` on stdin after host accepts POST /api/result.
fn spawn_cli_exit_watcher(close_requested: Arc<AtomicBool>) {
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut reader = std::io::BufReader::new(stdin.lock());
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line.trim() == "exit" {
                        close_requested.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn webview_bounds_for_window(window: &Window) -> Rect {
    let scale = window.scale_factor();
    let size = window.inner_size();
    Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(size.width as f64 / scale, size.height as f64 / scale).into(),
    }
}

fn fit_child_webview(webview: &WebView, window: &Window) {
    let _ = webview.set_bounds(webview_bounds_for_window(window));
}

struct ViewerApp {
    args: ViewerArgs,
    window: Option<Arc<Window>>,
    webview: Option<WebView>,
    posted_dismiss: bool,
    pending_resize: Arc<Mutex<Option<(u32, u32)>>>,
    pending_navigate: Arc<Mutex<Option<String>>>,
    viewport: ViewportBounds,
    first_resize_at: Option<Instant>,
    present_gate: HiddenUntilResize,
    bounds_injected: bool,
    close_requested: Arc<AtomicBool>,
    closing: bool,
}

impl ViewerApp {
    /// Graceful shutdown: release WebKit, then let winit/AppKit close the window.
    ///
    /// Do not drop [`Window`] while the event loop is still dispatching AppKit
    /// notifications (`windowDidResignKey:`) — winit 0.30 panics if the view's
    /// weak window ref is cleared mid-callback. `event_loop.exit()` runs
    /// `notify_windows_of_exit`, which closes windows through the normal delegate path.
    fn request_shutdown(&mut self, event_loop: &ActiveEventLoop) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.close_requested.store(false, Ordering::Relaxed);
        if let Some(window) = &self.window {
            window.set_visible(false);
        }
        // WKWebView child must detach before NSWindow closes (wry).
        self.webview.take();
        pump_gtk_events();
        event_loop.exit();
    }

    fn accepts_resize(&self) -> bool {
        match self.first_resize_at {
            None => true,
            Some(started) => started.elapsed() <= RESIZE_REFINEMENT_WINDOW,
        }
    }

    fn inject_viewport_bounds_if_needed(&mut self) {
        if self.bounds_injected {
            return;
        }
        let Some(webview) = self.webview.as_ref() else {
            return;
        };
        let script = self.viewport.dispatch_script();
        if let Err(err) = webview.evaluate_script(&script) {
            tracing::warn!(error = %err, "failed to inject viewport bounds");
            return;
        }
        self.bounds_injected = true;
    }

    fn apply_pending_resize(&mut self) {
        if !self.accepts_resize() {
            // Drain late messages so they do not apply after the refinement window.
            if let Ok(mut guard) = self.pending_resize.lock() {
                *guard = None;
            }
            return;
        }
        let pending = self
            .pending_resize
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());
        let Some((width, height)) = pending else {
            return;
        };
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let size = LogicalSize::new(width as f64, height as f64);
        let _ = window.request_inner_size(size);
        if let Some(webview) = &self.webview {
            fit_child_webview(webview, window);
        }
        if self.first_resize_at.is_none() {
            self.first_resize_at = Some(Instant::now());
        }
        if self.present_gate.note_content_resize() {
            present_viewer_window(window);
        }
    }

    fn apply_pending_navigate(&mut self) {
        let pending = self
            .pending_navigate
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());
        let Some(url) = pending else {
            return;
        };
        let Some(webview) = self.webview.as_ref() else {
            return;
        };
        if let Err(err) = webview.load_url(&url) {
            tracing::error!(error = %err, url = %url, "failed to navigate viewer");
            return;
        }
        // New page: hide until its first resize; re-inject bounds.
        self.first_resize_at = None;
        self.bounds_injected = false;
        self.present_gate.note_navigate();
        if let Some(window) = self.window.as_ref() {
            window.set_visible(false);
        }
    }
}

impl ApplicationHandler for ViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        self.viewport = viewport_bounds_from_event_loop(event_loop);
        let max_w = self.viewport.available_width();
        let max_h = self.viewport.available_height();

        let attrs = viewer_window_attributes(&self.args.title, self.args.width, self.args.height);
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(err) => {
                tracing::error!(error = %err, "failed to create viewer window");
                event_loop.exit();
                return;
            }
        };

        let pending_resize = Arc::clone(&self.pending_resize);
        let pending_navigate = Arc::clone(&self.pending_navigate);
        let close_requested = Arc::clone(&self.close_requested);
        let init_script = self.viewport.dispatch_script();
        let builder = WebViewBuilder::new()
            .with_url(&self.args.dialog_url)
            .with_initialization_script(&init_script)
            .with_devtools(cfg!(debug_assertions))
            .with_ipc_handler(move |req: Request<String>| {
                let msg = req.body();
                // Legacy: pages no longer send "close" after POST; CLI uses stdin `exit`.
                if msg == "close" {
                    close_requested.store(true, Ordering::Relaxed);
                    return;
                }
                if let Some(url) = msg.strip_prefix("navigate:") {
                    if let Ok(mut slot) = pending_navigate.lock() {
                        *slot = Some(url.to_string());
                    }
                    return;
                }
                if let Some(size) = parse_resize_message(msg, max_w, max_h) {
                    if let Ok(mut slot) = pending_resize.lock() {
                        *slot = Some(size);
                    }
                }
            });

        // macOS: child webview + explicit bounds (stable focus/alt-tab vs `build()`).
        #[cfg(target_os = "macos")]
        let webview = match builder.build_as_child(&*window) {
            Ok(wv) => {
                fit_child_webview(&wv, &window);
                wv
            }
            Err(err) => {
                tracing::error!(error = %err, "failed to create webview");
                event_loop.exit();
                return;
            }
        };

        #[cfg(not(target_os = "macos"))]
        let webview = match builder.build_as_child(&*window) {
            Ok(wv) => {
                fit_child_webview(&wv, &window);
                wv
            }
            Err(err) => {
                match WebViewBuilder::new()
                    .with_url(&self.args.dialog_url)
                    .with_initialization_script(&init_script)
                    .with_devtools(cfg!(debug_assertions))
                    .build(&*window)
                {
                    Ok(wv) => wv,
                    Err(err2) => {
                        tracing::error!(error = %err, fallback = %err2, "failed to create webview");
                        event_loop.exit();
                        return;
                    }
                }
            }
        };

        self.webview = Some(webview);
        self.window = Some(window);
        // Intentionally do NOT present yet — wait for first content resize (no 320×240 flash).
        self.inject_viewport_bounds_if_needed();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.close_requested.load(Ordering::Relaxed) {
            self.request_shutdown(event_loop);
            return;
        }
        self.inject_viewport_bounds_if_needed();
        self.apply_pending_resize();
        self.apply_pending_navigate();
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.webview.take();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.closing {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                if !self.posted_dismiss {
                    post_dismissed(&self.args.dialog_url);
                    self.posted_dismiss = true;
                }
                self.request_shutdown(event_loop);
            }
            WindowEvent::Destroyed => {
                self.window.take();
            }
            WindowEvent::Resized(_size) => {
                if let (Some(window), Some(webview)) = (&self.window, &self.webview) {
                    fit_child_webview(webview, window);
                }
            }
            _ => {}
        }
    }
}

/// Derive `http://host:port/api/result` from a dialog URL and POST dismissed.
fn post_dismissed(dialog_url: &str) {
    let Ok(mut url) = Url::parse(dialog_url) else {
        return;
    };
    url.set_path("/api/result");
    url.set_query(None);
    url.set_fragment(None);
    let body = r#"{"button":"dismissed"}"#;
    // Blocking HTTP from the UI thread is acceptable for one-shot dismiss on close,
    // but connect/read/write must be bounded so a stalled host cannot hang the process.
    let client_result = timed_post(url.as_str(), body);
    if let Err(err) = client_result {
        tracing::warn!(error = %err, "viewer dismiss POST failed");
    }
}

/// Minimal POST without adding reqwest to the viewer binary (keep deps small).
fn timed_post(url: &str, body: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|e| e.to_string())?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "missing host".to_string())?;
    let port = parsed.port_or_known_default().unwrap_or(80);
    let path = if parsed.path().is_empty() {
        "/"
    } else {
        parsed.path()
    };

    let addr = format!("{host}:{port}");
    let mut last_err = None;
    for socket in addr
        .to_socket_addrs()
        .map_err(|e| format!("resolve {addr}: {e}"))?
    {
        match connect_with_timeout(socket) {
            Ok(mut stream) => {
                stream
                    .set_read_timeout(Some(DISMISS_IO_TIMEOUT))
                    .map_err(|e| format!("set_read_timeout: {e}"))?;
                stream
                    .set_write_timeout(Some(DISMISS_IO_TIMEOUT))
                    .map_err(|e| format!("set_write_timeout: {e}"))?;
                let request = format!(
                    "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(request.as_bytes())
                    .map_err(|e| format!("write: {e}"))?;
                // Best-effort read so the server can finish; ignore body / timeout.
                let mut buf = [0u8; 256];
                let _ = stream.read(&mut buf);
                return Ok(());
            }
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap_or_else(|| format!("connect {addr}: no addresses")))
}

fn connect_with_timeout(addr: SocketAddr) -> Result<TcpStream, String> {
    TcpStream::connect_timeout(&addr, DISMISS_CONNECT_TIMEOUT)
        .map_err(|e| format!("connect {addr}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hosts_accepted() {
        for url in [
            "http://127.0.0.1:9/message/",
            "http://localhost:9/chrome/",
            "http://[::1]:9/input/",
        ] {
            let parsed = Url::parse(url).unwrap();
            enforce_dialog_url_policy(&parsed).expect(url);
        }
    }

    #[test]
    fn non_loopback_rejected_by_default() {
        let parsed = Url::parse("http://example.com:9/message/").unwrap();
        let err = enforce_dialog_url_policy(&parsed).expect_err("non-loopback");
        assert!(matches!(err, ViewerError::Usage { .. }));
    }

    #[test]
    fn non_http_scheme_rejected() {
        let parsed = Url::parse("file:///tmp/x").unwrap();
        let err = enforce_dialog_url_policy(&parsed).expect_err("scheme");
        assert!(matches!(err, ViewerError::Usage { .. }));
    }

    #[test]
    fn parse_resize_clamps_to_viewport_max() {
        let (w, h) = parse_resize_message("resize:5000x4000", 1920, 1080).unwrap();
        assert_eq!((w, h), (1920, 1080));
        let (w, h) = parse_resize_message("resize:100x50", 1920, 1080).unwrap();
        assert_eq!((w, h), (MIN_DIALOG_WIDTH, MIN_DIALOG_HEIGHT));
    }
}
