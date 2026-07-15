//! OS-close dismiss POST (REQ-0097 / d.8).
//!
//! Blocking dialogs POST `{ "button": "dismissed" }` to `/api/result`.
//! Wizard sessions GET `/api/wizard/state`, build the full visited stack, then
//! `POST /api/wizard/finish` with `{ "button": "dismissed", ... }` before exit.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::{json, Value};
use url::Url;

const DISMISS_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const DISMISS_IO_TIMEOUT: Duration = Duration::from_secs(2);
/// Cumulative budget for wizard GET state + POST finish (RSH-002).
const DISMISS_WIZARD_TOTAL_BUDGET: Duration = Duration::from_secs(4);
/// Budget for a single blocking `/api/result` POST.
const DISMISS_BLOCKING_BUDGET: Duration = Duration::from_secs(4);

/// Structured dismiss-path failure (RBP-F001).
#[derive(Debug)]
pub enum DismissError {
    /// Dialog / API URL could not be parsed or built.
    Url {
        /// Human-readable detail.
        message: String,
        /// Optional parse cause.
        cause: Option<String>,
    },
    /// DNS resolve or TCP connect failed.
    Connect {
        /// Human-readable detail.
        message: String,
        /// Optional I/O cause.
        cause: Option<String>,
    },
    /// Non-2xx HTTP status from the host.
    HttpStatus {
        /// HTTP method.
        method: String,
        /// Request URL.
        url: String,
        /// Response status code.
        status: u16,
    },
    /// Connect or I/O budget exhausted before a complete response.
    Timeout {
        /// Human-readable detail.
        message: String,
    },
    /// Read timed out or EOF left an incomplete HTTP response (RSH-001).
    IncompleteResponse {
        /// Human-readable detail.
        message: String,
    },
    /// Wizard state / finish body JSON failure.
    Json {
        /// Human-readable detail.
        message: String,
        /// Upstream serde cause.
        cause: String,
    },
    /// Required wizard-state field missing after typed deserialize.
    MissingField {
        /// Field name.
        field: &'static str,
    },
    /// Low-level socket I/O failure.
    Io {
        /// Human-readable detail.
        message: String,
        /// Upstream I/O cause.
        cause: String,
    },
}

impl DismissError {
    fn recovery(&self) -> &'static str {
        match self {
            Self::Url { .. } => "Ensure WYVERN_DIALOG_URL is a valid http(s) URL from the host.",
            Self::Connect { .. } => {
                "Confirm the host is still listening on the dialog URL before OS-close."
            }
            Self::HttpStatus { .. } => {
                "Retry dismiss after the host accepts wizard finish / result POSTs."
            }
            Self::Timeout { .. } | Self::IncompleteResponse { .. } => {
                "Host may be overloaded; CLI viewer-exit fallback should still emit dismissed."
            }
            Self::Json { .. } | Self::MissingField { .. } => {
                "Host GET /api/wizard/state must return page, page_data, and stack."
            }
            Self::Io { .. } => "Check local networking to the loopback dialog host.",
        }
    }

    fn cause(&self) -> Option<&str> {
        match self {
            Self::Url { cause, .. } | Self::Connect { cause, .. } => cause.as_deref(),
            Self::Json { cause, .. } | Self::Io { cause, .. } => Some(cause.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for DismissError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url { message, .. }
            | Self::Connect { message, .. }
            | Self::Timeout { message }
            | Self::IncompleteResponse { message }
            | Self::Json { message, .. }
            | Self::Io { message, .. } => {
                write!(f, "{message} (recovery: {})", self.recovery())
            }
            Self::HttpStatus {
                method,
                url,
                status,
            } => write!(
                f,
                "{method} {url} returned HTTP {status} (recovery: {})",
                self.recovery()
            ),
            Self::MissingField { field } => write!(
                f,
                "wizard state missing {field} (recovery: {})",
                self.recovery()
            ),
        }
    }
}

impl std::error::Error for DismissError {}

/// Boundary-local DTO for `GET /api/wizard/state` (RBP-F003).
///
/// Kept in the viewer so `wyvern-schema` stays out of the dependency boundary.
#[derive(Debug, Clone, Deserialize)]
pub struct WizardStateDto {
    page: Value,
    page_data: Value,
    stack: Vec<Value>,
}

