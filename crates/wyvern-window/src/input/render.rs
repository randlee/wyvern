//! Render the input HTML shell and parse page → host IPC.

use serde_json::{json, Value};
use wyvern_schema::{ButtonsPreset, InputMode};

use crate::chrome::{platform_chrome_for, title_bar_style, window_controls_block, CommandKind};
use crate::error::RunError;
use crate::markdown::markdown_to_html;
use crate::message::media::resolve_level_icon_html;
use crate::{DIALOG_MAX_HEIGHT, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH};

const INPUT_HTML: &str = include_str!("template.html");

/// Inputs for [`render_input_html`].
#[derive(Debug, Clone)]
pub struct InputRenderInput<'a> {
    /// Window / title-bar text.
    pub title: &'a str,
    /// Prompt text (plain or markdown source).
    pub message: &'a str,
    /// Optional status bar text.
    pub status: Option<&'a str>,
    /// Optional icon override (named, path, or data URI).
    pub icon: Option<&'a str>,
    /// When true, render `message` as markdown HTML.
    pub markdown: bool,
    /// When true, render a multiline textarea (text mode only).
    pub multiline: bool,
    /// Optional placeholder hint (text mode only).
    pub placeholder: Option<&'a str>,
    /// Optional pre-filled value (text mode only).
    pub default: Option<&'a str>,
    /// Input mode — file/folder omit the text field (picker-on-OK).
    pub mode: InputMode,
    /// Button preset (defaults to ok_cancel at schema layer).
    pub buttons: ButtonsPreset,
}

/// Parsed page → host IPC payload for input dialogs (ipc-dialog-contract.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputPageIpc {
    /// User confirmed or cancelled; `value` omitted on cancel.
    InputSubmitted {
        button: String,
        value: Option<String>,
    },
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

/// Build input HTML with prompt, text field / textarea, and buttons.
///
/// # Errors
///
/// Returns [`RunError::WindowCreate`] when a path-based `icon` cannot be loaded.
pub fn render_input_html(input: &InputRenderInput<'_>) -> Result<String, RunError> {
    let InputRenderInput {
        title,
        message,
        status,
        icon,
        markdown,
        multiline,
        placeholder,
        default,
        mode,
        buttons,
    } = input;

    let display = buttons.display_labels(None);
    let wire = buttons.wire_labels(None);
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

    let icon_html = resolve_level_icon_html(None, *icon)?;
    let level_icon_block = match icon_html {
        Some(html) => format!(r#"<div id="level-icon">{html}</div>"#),
        None => String::new(),
    };

    let picker_mode = matches!(mode, InputMode::File | InputMode::Folder);
    let input_control = if picker_mode {
        // File/folder: message + button bar only (picker opens on OK in host).
        String::new()
    } else {
        let placeholder_attr = placeholder
            .map(|p| format!(r#" placeholder="{}""#, escape_attr(p)))
            .unwrap_or_default();
        let default_value = default.unwrap_or("");
        if *multiline {
            format!(
                r#"<textarea id="input-field" class="multi-line"{placeholder_attr}>{value}</textarea>"#,
                placeholder_attr = placeholder_attr,
                value = escape_html_text(default_value),
            )
        } else {
            format!(
                r#"<input id="input-field" class="single-line" type="text"{placeholder_attr} value="{value}" />"#,
                placeholder_attr = placeholder_attr,
                value = escape_attr(default_value),
            )
        }
    };

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
        "type": "input",
        "title": title,
        "message": message,
        "buttons": display,
        "multiline": multiline,
        "markdown": markdown,
        "mode": mode.as_str(),
        "picker": picker_mode,
    });
    let context_json = context.to_string();
    let chrome = platform_chrome_for(CommandKind::Input);

    Ok(INPUT_HTML
        .replace("{{TITLE}}", &safe_title)
        .replace("{{TITLE_BAR_STYLE}}", title_bar_style(&chrome))
        .replace("{{WINDOW_CONTROLS_BLOCK}}", &window_controls_block(&chrome))
        .replace("{{STATUS_BLOCK}}", &status_block)
        .replace("{{LEVEL_ICON}}", &level_icon_block)
        .replace("{{BODY_CLASS}}", body_class)
        .replace("{{MESSAGE}}", &body_html)
        .replace("{{INPUT_CONTROL}}", &input_control)
        .replace("{{BUTTONS}}", &button_html)
        .replace("{{CONTEXT_JSON}}", &context_json))
}

/// Parse a raw IPC body from the input page. Malformed / unknown → [`None`].
pub fn parse_input_page_ipc(raw: &str) -> Option<InputPageIpc> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let kind = value.get("kind")?.as_str()?;
    match kind {
        "input_submitted" => {
            let button = value.get("button")?.as_str()?.to_string();
            let submitted_value = value
                .get("value")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            Some(InputPageIpc::InputSubmitted {
                button,
                value: submitted_value,
            })
        }
        "dismissed" => Some(InputPageIpc::Dismissed),
        _ => None,
    }
}

