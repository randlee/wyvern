//! Event loop: load dialog URL; on close POST `{ "button": "dismissed" }`.

use std::fmt;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;

use crate::platform::{
    init_platform, pump_gtk_events, viewer_window_attributes, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

/// Bounded timeouts for best-effort dismiss POST (must not block the UI thread indefinitely).
const DISMISS_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const DISMISS_IO_TIMEOUT: Duration = Duration::from_secs(2);

/// Viewer process failure.
#[derive(Debug)]
pub enum ViewerError {
    /// Missing or invalid dialog URL.
    Usage {
        /// Human-readable detail.
        message: String,
    },
    /// Window / event-loop failure.
    EventLoop {
        /// Failure detail.
        message: String,
    },
}

impl fmt::Display for ViewerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage { message } | Self::EventLoop { message } => f.write_str(message),
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
        })?;

    let parsed = Url::parse(&dialog_url).map_err(|e| ViewerError::Usage {
        message: format!("invalid dialog URL '{dialog_url}': {e}"),
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
    })?;
    if is_loopback_host(host) {
        return Ok(());
    }
    Err(ViewerError::Usage {
        message: format!(
            "refusing non-loopback dialog host '{host}'; set WYVERN_VIEWER_ALLOW_NON_LOOPBACK=1 to opt in"
        ),
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

    let event_loop = EventLoop::new().map_err(|err| ViewerError::EventLoop {
        message: err.to_string(),
    })?;

    let mut app = ViewerApp {
        args,
        window: None,
        webview: None,
        posted_dismiss: false,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|err| ViewerError::EventLoop {
            message: err.to_string(),
        })?;

    pump_gtk_events();
    Ok(())
}

struct ViewerApp {
    args: ViewerArgs,
    window: Option<Arc<Window>>,
    webview: Option<wry::WebView>,
    posted_dismiss: bool,
}

impl ApplicationHandler for ViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = viewer_window_attributes(&self.args.title, self.args.width, self.args.height);
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(err) => {
                tracing::error!(error = %err, "failed to create viewer window");
                event_loop.exit();
                return;
            }
        };

        let builder = WebViewBuilder::new()
            .with_url(&self.args.dialog_url)
            .with_devtools(cfg!(debug_assertions));

        let webview = match builder.build_as_child(&*window) {
            Ok(wv) => wv,
            Err(err) => {
                // Fallback: some platforms prefer `build` with window handle.
                match WebViewBuilder::new()
                    .with_url(&self.args.dialog_url)
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
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if !self.posted_dismiss {
                    post_dismissed(&self.args.dialog_url);
                    self.posted_dismiss = true;
                }
                self.webview.take();
                self.window.take();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let (Some(window), Some(webview)) = (&self.window, &self.webview) {
                    let _ = window;
                    let _ = webview;
                    // wry child webviews track resize via the window; nothing required on all platforms.
                    let _ = size;
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
}
