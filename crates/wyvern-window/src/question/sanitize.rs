//! Sanitize question option `preview` HTML fragments (sprint b.8).
//!
//! Authoritative policy from `docs/plans/phase-B/b8-question-preview.md`:
//! - Strip `<script>` / `<style>` tags and their contents
//! - Remove all `on*` event attributes
//! - Reject `javascript:` URLs in `href` / `src`
//! - Allow `data:` URIs for inline images only (`img[src]`)
//! - Keep semantic tags needed for preview fragments

use std::borrow::Cow;
use std::sync::OnceLock;

use ammonia::Builder;

/// Sanitize a preview HTML fragment for safe embedding in the question template.
///
/// Callers should convert markdown previews via [`crate::markdown::markdown_to_html`]
/// first, then pass the HTML through this function.
pub fn sanitize_preview_html(html: &str) -> String {
    preview_cleaner().clean(html).to_string()
}

fn preview_cleaner() -> &'static Builder<'static> {
    static CLEANER: OnceLock<Builder<'static>> = OnceLock::new();
    CLEANER.get_or_init(|| {
        let mut builder = Builder::default();
        // Defaults already strip script/style contents and reject non-whitelisted
        // attributes (including on*). Allow data: then restrict to image srcs.
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

/// Convert a preview field (HTML fragment or markdown) to sanitized HTML.
pub fn render_preview_fragment(preview: &str) -> String {
    let html = crate::markdown::markdown_to_html(preview);
    sanitize_preview_html(&html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_tags_and_contents() {
        let clean = sanitize_preview_html(r#"safe<script>alert(1)</script><pre>x</pre>"#);
        assert!(
            !clean.to_ascii_lowercase().contains("script"),
            "clean={clean}"
        );
        assert!(!clean.contains("alert"), "clean={clean}");
        assert!(
            clean.contains("<pre>x</pre>") || clean.contains("x"),
            "clean={clean}"
        );
    }

    #[test]
    fn strips_style_tags_and_contents() {
        let clean = sanitize_preview_html(r#"ok<style>body{display:none}</style><code>y</code>"#);
        assert!(
            !clean.to_ascii_lowercase().contains("style"),
            "clean={clean}"
        );
        assert!(!clean.contains("display:none"), "clean={clean}");
        assert!(clean.contains("y"), "clean={clean}");
    }

    #[test]
    fn removes_on_event_attributes() {
        let clean = sanitize_preview_html(r#"<p onclick="evil()" onmouseover="x()">hi</p>"#);
        assert!(
            !clean.to_ascii_lowercase().contains("onclick"),
            "clean={clean}"
        );
        assert!(
            !clean.to_ascii_lowercase().contains("onmouseover"),
            "clean={clean}"
        );
        assert!(!clean.contains("evil"), "clean={clean}");
        assert!(clean.contains("hi"), "clean={clean}");
    }

    #[test]
    fn rejects_javascript_urls() {
        let clean = sanitize_preview_html(
            r#"<a href="javascript:alert(1)">x</a><img src="javascript:alert(2)" alt="a">"#,
        );
        assert!(
            !clean.to_ascii_lowercase().contains("javascript:"),
            "clean={clean}"
        );
    }

    #[test]
    fn allows_data_image_uris_only() {
        let ok = sanitize_preview_html(
            r#"<img src="data:image/png;base64,aaaa" alt="ok"><a href="data:text/html,x">bad</a>"#,
        );
        assert!(
            ok.contains("data:image/png;base64,aaaa"),
            "image data URI kept: {ok}"
        );
        assert!(
            !ok.contains("data:text/html"),
            "non-image data URI rejected: {ok}"
        );
    }

    #[test]
    fn keeps_semantic_preview_tags() {
        let clean = sanitize_preview_html(
            r#"<pre><code>{"ok":true}</code></pre><p><strong>T</strong></p>"#,
        );
        assert!(clean.contains("<pre>"), "clean={clean}");
        assert!(clean.contains("<code>"), "clean={clean}");
        assert!(clean.contains("<strong>"), "clean={clean}");
        assert!(
            clean.contains(r#"{"ok":true}"#) || clean.contains("{&quot;ok&quot;:true}"),
            "clean={clean}"
        );
    }

    #[test]
    fn markdown_preview_converted_then_sanitized() {
        let html = render_preview_fragment("**JSON** output");
        assert!(html.contains("<strong>JSON</strong>"), "html={html}");
        assert!(!html.contains("**JSON**"), "html={html}");
    }

    #[test]
    fn html_preview_passthrough_then_sanitized() {
        let html = render_preview_fragment(r#"<pre>{"ok":true}</pre>"#);
        assert!(html.contains("<pre>"), "html={html}");
        assert!(html.contains("ok"), "html={html}");
    }

    #[test]
    fn malicious_markdown_link_sanitized() {
        let html = render_preview_fragment("[x](javascript:alert(1))");
        assert!(
            !html.to_ascii_lowercase().contains("javascript:"),
            "html={html}"
        );
    }
}
