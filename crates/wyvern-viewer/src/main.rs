//! Wyvern Browser — thin shell that loads dialog/wizard URLs over HTTP.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]

use std::process::ExitCode;

mod platform;
mod run;

fn main() -> ExitCode {
    match run::run_from_env_and_args(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            emit_structured_error(&err);
            ExitCode::from(1)
        }
    }
}

/// Emit a structured stderr envelope without depending on `wyvern-schema` (boundary).
fn emit_structured_error(err: &run::ViewerError) {
    let (code, slug, recovery) = match err {
        run::ViewerError::Usage { .. } => (
            "VALIDATION_ERROR",
            "validation",
            &[
                "Pass the dialog URL as argv[1] or set WYVERN_DIALOG_URL",
                "Use an http://127.0.0.1:... URL from the host bind",
                "Set WYVERN_VIEWER_ALLOW_NON_LOOPBACK=1 only for intentional non-loopback URLs",
            ][..],
        ),
        run::ViewerError::EventLoop { .. } => (
            "EVENT_LOOP_ERROR",
            "event_loop",
            &[
                "Ensure a display/session is available for the embedded webview",
                "Retry with a fresh WYVERN_DIALOG_URL from a running host",
                "See docs/plans/phase-C/http-viewer-contract.md",
            ][..],
        ),
    };
    let message = json_escape(&err.to_string());
    let cause_json = match err.cause() {
        Some(cause) => format!(",\"cause\":\"{}\"", json_escape(cause)),
        None => String::new(),
    };
    let recovery_json: String = recovery
        .iter()
        .map(|step| format!("\"{}\"", json_escape(step)))
        .collect::<Vec<_>>()
        .join(",");
    eprintln!(
        "{{\"error\":\"{slug}\",\"code\":\"{code}\",\"message\":\"{message}\"{cause_json},\"recovery\":[{recovery_json}],\"docs\":\"docs/plans/phase-C/http-viewer-contract.md\"}}"
    );
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_error_includes_cause_when_present() {
        let err = run::ViewerError::EventLoop {
            message: "event loop failed".into(),
            cause: Some("display unavailable".into()),
        };
        assert_eq!(err.cause(), Some("display unavailable"));
        let message = json_escape(&err.to_string());
        let cause_json = match err.cause() {
            Some(cause) => format!(",\"cause\":\"{}\"", json_escape(cause)),
            None => String::new(),
        };
        let envelope = format!(
            "{{\"error\":\"event_loop\",\"code\":\"EVENT_LOOP_ERROR\",\"message\":\"{message}\"{cause_json},\"recovery\":[],\"docs\":\"docs/plans/phase-C/http-viewer-contract.md\"}}"
        );
        assert!(envelope.contains("\"cause\":\"display unavailable\""));
        assert!(envelope.contains("event loop failed"));
    }

    #[test]
    fn structured_error_omits_cause_when_absent() {
        let err = run::ViewerError::Usage {
            message: "missing dialog URL".into(),
            cause: None,
        };
        assert!(err.cause().is_none());
        let cause_json = match err.cause() {
            Some(cause) => format!(",\"cause\":\"{}\"", json_escape(cause)),
            None => String::new(),
        };
        assert!(cause_json.is_empty());
    }
}