/// Best-effort dismiss POST for the dialog loaded at `dialog_url`.
///
/// Wizard URLs (`/wizard/…`) use the finish stack algorithm; other dialogs use
/// `/api/result`. Failures are logged only — never fatal to process exit.
pub fn post_dismissed(dialog_url: &str) {
    let result = if is_wizard_dialog_url(dialog_url) {
        post_wizard_dismissed(dialog_url)
    } else {
        post_blocking_dismissed(dialog_url)
    };
    if let Err(err) = result {
        tracing::warn!(
            error = %err,
            cause = err.cause().unwrap_or(""),
            recovery = err.recovery(),
            "viewer dismiss POST failed"
        );
    }
}

/// True when the dialog URL path is under `/wizard/` (d.1 wizard handoff).
#[must_use]
pub fn is_wizard_dialog_url(dialog_url: &str) -> bool {
    Url::parse(dialog_url)
        .map(|u| {
            let path = u.path();
            path == "/wizard" || path.starts_with("/wizard/")
        })
        .unwrap_or(false)
}

/// Build the `POST /api/wizard/finish` body from a typed wizard-state DTO.
///
/// Full visited stack = prior `stack` + `{ page, data: page_data }` (d.2 / d.8).
/// Request `data` is `page_data` so host stack validation matches `finish`.
///
/// Moves `stack` / `page` / `page_data` once into the JSON value instead of
/// eagerly cloning each field (RBP-F006).
///
/// # Errors
///
/// Returns [`DismissError::MissingField`] when required fields are null / wrong shape.
pub fn wizard_dismiss_finish_body(state: &WizardStateDto) -> Result<Value, DismissError> {
    if state.page.is_null() {
        return Err(DismissError::MissingField { field: "page" });
    }
    if state.page_data.is_null() {
        return Err(DismissError::MissingField { field: "page_data" });
    }

    // Clone once into owned Values, then move them into the request body.
    let page = state.page.clone();
    let page_data = state.page_data.clone();
    let mut full_stack = state.stack.clone();
    full_stack.push(json!({
        "page": page,
        "data": page_data.clone(),
    }));

    Ok(json!({
        "button": "dismissed",
        "data": page_data,
        "stack": full_stack,
    }))
}

