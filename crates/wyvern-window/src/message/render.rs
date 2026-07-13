//! Render the message HTML shell and parse page → host IPC.

use serde_json::{json, Value};
use wyvern_schema::{ButtonsPreset, MessageLevel};

use crate::error::RunError;
use crate::markdown::markdown_to_html;
use crate::message::media::{resolve_image_src, resolve_level_icon_html};
use crate::{DIALOG_MAX_HEIGHT, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH};

const MESSAGE_HTML: &str = include_str!("template.html");

/// Inputs for [`render_message_html`].
#[derive(Debug, Clone)]
pub struct MessageRenderInput<'a> {
    /// Window / title-bar text.
    pub title: &'a str,
    /// Message body (plain or markdown source).
    pub message: &'a str,
    /// Optional status bar text.
    pub status: Option<&'a str>,
    /// Button preset.
    pub buttons: ButtonsPreset,
    /// Custom labels when `buttons` is [`ButtonsPreset::Custom`].
    pub custom_buttons: Option<&'a [String]>,
    /// 0-based default/focus button index.
    pub default_button: Option<u32>,
    /// Optional semantic level icon.
    pub level: Option<MessageLevel>,
    /// Optional icon override (named, path, or data URI).
    pub icon: Option<&'a str>,
    /// Optional decorative body image.
    pub image: Option<&'a str>,
    /// When true, render `message` as markdown HTML.
    pub markdown: bool,
}

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

