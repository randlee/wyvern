//! OS-close dismiss POST (REQ-0097 / d.8).
//!
//! Blocking dialogs POST `{ "button": "dismissed" }` to `/api/result`.
//! Wizard sessions GET `/api/wizard/state`, build the full visited stack, then
//! `POST /api/wizard/finish` with `{ "button": "dismissed", ... }` before exit.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

const DISMISS_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const DISMISS_IO_TIMEOUT: Duration = Duration::from_secs(2);

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
        tracing::warn!(error = %err, "viewer dismiss POST failed");
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

/// Build the `POST /api/wizard/finish` body from a `GET /api/wizard/state` JSON object.
///
/// Full visited stack = prior `stack` + `{ page, data: page_data }` (d.2 / d.8).
/// Request `data` is `page_data` so host stack validation matches `finish`.
///
/// # Errors
///
/// Returns an error when required fields are missing or not objects/arrays.
pub fn wizard_dismiss_finish_body(state: &Value) -> Result<Value, String> {
    let page = state
        .get("page")
        .cloned()
        .ok_or_else(|| "wizard state missing page".to_string())?;
    let page_data = state
        .get("page_data")
        .cloned()
        .ok_or_else(|| "wizard state missing page_data".to_string())?;
    let prior = state
        .get("stack")
        .and_then(Value::as_array)
        .ok_or_else(|| "wizard state missing stack array".to_string())?;

    let mut full_stack = prior.clone();
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

fn post_blocking_dismissed(dialog_url: &str) -> Result<(), String> {
    let url = api_url(dialog_url, "/api/result")?;
    timed_post(url.as_str(), r#"{"button":"dismissed"}"#)
}

fn post_wizard_dismissed(dialog_url: &str) -> Result<(), String> {
    let state_url = api_url(dialog_url, "/api/wizard/state")?;
    let state_raw = timed_get(state_url.as_str())?;
    let state: Value =
        serde_json::from_str(&state_raw).map_err(|e| format!("wizard state JSON: {e}"))?;
    let body = wizard_dismiss_finish_body(&state)?;
    let body_str =
        serde_json::to_string(&body).map_err(|e| format!("serialize finish body: {e}"))?;
    let finish_url = api_url(dialog_url, "/api/wizard/finish")?;
    timed_post(finish_url.as_str(), &body_str)
}

fn api_url(dialog_url: &str, path: &str) -> Result<Url, String> {
    let mut url = Url::parse(dialog_url).map_err(|e| e.to_string())?;
    url.set_path(path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn timed_get(url: &str) -> Result<String, String> {
    let (status, body) = timed_http("GET", url, None)?;
    if !(200..300).contains(&status) {
        return Err(format!("GET {url} returned HTTP {status}"));
    }
    Ok(body)
}

fn timed_post(url: &str, body: &str) -> Result<(), String> {
    let (status, _) = timed_http("POST", url, Some(body))?;
    if !(200..300).contains(&status) {
        return Err(format!("POST {url} returned HTTP {status}"));
    }
    Ok(())
}

/// Minimal HTTP/1.1 client without reqwest (keep viewer deps small).
fn timed_http(method: &str, url: &str, body: Option<&str>) -> Result<(u16, String), String> {
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
                    .map_err(|e| format!("write: {e}"))?;

                let mut response = Vec::new();
                let mut buf = [0u8; 1024];
                loop {
                    match stream.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => response.extend_from_slice(&buf[..n]),
                        Err(err)
                            if err.kind() == std::io::ErrorKind::WouldBlock
                                || err.kind() == std::io::ErrorKind::TimedOut =>
                        {
                            break;
                        }
                        Err(err) => return Err(format!("read: {err}")),
                    }
                    if response.len() > 64 * 1024 {
                        break;
                    }
                }
                return parse_http_response(&response);
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

fn parse_http_response(raw: &[u8]) -> Result<(u16, String), String> {
    let text = String::from_utf8_lossy(raw);
    let (header, body) = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .unwrap_or((text.as_ref(), ""));
    let status_line = header.lines().next().unwrap_or("");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| format!("invalid HTTP status line: {status_line}"))?;
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

    #[test]
    fn finish_body_appends_current_page_to_prior_stack() {
        let state = json!({
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
        });
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
        let state = json!({
            "page": { "id": "a", "title": "a", "html": "pages/a.html" },
            "page_data": {},
            "stack": []
        });
        let body = wizard_dismiss_finish_body(&state).expect("body");
        let stack = body["stack"].as_array().expect("stack");
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0]["page"]["id"], "a");
        assert_eq!(stack[0]["data"], json!({}));
    }
}
