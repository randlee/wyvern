//! Render the message HTML shell and parse page → host IPC.

use serde_json::{json, Value};
use wyvern_schema::ButtonsPreset;

use crate::{DIALOG_MAX_HEIGHT, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH};

const MESSAGE_HTML: &str = include_str!("template.html");

/// Parsed page → host IPC payload (ipc-dialog-contract.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageIpc {
    /// User clicked a button; `label` is the stdout wire value.
    ButtonPressed { label: String },
    /// Explicit dismiss from page (rare; OS close is handled by winit).
    Dismissed,
}

/// Escape text for safe insertion into HTML element bodies.
fn escape_html_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape a string for embedding inside a double-quoted HTML attribute.
fn escape_attr(s: &str) -> String {
    escape_html_text(s)
}

/// Build message HTML with title, optional status, body text, and buttons.
pub fn render_message_html(
    title: &str,
    message: &str,
    status: Option<&str>,
    buttons: ButtonsPreset,
    custom_buttons: Option<&[String]>,
    default_button: Option<u32>,
) -> String {
    let display = buttons.display_labels(custom_buttons);
    let wire = buttons.wire_labels(custom_buttons);
    debug_assert_eq!(display.len(), wire.len());

    let safe_title = escape_html_text(title);
    let safe_message = escape_html_text(message);
    let status_block = status
        .map(|s| {
            let safe_status = escape_html_text(s);
            format!(r#"<div id="status-bar">{safe_status}</div>"#)
        })
        .unwrap_or_default();

    let mut button_html = String::new();
    for (i, (label, value)) in display.iter().zip(wire.iter()).enumerate() {
        let primary = if default_button.map(|d| d as usize) == Some(i)
            || (default_button.is_none() && i == 0)
        {
            " primary"
        } else {
            ""
        };
        button_html.push_str(&format!(
            r#"<button type="button" class="{}" data-wire="{}">{}</button>"#,
            primary.trim(),
            escape_attr(value),
            escape_html_text(label),
        ));
    }

    let context = json!({
        "type": "message",
        "title": title,
        "message": message,
        "buttons": display,
        "default_button": default_button.unwrap_or(0),
    });
    let context_json = context.to_string();

    MESSAGE_HTML
        .replace("{{TITLE}}", &safe_title)
        .replace("{{STATUS_BLOCK}}", &status_block)
        .replace("{{MESSAGE}}", &safe_message)
        .replace("{{BUTTONS}}", &button_html)
        .replace("{{CONTEXT_JSON}}", &context_json)
}

/// Parse a raw IPC body from the page. Malformed / unknown → [`None`].
pub fn parse_page_ipc(raw: &str) -> Option<PageIpc> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let kind = value.get("kind")?.as_str()?;
    match kind {
        "button_pressed" => {
            let label = value.get("label")?.as_str()?.to_string();
            Some(PageIpc::ButtonPressed { label })
        }
        "dismissed" => Some(PageIpc::Dismissed),
        _ => None,
    }
}

/// Estimate dialog inner size from message text (word-wrap heuristic).
///
/// Clamped to Phase B bounds: min 320×200, max 800×600 (REQ-0041).
pub fn estimate_message_window_size(
    message: &str,
    button_count: usize,
    has_status: bool,
) -> (f64, f64) {
    const CHAR_W: f64 = 7.2;
    const LINE_H: f64 = 18.0;
    const PAD_X: f64 = 48.0;
    const TITLE_H: f64 = 36.0;
    const STATUS_H: f64 = 24.0;
    const BUTTON_BAR_H: f64 = 52.0;
    const CONTENT_PAD_Y: f64 = 28.0;
    const CONTENT_MAX_W: f64 = DIALOG_MAX_WIDTH - PAD_X;

    let wrap_width = CONTENT_MAX_W.max(DIALOG_MIN_WIDTH - PAD_X);
    let chars_per_line = ((wrap_width / CHAR_W).floor() as usize).max(1);

    let mut lines = 0usize;
    for paragraph in message.split('\n') {
        if paragraph.is_empty() {
            lines += 1;
            continue;
        }
        let len = paragraph.chars().count().max(1);
        lines += len.div_ceil(chars_per_line);
    }
    lines = lines.max(1);

    let longest = message
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    let text_w = (longest as f64).mul_add(CHAR_W, PAD_X);
    let buttons_w = (button_count as f64).mul_add(96.0, 40.0);
    let width = text_w
        .max(buttons_w)
        .clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);

    let status_h = if has_status { STATUS_H } else { 0.0 };
    let height = (TITLE_H + status_h + CONTENT_PAD_Y + (lines as f64) * LINE_H + BUTTON_BAR_H)
        .clamp(DIALOG_MIN_HEIGHT, DIALOG_MAX_HEIGHT);

    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_includes_buttons_and_message() {
        let html = render_message_html(
            "Title",
            "Hello body",
            None,
            ButtonsPreset::OkCancel,
            None,
            Some(0),
        );
        assert!(html.contains(r#"id="title-bar">Title"#));
        assert!(html.contains("Hello body"));
        assert!(!html.contains(r#"id="status-bar""#));
        assert!(html.contains(r#"data-wire="ok""#));
        assert!(html.contains(r#"data-wire="cancel""#));
        assert!(html.contains(">OK</button>"));
        assert!(html.contains(">Cancel</button>"));
        assert!(!html.contains(r#"id="button-bar" hidden"#));
    }

    #[test]
    fn render_custom_buttons_verbatim() {
        let custom = vec!["Save".into(), "Discard".into()];
        let html = render_message_html(
            "T",
            "M",
            Some("Ready"),
            ButtonsPreset::Custom,
            Some(&custom),
            Some(1),
        );
        assert!(html.contains(r#"data-wire="Save""#));
        assert!(html.contains(r#"data-wire="Discard""#));
        assert!(html.contains(r#"id="status-bar">Ready"#));
    }

    #[test]
    fn parse_button_pressed() {
        let ipc = parse_page_ipc(r#"{"kind":"button_pressed","label":"ok"}"#).unwrap();
        assert_eq!(ipc, PageIpc::ButtonPressed { label: "ok".into() });
    }

    #[test]
    fn parse_malformed_returns_none() {
        assert!(parse_page_ipc("not-json").is_none());
        assert!(parse_page_ipc(r#"{"kind":"unknown"}"#).is_none());
        assert!(parse_page_ipc(r#"{"kind":"button_pressed"}"#).is_none());
    }

    #[test]
    fn estimate_size_clamped_to_bounds() {
        let (w, h) = estimate_message_window_size("Hi", 1, false);
        assert!((DIALOG_MIN_WIDTH..=DIALOG_MAX_WIDTH).contains(&w));
        assert!((DIALOG_MIN_HEIGHT..=DIALOG_MAX_HEIGHT).contains(&h));

        let long = "x".repeat(10_000);
        let (w2, h2) = estimate_message_window_size(&long, 3, true);
        assert!((w2 - DIALOG_MAX_WIDTH).abs() < f64::EPSILON || w2 <= DIALOG_MAX_WIDTH);
        assert!(h2 <= DIALOG_MAX_HEIGHT);
        assert!(w2 >= DIALOG_MIN_WIDTH);
        assert!(h2 >= DIALOG_MIN_HEIGHT);
    }
}
