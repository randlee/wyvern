//! Render the chrome HTML shell from the embedded template.

const CHROME_HTML: &str = include_str!("template.html");

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

/// Build chrome HTML with `title` and optional `status` bar content.
///
/// When `status` is [`None`], the status bar element is omitted so it stays
/// hidden. The button bar is always present and `hidden` until Phase B.
pub fn render_chrome_html(title: &str, status: Option<&str>) -> String {
    let safe_title = escape_html_text(title);
    let status_block = status
        .map(|s| {
            let safe_status = escape_html_text(s);
            format!(r#"<div id="status-bar">{safe_status}</div>"#)
        })
        .unwrap_or_default();
    CHROME_HTML
        .replace("{{TITLE}}", &safe_title)
        .replace("{{STATUS_BLOCK}}", &status_block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_includes_title_and_omits_status_when_absent() {
        let html = render_chrome_html("Hello", None);
        assert!(html.contains(r#"<div id="title-bar">Hello</div>"#));
        assert!(!html.contains(r#"id="status-bar""#));
        assert!(html.contains(r#"<div id="button-bar" hidden></div>"#));
        assert!(html.contains("-webkit-app-region: drag"));
        assert!(html.contains("padding-left: 72px"));
    }

    #[test]
    fn render_escapes_html_in_title_and_status() {
        let html = render_chrome_html(r#"<script>"alert"</script>"#, Some(r#"a & b"#));
        assert!(html.contains(
            r#"<div id="title-bar">&lt;script&gt;&quot;alert&quot;&lt;/script&gt;</div>"#
        ));
        assert!(html.contains(r#"<div id="status-bar">a &amp; b</div>"#));
        assert!(!html.contains("<script>"));
    }
}
