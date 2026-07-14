//! Chrome window-control IPC (Phase C chrome-ipc-contract.md).

/// Page → host messages for HTML window chrome (Win/Linux).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChromeIpc {
    /// User clicked HTML close (`{"kind":"window_close"}`).
    WindowClose,
    /// User clicked HTML minimize (`{"kind":"window_minimize"}`).
    WindowMinimize,
}

/// Parse a raw IPC body as chrome window-control IPC.
///
/// Returns [`None`] for malformed JSON or unknown / non-chrome kinds so callers
/// can fall through to dialog IPC or the fail-safe dismiss path.
pub(crate) fn parse_chrome_ipc(raw: &str) -> Option<ChromeIpc> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    match v.get("kind")?.as_str()? {
        "window_close" => Some(ChromeIpc::WindowClose),
        "window_minimize" => Some(ChromeIpc::WindowMinimize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_window_close() {
        assert_eq!(
            parse_chrome_ipc(r#"{"kind":"window_close"}"#),
            Some(ChromeIpc::WindowClose)
        );
    }

    #[test]
    fn parse_window_minimize() {
        assert_eq!(
            parse_chrome_ipc(r#"{"kind":"window_minimize"}"#),
            Some(ChromeIpc::WindowMinimize)
        );
    }

    #[test]
    fn parse_unknown_or_malformed_returns_none() {
        assert!(parse_chrome_ipc("not-json").is_none());
        assert!(parse_chrome_ipc(r#"{"kind":"button_pressed","label":"ok"}"#).is_none());
        assert!(parse_chrome_ipc(r#"{"kind":"dismissed"}"#).is_none());
        assert!(parse_chrome_ipc(r#"{}"#).is_none());
    }
}