fn post_blocking_dismissed(dialog_url: &str) -> Result<(), DismissError> {
    let url = api_url(dialog_url, "/api/result")?;
    let deadline = Instant::now() + DISMISS_BLOCKING_BUDGET;
    timed_post(url.as_str(), r#"{"button":"dismissed"}"#, deadline)
}

fn post_wizard_dismissed(dialog_url: &str) -> Result<(), DismissError> {
    let deadline = Instant::now() + DISMISS_WIZARD_TOTAL_BUDGET;
    let state_url = api_url(dialog_url, "/api/wizard/state")?;
    let state_raw = timed_get(state_url.as_str(), deadline)?;
    let state: WizardStateDto =
        serde_json::from_str(&state_raw).map_err(|e| DismissError::Json {
            message: "wizard state JSON parse failed".into(),
            cause: e.to_string(),
        })?;
    let body = wizard_dismiss_finish_body(&state)?;
    let body_str = serde_json::to_string(&body).map_err(|e| DismissError::Json {
        message: "serialize finish body failed".into(),
        cause: e.to_string(),
    })?;
    let finish_url = api_url(dialog_url, "/api/wizard/finish")?;
    timed_post(finish_url.as_str(), &body_str, deadline)
}

fn api_url(dialog_url: &str, path: &str) -> Result<Url, DismissError> {
    let mut url = Url::parse(dialog_url).map_err(|e| DismissError::Url {
        message: format!("invalid dialog URL '{dialog_url}'"),
        cause: Some(e.to_string()),
    })?;
    url.set_path(path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn timed_get(url: &str, deadline: Instant) -> Result<String, DismissError> {
    let (status, body) = timed_http("GET", url, None, deadline)?;
    if !(200..300).contains(&status) {
        return Err(DismissError::HttpStatus {
            method: "GET".into(),
            url: url.into(),
            status,
        });
    }
    Ok(body)
}

fn timed_post(url: &str, body: &str, deadline: Instant) -> Result<(), DismissError> {
    let (status, _) = timed_http("POST", url, Some(body), deadline)?;
    if !(200..300).contains(&status) {
        return Err(DismissError::HttpStatus {
            method: "POST".into(),
            url: url.into(),
            status,
        });
    }
    Ok(())
}

fn remaining_budget(deadline: Instant) -> Result<Duration, DismissError> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        Err(DismissError::Timeout {
            message: "dismiss cumulative deadline exhausted".into(),
        })
    } else {
        Ok(left)
    }
}

/// Minimal HTTP/1.1 client without reqwest (keep viewer deps small).
fn timed_http(
    method: &str,
    url: &str,
    body: Option<&str>,
    deadline: Instant,
) -> Result<(u16, String), DismissError> {
    let parsed = Url::parse(url).map_err(|e| DismissError::Url {
        message: format!("invalid request URL '{url}'"),
        cause: Some(e.to_string()),
    })?;
    let host = parsed.host_str().ok_or_else(|| DismissError::Url {
        message: "missing host".into(),
        cause: None,
    })?;
    let port = parsed.port_or_known_default().unwrap_or(80);
    let path = if parsed.path().is_empty() {
        "/"
    } else {
        parsed.path()
    };

    let addr = format!("{host}:{port}");
    let mut last_err = None;
    let sockets = addr.to_socket_addrs().map_err(|e| DismissError::Connect {
        message: format!("resolve {addr}"),
        cause: Some(e.to_string()),
    })?;
    for socket in sockets {
        let connect_budget = remaining_budget(deadline)?.min(DISMISS_CONNECT_TIMEOUT);
        match connect_with_timeout(socket, connect_budget) {
            Ok(mut stream) => {
                let io_budget = remaining_budget(deadline)?.min(DISMISS_IO_TIMEOUT);
                stream
                    .set_read_timeout(Some(io_budget))
                    .map_err(|e| DismissError::Io {
                        message: "set_read_timeout failed".into(),
                        cause: e.to_string(),
                    })?;
                stream
                    .set_write_timeout(Some(io_budget))
                    .map_err(|e| DismissError::Io {
                        message: "set_write_timeout failed".into(),
                        cause: e.to_string(),
                    })?;

                let request = match body {
                    Some(body) => format!(
                        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    ),
                    None => format!(
                        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"
                    ),
                };
                stream
                    .write_all(request.as_bytes())
                    .map_err(|e| DismissError::Io {
                        message: "write request failed".into(),
                        cause: e.to_string(),
                    })?;

                let mut response = Vec::new();
                let mut buf = [0u8; 1024];
                loop {
                    if remaining_budget(deadline).is_err() {
                        return Err(incomplete_or_timeout(&response, "cumulative deadline"));
                    }
                    match stream.read(&mut buf) {
                        Ok(0) => {
                            // EOF: headers required; Content-Length must be satisfied if present.
                            if !headers_complete(&response) {
                                return Err(DismissError::IncompleteResponse {
                                    message: "HTTP response incomplete on connection close".into(),
                                });
                            }
                            if content_length_unsatisfied(&response) {
                                return Err(DismissError::IncompleteResponse {
                                    message: "HTTP body shorter than Content-Length on close"
                                        .into(),
                                });
                            }
                            break;
                        }
                        Ok(n) => response.extend_from_slice(&buf[..n]),
                        Err(err)
                            if err.kind() == std::io::ErrorKind::WouldBlock
                                || err.kind() == std::io::ErrorKind::TimedOut =>
                        {
                            // RSH-001: incomplete reads on timeout must not parse as 2xx.
                            return Err(incomplete_or_timeout(&response, "read timeout"));
                        }
                        Err(err) => {
                            return Err(DismissError::Io {
                                message: "read response failed".into(),
                                cause: err.to_string(),
                            });
                        }
                    }
                    if content_length_satisfied(&response) {
                        break;
                    }
                    if response.len() > 64 * 1024 {
                        return Err(DismissError::IncompleteResponse {
                            message: "HTTP response exceeded 64 KiB".into(),
                        });
                    }
                }
                return parse_http_response(&response);
            }
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap_or_else(|| DismissError::Connect {
        message: format!("connect {addr}: no addresses"),
        cause: None,
    }))
}

fn incomplete_or_timeout(raw: &[u8], reason: &str) -> DismissError {
    if raw.is_empty() || !headers_complete(raw) {
        DismissError::Timeout {
            message: format!("HTTP {reason} before headers completed"),
        }
    } else if content_length_unsatisfied(raw) {
        DismissError::IncompleteResponse {
            message: format!("HTTP {reason} with incomplete body"),
        }
    } else {
        // Headers complete and no unsatisfied Content-Length — still treat
        // mid-read timeout as incomplete when body may still be in flight
        // (no Content-Length / chunked).
        DismissError::IncompleteResponse {
            message: format!("HTTP {reason} before response finished"),
        }
    }
}

fn headers_complete(raw: &[u8]) -> bool {
    let text = String::from_utf8_lossy(raw);
    text.contains("\r\n\r\n") || text.contains("\n\n")
}

fn split_headers_body(raw: &[u8]) -> Option<(String, String)> {
    let text = String::from_utf8_lossy(raw);
    text.split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .map(|(h, b)| (h.to_string(), b.to_string()))
}

fn content_length_value(raw: &[u8]) -> Option<usize> {
    let (header, _) = split_headers_body(raw)?;
    header.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("content-length") {
            value.trim().parse::<usize>().ok()
        } else {
            None
        }
    })
}