/// Estimate dialog inner size from prompt text and field height (REQ-0041).
pub fn estimate_input_window_size(
    message: &str,
    button_count: usize,
    has_status: bool,
    has_icon: bool,
    multiline: bool,
    picker_mode: bool,
) -> (f64, f64) {
    const CHAR_W: f64 = 7.2;
    const LINE_H: f64 = 18.0;
    const PAD_X: f64 = 48.0;
    const TITLE_H: f64 = 36.0;
    const STATUS_H: f64 = 24.0;
    const BUTTON_BAR_H: f64 = 52.0;
    const CONTENT_PAD_Y: f64 = 28.0;
    const ICON_COL_W: f64 = 48.0;
    const SINGLE_FIELD_H: f64 = 40.0;
    const MULTI_FIELD_H: f64 = 100.0;
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
    let field_h = if picker_mode {
        0.0
    } else if multiline {
        MULTI_FIELD_H
    } else {
        SINGLE_FIELD_H
    };
    let height =
        (TITLE_H + status_h + CONTENT_PAD_Y + (lines as f64) * LINE_H + field_h + BUTTON_BAR_H)
            .clamp(DIALOG_MIN_HEIGHT, DIALOG_MAX_HEIGHT);

    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_basic(
        title: &str,
        message: &str,
        multiline: bool,
        placeholder: Option<&str>,
        default: Option<&str>,
    ) -> String {
        render_input_html(&InputRenderInput {
            title,
            message,
            status: None,
            icon: None,
            markdown: false,
            multiline,
            placeholder,
            default,
            mode: InputMode::Text,
            buttons: ButtonsPreset::OkCancel,
        })
        .expect("render")
    }

    #[test]
    fn render_single_line_includes_input_and_buttons() {
        let html = render_basic("Name", "Enter name", false, Some("hint"), Some("Ada"));
        assert!(html.contains(r#"id="title-text">Name</span>"#));
        assert!(html.contains("Enter name"));
        assert!(html.contains(r#"id="input-field""#));
        assert!(html.contains(r#"class="single-line""#));
        assert!(html.contains(r#"placeholder="hint""#));
        assert!(html.contains(r#"value="Ada""#));
        assert!(!html.contains("<textarea"));
        assert!(html.contains(r#"data-wire="ok""#));
        assert!(html.contains(r#"data-wire="cancel""#));
    }

    #[test]
    fn render_multiline_uses_textarea() {
        let html = render_basic("T", "M", true, None, Some("prefill"));
        assert!(html.contains("<textarea"));
        assert!(html.contains(r#"class="multi-line""#));
        assert!(html.contains(">prefill</textarea>"));
        assert!(!html.contains(r#"type="text""#));
    }

    #[test]
    fn render_file_mode_omits_text_field() {
        let html = render_input_html(&InputRenderInput {
            title: "Open",
            message: "Choose a file",
            status: None,
            icon: None,
            markdown: false,
            multiline: false,
            placeholder: None,
            default: None,
            mode: InputMode::File,
            buttons: ButtonsPreset::OkCancel,
        })
        .expect("render");
        assert!(html.contains("Choose a file"));
        assert!(!html.contains(r#"id="input-field""#));
        assert!(!html.contains("<textarea"));
        assert!(html.contains(r#"data-wire="ok""#));
        assert!(html.contains(r#""picker":true"#));
        assert!(html.contains(r#""mode":"file""#));
    }

    #[test]
    fn render_folder_mode_omits_text_field() {
        let html = render_input_html(&InputRenderInput {
            title: "Open",
            message: "Choose a folder",
            status: None,
            icon: None,
            markdown: false,
            multiline: false,
            placeholder: None,
            default: None,
            mode: InputMode::Folder,
            buttons: ButtonsPreset::OkCancel,
        })
        .expect("render");
        assert!(!html.contains(r#"id="input-field""#));
        assert!(html.contains(r#""mode":"folder""#));
    }

    #[test]
    fn render_markdown_prompt() {
        let html = render_input_html(&InputRenderInput {
            title: "T",
            message: "**Bold** prompt",
            status: None,
            icon: None,
            markdown: true,
            multiline: false,
            placeholder: None,
            default: None,
            mode: InputMode::Text,
            buttons: ButtonsPreset::OkCancel,
        })
        .expect("render");
        assert!(html.contains("<strong>Bold</strong>"));
        assert!(html.contains(r#"class="markdown-body""#));
    }

    #[test]
    fn parse_input_submitted_with_value() {
        let ipc =
            parse_input_page_ipc(r#"{"kind":"input_submitted","button":"ok","value":"hello"}"#)
                .unwrap();
        assert_eq!(
            ipc,
            InputPageIpc::InputSubmitted {
                button: "ok".into(),
                value: Some("hello".into()),
            }
        );
    }

    #[test]
    fn parse_input_submitted_ok_without_value() {
        let ipc = parse_input_page_ipc(r#"{"kind":"input_submitted","button":"ok"}"#).unwrap();
        assert_eq!(
            ipc,
            InputPageIpc::InputSubmitted {
                button: "ok".into(),
                value: None,
            }
        );
    }

    #[test]
    fn parse_input_submitted_cancel_omits_value() {
        let ipc = parse_input_page_ipc(r#"{"kind":"input_submitted","button":"cancel"}"#).unwrap();
        assert_eq!(
            ipc,
            InputPageIpc::InputSubmitted {
                button: "cancel".into(),
                value: None,
            }
        );
    }

    #[test]
    fn parse_malformed_returns_none() {
        assert!(parse_input_page_ipc("not-json").is_none());
        assert!(parse_input_page_ipc(r#"{"kind":"button_pressed","label":"ok"}"#).is_none());
        assert!(parse_input_page_ipc(r#"{"kind":"input_submitted"}"#).is_none());
    }

    #[test]
    fn input_submitted_maps_to_input_result_wire() {
        use wyvern_schema::{ButtonLabel, CommandResult, InputResult, InputValue};

        let ipc =
            parse_input_page_ipc(r#"{"kind":"input_submitted","button":"ok","value":"user text"}"#)
                .unwrap();
        let InputPageIpc::InputSubmitted { button, value } = ipc else {
            panic!("expected InputSubmitted");
        };
        let result = CommandResult::Input(InputResult {
            button: ButtonLabel::new(button),
            input: value.map(InputValue::Text),
        });
        let wire = serde_json::to_string(&result).expect("serialize");
        assert_eq!(wire, r#"{"button":"ok","input":"user text"}"#);

        let cancel =
            parse_input_page_ipc(r#"{"kind":"input_submitted","button":"cancel"}"#).unwrap();
        let InputPageIpc::InputSubmitted { button, value } = cancel else {
            panic!("expected InputSubmitted");
        };
        let result = CommandResult::Input(InputResult {
            button: ButtonLabel::new(button),
            input: value.map(InputValue::Text),
        });
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"button":"cancel"}"#
        );
    }

    #[test]
    fn estimate_size_clamped_to_bounds() {
        let (w, h) = estimate_input_window_size("Hi", 2, false, false, false, false);
        assert!((DIALOG_MIN_WIDTH..=DIALOG_MAX_WIDTH).contains(&w));
        assert!((DIALOG_MIN_HEIGHT..=DIALOG_MAX_HEIGHT).contains(&h));

        let (w2, h2) = estimate_input_window_size("Hi", 2, true, true, true, false);
        assert!(w2 >= DIALOG_MIN_WIDTH);
        assert!(h2 >= DIALOG_MIN_HEIGHT);
        assert!(h2 <= DIALOG_MAX_HEIGHT);

        let (_, h3) = estimate_input_window_size("Hi", 2, false, false, false, true);
        assert!(h3 >= DIALOG_MIN_HEIGHT);
        assert!(h3 <= h2);
    }
}
