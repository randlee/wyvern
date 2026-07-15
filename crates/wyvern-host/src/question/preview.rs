//! Server-side option `preview` → sanitized `preview_html` (sprint c.13).
//!
//! Pipeline matches markdown dialogs: `pulldown-cmark` → `ammonia` (REQ-0099).

/// Render an option `preview` field to ammonia-sanitized HTML for dialog JSON.
pub(crate) fn render_preview_html(preview: &str) -> String {
    crate::markdown::render_content_html(preview)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_html_strips_script_and_handlers() {
        let source = "Hi <script>alert(1)</script>\n\n<img src=x onerror=alert(2)>";
        let clean = render_preview_html(source);
        let lower = clean.to_ascii_lowercase();
        assert!(
            !lower.contains("<script") && !lower.contains("alert(1)"),
            "clean={clean}"
        );
        assert!(!lower.contains("onerror"), "clean={clean}");
        assert!(clean.contains("Hi"), "clean={clean}");
    }

    #[test]
    fn preview_html_converts_markdown() {
        let html = render_preview_html("**JSON** output");
        assert!(html.contains("<strong>JSON</strong>"), "html={html}");
    }

    #[test]
    fn preview_html_keeps_pre_fragment() {
        let html = render_preview_html(r#"<pre>{"ok":true}</pre>"#);
        assert!(html.contains("<pre>"), "html={html}");
        assert!(html.contains("ok"), "html={html}");
    }
}