fn content_length_satisfied(raw: &[u8]) -> bool {
    let Some(len) = content_length_value(raw) else {
        return false;
    };
    let Some((_, body)) = split_headers_body(raw) else {
        return false;
    };
    body.len() >= len
}

fn content_length_unsatisfied(raw: &[u8]) -> bool {
    let Some(len) = content_length_value(raw) else {
        return false;
    };
    let Some((_, body)) = split_headers_body(raw) else {
        return true;
    };
    body.len() < len
}

fn connect_with_timeout(addr: SocketAddr, budget: Duration) -> Result<TcpStream, DismissError> {
    TcpStream::connect_timeout(&addr, budget).map_err(|e| DismissError::Connect {
        message: format!("connect {addr}"),
        cause: Some(e.to_string()),
    })
}

fn parse_http_response(raw: &[u8]) -> Result<(u16, String), DismissError> {
    let text = String::from_utf8_lossy(raw);
    let (header, body) = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .ok_or_else(|| DismissError::IncompleteResponse {
            message: "HTTP response missing header terminator".into(),
        })?;
    let status_line = header.lines().next().unwrap_or("");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| DismissError::IncompleteResponse {
            message: format!("invalid HTTP status line: {status_line}"),
        })?;
    Ok((status, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_path_detection() {
        assert!(is_wizard_dialog_url(
            "http://127.0.0.1:9/wizard/pages/start.html"
        ));
        assert!(is_wizard_dialog_url("http://127.0.0.1:9/wizard/"));
        assert!(!is_wizard_dialog_url("http://127.0.0.1:9/message/"));
        assert!(!is_wizard_dialog_url("http://127.0.0.1:9/chrome/"));
        assert!(!is_wizard_dialog_url("not a url"));
    }

    fn dto_from(value: Value) -> WizardStateDto {
        serde_json::from_value(value).expect("dto")
    }

    #[test]
    fn finish_body_appends_current_page_to_prior_stack() {
        let state = dto_from(json!({
            "type": "wizard",
            "config": {},
            "page": { "id": "b", "title": "b", "html": "pages/b.html" },
            "page_data": { "b": 2 },
            "stack": [
                {
                    "page": { "id": "a", "title": "a", "html": "pages/a.html" },
                    "data": { "a": 1 }
                }
            ]
        }));
        let body = wizard_dismiss_finish_body(&state).expect("body");
        assert_eq!(body["button"], "dismissed");
        assert_eq!(body["data"], json!({ "b": 2 }));
        let stack = body["stack"].as_array().expect("stack");
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0]["data"], json!({ "a": 1 }));
        assert_eq!(stack[1]["page"]["id"], "b");
        assert_eq!(stack[1]["data"], json!({ "b": 2 }));
    }

    #[test]
    fn finish_body_first_page_empty_prior_stack() {
        let state = dto_from(json!({
            "page": { "id": "a", "title": "a", "html": "pages/a.html" },
            "page_data": {},
            "stack": []
        }));
        let body = wizard_dismiss_finish_body(&state).expect("body");
        let stack = body["stack"].as_array().expect("stack");
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0]["page"]["id"], "a");
        assert_eq!(stack[0]["data"], json!({}));
    }

    #[test]
    fn content_length_not_satisfied_mid_body() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nshort";
        assert!(!content_length_satisfied(raw));
        assert!(content_length_unsatisfied(raw));
    }

    #[test]
    fn content_length_satisfied_when_body_matches() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        assert!(content_length_satisfied(raw));
        assert!(!content_length_unsatisfied(raw));
    }
}
