//! Shared markdown → HTML converter for Phase B dialogs.
//!
//! Owned by `wyvern-window` (not `wyvern-schema`). Reused by message body,
//! markdown dialog (b.5/b.6), and question option previews (b.8).

use pulldown_cmark::{html, Options, Parser};

/// Convert markdown source to an HTML fragment.
///
/// Enables commonmark + tables + strikethrough + tasklists + footnotes.
/// Output is unsanitized HTML suitable for embedding in controlled dialog
/// templates (caller supplies the surrounding chrome).
pub fn markdown_to_html(source: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_renders_emphasis_and_heading() {
        let html = markdown_to_html("**Low** on volume\n\n# Title");
        assert!(html.contains("<strong>Low</strong>"), "html={html}");
        assert!(html.contains("<h1>Title</h1>"), "html={html}");
    }

    #[test]
    fn markdown_renders_link() {
        let html = markdown_to_html("[docs](https://example.com)");
        assert!(
            html.contains("<a href=\"https://example.com\">docs</a>"),
            "html={html}"
        );
    }

    #[test]
    fn markdown_plain_paragraph() {
        let html = markdown_to_html("hello");
        assert!(html.contains("<p>hello</p>"), "html={html}");
    }
}