/// Build message HTML with title, optional status, body, icons/images, and buttons.
///
/// # Errors
///
/// Returns [`RunError::WindowCreate`] when a path-based `icon` or `image` cannot
/// be loaded from the filesystem.
pub fn render_message_html(input: &MessageRenderInput<'_>) -> Result<String, RunError> {
    let MessageRenderInput {
        title,
        message,
        status,
        buttons,
        custom_buttons,
        default_button,
        level,
        icon,
        image,
        markdown,
    } = input;

    let display = buttons.display_labels(*custom_buttons);
    let wire = buttons.wire_labels(*custom_buttons);
    debug_assert_eq!(display.len(), wire.len());

    let safe_title = escape_html_text(title);
    let body_html = if *markdown {
        markdown_to_html(message)
    } else {
        escape_html_text(message)
    };
    let body_class = if *markdown {
        "markdown-body"
    } else {
        "plain-body"
    };

    let status_block = status
        .map(|s| {
            let safe_status = escape_html_text(s);
            format!(r#"<div id="status-bar">{safe_status}</div>"#)
        })
        .unwrap_or_default();

    let icon_html = resolve_level_icon_html(*level, *icon)?;
    let level_icon_block = match icon_html {
        Some(html) => format!(r#"<div id="level-icon">{html}</div>"#),
        None => String::new(),
    };

    let image_src = resolve_image_src(*image)?;
    let image_block = match image_src {
        Some(src) => format!(
            r#"<img id="decorative-image" src="{}" alt="" />"#,
            escape_attr(&src)
        ),
        None => String::new(),
    };

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
        "markdown": markdown,
    });
    let context_json = context.to_string();

    Ok(MESSAGE_HTML
        .replace("{{TITLE}}", &safe_title)
        .replace("{{STATUS_BLOCK}}", &status_block)
        .replace("{{LEVEL_ICON}}", &level_icon_block)
        .replace("{{BODY_CLASS}}", body_class)
        .replace("{{MESSAGE}}", &body_html)
        .replace("{{DECORATIVE_IMAGE}}", &image_block)
        .replace("{{BUTTONS}}", &button_html)
        .replace("{{CONTEXT_JSON}}", &context_json))
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
    has_icon: bool,
    has_image: bool,
) -> (f64, f64) {
    const CHAR_W: f64 = 7.2;
    const LINE_H: f64 = 18.0;
    const PAD_X: f64 = 48.0;
    const TITLE_H: f64 = 36.0;
    const STATUS_H: f64 = 24.0;
    const BUTTON_BAR_H: f64 = 52.0;
    const CONTENT_PAD_Y: f64 = 28.0;
    const ICON_COL_W: f64 = 48.0;
    const IMAGE_H: f64 = 120.0;
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
    let icon_w = if has_icon { ICON_COL_W } else { 0.0 };
    let text_w = (longest as f64).mul_add(CHAR_W, PAD_X) + icon_w;
    let buttons_w = (button_count as f64).mul_add(96.0, 40.0);
    let width = text_w
        .max(buttons_w)
        .clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);

    let status_h = if has_status { STATUS_H } else { 0.0 };
    let image_h = if has_image { IMAGE_H } else { 0.0 };
    let height =
        (TITLE_H + status_h + CONTENT_PAD_Y + (lines as f64) * LINE_H + image_h + BUTTON_BAR_H)
            .clamp(DIALOG_MIN_HEIGHT, DIALOG_MAX_HEIGHT);

    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_basic(
        title: &str,
        message: &str,
        status: Option<&str>,
        buttons: ButtonsPreset,
        custom: Option<&[String]>,
        default_button: Option<u32>,
    ) -> String {
        render_message_html(&MessageRenderInput {
            title,
            message,
            status,
            buttons,
            custom_buttons: custom,
            default_button,
            level: None,
            icon: None,
            image: None,
            markdown: false,
        })
        .expect("render")
    }

    #[test]
    fn render_includes_buttons_and_message() {
        let html = render_basic(
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
        assert!(!html.contains(r#"id="level-icon""#));
        assert!(!html.contains(r#"id="decorative-image""#));
    }

    #[test]
    fn render_custom_buttons_verbatim() {
        let custom = vec!["Save".into(), "Discard".into()];
        let html = render_basic(
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
    fn render_markdown_body_contains_tags() {
        let html = render_message_html(&MessageRenderInput {
            title: "Disk",
            message: "**Low** on volume",
            status: None,
            buttons: ButtonsPreset::Ok,
            custom_buttons: None,
            default_button: None,
            level: None,
            icon: None,
            image: None,
            markdown: true,
        })
        .expect("render");
        assert!(html.contains("<strong>Low</strong>"), "html={html}");
        assert!(html.contains(r#"id="body" class="markdown-body""#));
        let body_start = html.find(r#"id="body""#).expect("body");
        let body_end = html.find(r#"id="button-bar""#).expect("buttons");
        let body = &html[body_start..body_end];
        assert!(body.contains("<strong>Low</strong>"));
        assert!(!body.contains("**Low**"), "raw markdown leaked into body");
    }

    #[test]
    fn render_level_embeds_placeholder_svg() {
        for level in [
            MessageLevel::Info,
            MessageLevel::Warning,
            MessageLevel::Error,
            MessageLevel::Question,
        ] {
            let html = render_message_html(&MessageRenderInput {
                title: "T",
                message: "M",
                status: None,
                buttons: ButtonsPreset::Ok,
                custom_buttons: None,
                default_button: None,
                level: Some(level),
                icon: None,
                image: None,
                markdown: false,
            })
            .expect("render");
            let marker = format!(r#"data-placeholder-level="{}""#, level.as_str());
            assert!(
                html.contains(&marker),
                "missing {marker} in html for {level:?}"
            );
            assert!(html.contains(r#"id="level-icon""#));
        }
    }

    #[test]
    fn render_icon_wins_over_level() {
        let html = render_message_html(&MessageRenderInput {
            title: "T",
            message: "M",
            status: None,
            buttons: ButtonsPreset::Ok,
            custom_buttons: None,
            default_button: None,
            level: Some(MessageLevel::Info),
            icon: Some("warning"),
            image: None,
            markdown: false,
        })
        .expect("render");
        assert!(html.contains(r#"data-placeholder-level="warning""#));
        assert!(!html.contains(r#"data-placeholder-level="info""#));
    }

    #[test]
    fn render_image_decorative_keeps_button_bar() {
        let html = render_message_html(&MessageRenderInput {
            title: "T",
            message: "Body text",
            status: None,
            buttons: ButtonsPreset::OkCancel,
            custom_buttons: None,
            default_button: None,
            level: Some(MessageLevel::Error),
            icon: None,
            image: Some("data:image/png;base64,AA=="),
            markdown: false,
        })
        .expect("render");
        assert!(html.contains(r#"id="decorative-image""#));
        assert!(html.contains(r#"src="data:image/png;base64,AA==""#));
        assert!(html.contains(r#"id="button-bar""#));
        assert!(html.contains(r#"data-wire="ok""#));
        assert!(html.contains(r#"data-wire="cancel""#));
    }

    #[test]
    fn render_combo_level_markdown_image_icon() {
        let html = render_message_html(&MessageRenderInput {
            title: "T",
            message: "## Hello\n\n**world**",
            status: Some("status"),
            buttons: ButtonsPreset::YesNo,
            custom_buttons: None,
            default_button: Some(1),
            level: Some(MessageLevel::Question),
            icon: Some("error"),
            image: Some("data:image/gif;base64,R0lGODlhAQABAAAAACw="),
            markdown: true,
        })
        .expect("render");
        assert!(html.contains(r#"data-placeholder-level="error""#));
        assert!(html.contains("<h2>Hello</h2>") || html.contains("<strong>world</strong>"));
        assert!(html.contains(r#"id="decorative-image""#));
        assert!(html.contains(r#"id="button-bar""#));
        assert!(html.contains(r#"id="status-bar">status"#));
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
        let (w, h) = estimate_message_window_size("Hi", 1, false, false, false);
        assert!((DIALOG_MIN_WIDTH..=DIALOG_MAX_WIDTH).contains(&w));
        assert!((DIALOG_MIN_HEIGHT..=DIALOG_MAX_HEIGHT).contains(&h));

        let long = "x".repeat(10_000);
        let (w2, h2) = estimate_message_window_size(&long, 3, true, true, true);
        assert!((w2 - DIALOG_MAX_WIDTH).abs() < f64::EPSILON || w2 <= DIALOG_MAX_WIDTH);
        assert!(h2 <= DIALOG_MAX_HEIGHT);
        assert!(w2 >= DIALOG_MIN_WIDTH);
        assert!(h2 >= DIALOG_MIN_HEIGHT);
    }
}
