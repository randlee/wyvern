//! Render the markdown viewer HTML shell and parse page → host IPC.

use serde_json::{json, Value};
use wyvern_schema::ButtonsPreset;

use crate::markdown::markdown_to_html;
use crate::{DIALOG_MAX_HEIGHT, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH};

const MARKDOWN_HTML: &str = include_str!("template.html");
const MARKDOWN_CSS: &str = include_str!("styles.css");

/// Inputs for [`render_markdown_html`].
#[derive(Debug, Clone)]
pub struct MarkdownRenderInput<'a> {
    /// Window / title-bar text.
    pub title: &'a str,
    /// Markdown source (already loaded from file or inline).
    pub source: &'a str,
    /// Optional status bar text.
    pub status: Option<&'a str>,
    /// Button preset.
    pub buttons: ButtonsPreset,
}

/// Parsed page → host IPC payload for markdown viewer (same as message).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownPageIpc {
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

/// Build markdown viewer HTML with title, optional status, body, and buttons.
pub fn render_markdown_html(input: &MarkdownRenderInput<'_>) -> String {
    let MarkdownRenderInput {
        title,
        source,
        status,
        buttons,
    } = input;

    let display = buttons.display_labels(None);
    let wire = buttons.wire_labels(None);
    debug_assert_eq!(display.len(), wire.len());

    let safe_title = escape_html_text(title);
    let body_html = markdown_to_html(source);

    let status_block = status
        .map(|s| {
            let safe_status = escape_html_text(s);
            format!(r#"<div id="status-bar">{safe_status}</div>"#)
        })
        .unwrap_or_default();

    let mut button_html = String::new();
    for (i, (label, value)) in display.iter().zip(wire.iter()).enumerate() {
        let primary = if i == 0 { " primary" } else { "" };
        button_html.push_str(&format!(
            r#"<button type="button" class="{}" data-wire="{}">{}</button>"#,
            primary.trim(),
            escape_attr(value),
            escape_html_text(label),
        ));
    }

    let context = json!({
        "type": "markdown",
        "title": title,
        "buttons": display,
        "default_button": 0,
    });
    let context_json = context.to_string();

    MARKDOWN_HTML
        .replace("{{STYLESHEET}}", MARKDOWN_CSS)
        .replace("{{TITLE}}", &safe_title)
        .replace("{{STATUS_BLOCK}}", &status_block)
        .replace("{{BODY}}", &body_html)
        .replace("{{BUTTONS}}", &button_html)
        .replace("{{CONTEXT_JSON}}", &context_json)
}

/// Parse a raw IPC body from the page. Malformed / unknown → [`None`].
pub fn parse_markdown_page_ipc(raw: &str) -> Option<MarkdownPageIpc> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let kind = value.get("kind")?.as_str()?;
    match kind {
        "button_pressed" => {
            let label = value.get("label")?.as_str()?.to_string();
            Some(MarkdownPageIpc::ButtonPressed { label })
        }
        "dismissed" => Some(MarkdownPageIpc::Dismissed),
        _ => None,
    }
}

/// Estimate dialog inner size from markdown source (word-wrap heuristic).
///
/// Clamped to Phase B bounds: min 320×200, max 800×600 (REQ-0041). Content
/// scrolls inside the window when the document exceeds the height cap.
pub fn estimate_markdown_window_size(
    source: &str,
    button_count: usize,
    has_status: bool,
) -> (f64, f64) {
    const CHAR_W: f64 = 7.0;
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
    for paragraph in source.split('\n') {
        if paragraph.is_empty() {
            lines += 1;
            continue;
        }
        let len = paragraph.chars().count().max(1);
        lines += len.div_ceil(chars_per_line);
    }
    lines = lines.max(1);

    let longest = source.lines().map(|l| l.chars().count()).max().unwrap_or(0);
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
    fn render_includes_headings_code_tables_lists() {
        let source = r#"# Title

- item one
- item two

```rust
fn main() {}
```

| A | B |
|---|---|
| 1 | 2 |
"#;
        let html = render_markdown_html(&MarkdownRenderInput {
            title: "doc.md",
            source,
            status: None,
            buttons: ButtonsPreset::Ok,
        });
        assert!(html.contains(r#"id="title-bar">doc.md"#));
        assert!(html.contains(r#"id="markdown-body""#));
        assert!(
            html.contains("<h1>Title</h1>") || html.contains("<h1>"),
            "html={html}"
        );
        assert!(
            html.contains("<ul>") || html.contains("<li>"),
            "html={html}"
        );
        assert!(
            html.contains("<pre>") || html.contains("<code>"),
            "html={html}"
        );
        assert!(
            html.contains("<table>") || html.contains("<th>"),
            "html={html}"
        );
        assert!(html.contains(r#"data-wire="ok""#));
        assert!(html.contains(">OK</button>"));
        assert!(!html.contains(r#"id="status-bar""#));
        assert!(
            html.contains("#markdown-body") && html.contains("max-width: 720px"),
            "default stylesheet should be inlined"
        );
        assert!(
            html.contains("overflow-y: auto"),
            "content region must scroll vertically"
        );
        assert!(html.contains(r#"href="styles.css""#));
    }

    #[test]
    fn render_inline_content_includes_status_and_body() {
        let html = render_markdown_html(&MarkdownRenderInput {
            title: "Markdown",
            source: "## Notes\n\n- item one\n- item two",
            status: Some("Read-only"),
            buttons: ButtonsPreset::Ok,
        });
        assert!(html.contains(r#"id="title-bar">Markdown"#));
        assert!(html.contains(r#"id="status-bar">Read-only"#));
        assert!(
            html.contains("<h2>Notes</h2>") || html.contains("<h2>"),
            "html={html}"
        );
        assert!(
            html.contains("<li>") && html.contains("item one"),
            "html={html}"
        );
    }

    #[test]
    fn render_status_and_ok_cancel() {
        let html = render_markdown_html(&MarkdownRenderInput {
            title: "T",
            source: "hello",
            status: Some("Ready"),
            buttons: ButtonsPreset::OkCancel,
        });
        assert!(html.contains(r#"id="status-bar">Ready"#));
        assert!(html.contains(r#"data-wire="ok""#));
        assert!(html.contains(r#"data-wire="cancel""#));
    }

    #[test]
    fn parse_button_pressed() {
        let ipc = parse_markdown_page_ipc(r#"{"kind":"button_pressed","label":"ok"}"#).unwrap();
        assert_eq!(ipc, MarkdownPageIpc::ButtonPressed { label: "ok".into() });
    }

    #[test]
    fn parse_malformed_returns_none() {
        assert!(parse_markdown_page_ipc("not-json").is_none());
        assert!(parse_markdown_page_ipc(r#"{"kind":"unknown"}"#).is_none());
    }

    #[test]
    fn estimate_size_clamped_for_long_docs() {
        let long = "# H\n\n".to_string() + &"paragraph\n\n".repeat(500);
        let (w, h) = estimate_markdown_window_size(&long, 1, true);
        assert!((DIALOG_MIN_WIDTH..=DIALOG_MAX_WIDTH).contains(&w));
        assert!((DIALOG_MIN_HEIGHT..=DIALOG_MAX_HEIGHT).contains(&h));
        assert!((h - DIALOG_MAX_HEIGHT).abs() < f64::EPSILON || h <= DIALOG_MAX_HEIGHT);
    }
}
