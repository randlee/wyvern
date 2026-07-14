//! Render the chrome HTML shell from the embedded template.

use super::PlatformChrome;

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

/// Inline style fragment for `#title-bar` (macOS safe zone vs Win/Linux).
pub(crate) fn title_bar_style(chrome: &PlatformChrome) -> &'static str {
    if chrome.macos_safe_zone {
        "padding-left: 72px;"
    } else {
        ""
    }
}

/// HTML for `#window-controls`, or empty when controls are not shown.
pub(crate) fn window_controls_block(chrome: &PlatformChrome) -> String {
    if !chrome.show_window_controls {
        return String::new();
    }
    render_window_controls(chrome.show_minimize)
}

fn render_window_controls(show_minimize: bool) -> String {
    let minimize = if show_minimize {
        r#"<button id="btn-minimize" data-action="minimize" aria-label="Minimize">—</button>"#
    } else {
        ""
    };
    format!(
        r#"<div id="window-controls" class="no-drag">{minimize}<button id="btn-close" data-action="close" aria-label="Close">×</button></div>"#
    )
}

/// Build chrome HTML with `title`, optional `status` bar, and platform chrome.
///
/// When `status` is [`None`], the status bar element is omitted so it stays
/// hidden. The button bar is always present and `hidden` until Phase B.
pub fn render_chrome_html(title: &str, status: Option<&str>, chrome: PlatformChrome) -> String {
    let safe_title = escape_html_text(title);
    let status_block = status
        .map(|s| {
            let safe_status = escape_html_text(s);
            format!(r#"<div id="status-bar">{safe_status}</div>"#)
        })
        .unwrap_or_default();
    CHROME_HTML
        .replace("{{TITLE}}", &safe_title)
        .replace("{{TITLE_BAR_STYLE}}", title_bar_style(&chrome))
        .replace("{{WINDOW_CONTROLS_BLOCK}}", &window_controls_block(&chrome))
        .replace("{{STATUS_BLOCK}}", &status_block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chrome::platform::{platform_chrome_for, CommandKind};

    #[test]
    fn render_includes_title_and_omits_status_when_absent() {
        let chrome = platform_chrome_for(CommandKind::Chrome);
        let html = render_chrome_html("Hello", None, chrome);
        assert!(html.contains(r#"id="title-text">Hello</span>"#));
        assert!(!html.contains(r#"id="status-bar""#));
        assert!(html.contains(r#"<div id="button-bar" hidden></div>"#));
        assert!(html.contains("-webkit-app-region: drag"));
        assert!(html.contains("window.ipc.postMessage"));
        assert!(html.contains(r#"getElementById("window-controls")"#));

        #[cfg(target_os = "macos")]
        {
            assert!(html.contains("padding-left: 72px"));
            assert!(!html.contains(r#"id="window-controls""#));
            assert!(!html.contains(r#"data-action="close""#));
        }
        #[cfg(not(target_os = "macos"))]
        {
            // Title-bar padding comes only via TITLE_BAR_STYLE (empty on Win/Linux).
            assert!(!html.contains("padding-left: 72px"));
            assert!(html.contains(r#"id="window-controls""#));
            assert!(html.contains(r#"data-action="close""#));
            assert!(html.contains(r#"data-action="minimize""#));
            assert!(html.contains(r#"id="btn-minimize""#));
        }
    }

    #[test]
    fn render_escapes_html_in_title_and_status() {
        let chrome = platform_chrome_for(CommandKind::Chrome);
        let html = render_chrome_html(r#"<script>"alert"</script>"#, Some(r#"a & b"#), chrome);
        assert!(html
            .contains(r#"id="title-text">&lt;script&gt;&quot;alert&quot;&lt;/script&gt;</span>"#));
        assert!(html.contains(r#"<div id="status-bar">a &amp; b</div>"#));
        // Escaped title must not introduce a raw script tag in the title bar.
        let title_start = html.find(r#"id="title-text""#).expect("title-text");
        let title_end = html.find("</span>").expect("title close");
        let title_slice = &html[title_start..title_end];
        assert!(!title_slice.contains("<script>"));
    }

    #[test]
    fn modal_controls_omit_minimize_on_non_macos() {
        let chrome = platform_chrome_for(CommandKind::Message);
        let block = window_controls_block(&chrome);
        #[cfg(target_os = "macos")]
        {
            assert!(block.is_empty());
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(block.contains(r#"id="window-controls""#));
            assert!(block.contains(r#"data-action="close""#));
            assert!(!block.contains(r#"data-action="minimize""#));
            assert!(!block.contains(r#"id="btn-minimize""#));
        }
    }
}
