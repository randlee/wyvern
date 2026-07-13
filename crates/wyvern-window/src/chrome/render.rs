//! Render the chrome HTML shell from the embedded template.

const CHROME_HTML: &str = include_str!("template.html");

/// Build chrome HTML with `title` and optional `status` bar content.
///
/// When `status` is [`None`], the status bar element is omitted so it stays
/// hidden. The button bar is always present and `hidden` until Phase B.
pub fn render_chrome_html(title: &str, status: Option<&str>) -> String {
    let status_block = status
        .map(|s| format!(r#"<div id="status-bar">{s}</div>"#))
        .unwrap_or_default();
    CHROME_HTML
        .replace("{{TITLE}}", title)
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
    fn render_includes_status_when_present() {
        let html = render_chrome_html("T", Some("Ready"));
        assert!(html.contains(r#"<div id="status-bar">Ready</div>"#));
    }
}
