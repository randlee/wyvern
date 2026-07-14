//! Server-side markdown → sanitized HTML for `GET /api/dialog` (`content_html`).
//!
//! Pipeline: `pulldown-cmark` → `ammonia` (REQ-0099 / sprint c.12).

use std::borrow::Cow;
use std::sync::OnceLock;

use ammonia::Builder;
use pulldown_cmark::{html, Options, Parser};

/// Convert markdown source to an HTML fragment (unsanitized).
pub(crate) fn markdown_to_html(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(source, options);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

/// Render markdown to ammonia-sanitized HTML for dialog JSON `content_html`.
pub(crate) fn render_content_html(source: &str) -> String {
    let html = markdown_to_html(source);
    sanitize_html(&html)
}

/// Sanitize HTML so `content_html` has no script tags or event handlers.
pub(crate) fn sanitize_html(html: &str) -> String {
    content_cleaner().clean(html).to_string()
}

fn content_cleaner() -> &'static Builder<'static> {
    static CLEANER: OnceLock<Builder<'static>> = OnceLock::new();
    CLEANER.get_or_init(|| {
        let mut builder = Builder::default();
        builder.add_url_schemes(&["data"]);
        builder.attribute_filter(|element, attribute, value| {
            let attr = attribute.to_ascii_lowercase();
            if attr.starts_with("on") {
                return None;
            }
            if attr == "href" || attr == "src" {
                let lower = value.trim_start().to_ascii_lowercase();
                if lower.starts_with("javascript:") {
                    return None;
                }
                if lower.starts_with("data:") {
                    if element == "img" && attr == "src" && lower.starts_with("data:image/") {
                        return Some(Cow::Borrowed(value));
                    }
                    return None;
                }
            }
            Some(Cow::Borrowed(value))
        });
        builder
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_renders_heading_and_emphasis() {
        let html = markdown_to_html("**Bold**\n\n# Title");
        assert!(html.contains("<strong>Bold</strong>"), "html={html}");
        assert!(html.contains("<h1>Title</h1>"), "html={html}");
    }

    #[test]
    fn content_html_strips_script_and_handlers() {
        let source = "Hello <script>alert(1)</script>\n\n<img src=x onerror=alert(2)>";
        let clean = render_content_html(source);
        let lower = clean.to_ascii_lowercase();
        assert!(
            !lower.contains("<script") && !lower.contains("alert(1)"),
            "clean={clean}"
        );
        assert!(!lower.contains("onerror"), "clean={clean}");
        assert!(clean.contains("Hello"), "clean={clean}");
    }

    #[test]
    fn content_html_keeps_safe_markup() {
        let clean = render_content_html("# Hello\n\n- a\n- b\n");
        assert!(clean.contains("<h1>Hello</h1>"), "clean={clean}");
        assert!(clean.contains("<li>"), "clean={clean}");
    }
}
