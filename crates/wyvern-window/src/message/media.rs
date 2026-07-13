//! Resolve message `icon` / `image` specs and level production SVGs.

use std::fs;
use std::path::Path;

use wyvern_schema::icons as schema_icons;
use wyvern_schema::MessageLevel;

use crate::error::RunError;
use crate::icons;

/// Phase B placeholder SVGs — retained for regression tests only (c.1+).
const PLACEHOLDER_INFO: &str = include_str!("../../assets/icons/placeholder/info.svg");
const PLACEHOLDER_WARNING: &str = include_str!("../../assets/icons/placeholder/warning.svg");
const PLACEHOLDER_ERROR: &str = include_str!("../../assets/icons/placeholder/error.svg");
const PLACEHOLDER_QUESTION: &str = include_str!("../../assets/icons/placeholder/question.svg");

/// HTML snippet for the `#level-icon` slot (inline SVG or `<img>`).
pub type IconHtml = String;

/// `src` attribute value for `#decorative-image`.
pub type ImageSrc = String;

/// Production SVG markup for a message level (variant 1).
pub fn icon_html_for_level(level: MessageLevel) -> IconHtml {
    let role = level.as_str();
    icons::svg_markup(role, 1)
        .expect("c.1 bundles variant 1 for every level role")
        .to_string()
}

/// Phase B placeholder SVG for `level` — test / regression use only.
pub fn placeholder_svg_for_level(level: MessageLevel) -> &'static str {
    match level {
        MessageLevel::Info => PLACEHOLDER_INFO,
        MessageLevel::Warning => PLACEHOLDER_WARNING,
        MessageLevel::Error => PLACEHOLDER_ERROR,
        MessageLevel::Question => PLACEHOLDER_QUESTION,
    }
}

/// Resolve the `#level-icon` winner: `icon` overrides `level` when both set.
///
/// # Errors
///
/// Returns [`RunError::WindowCreate`] when a path-based icon cannot be read.
pub fn resolve_level_icon_html(
    level: Option<MessageLevel>,
    icon: Option<&str>,
) -> Result<Option<IconHtml>, RunError> {
    if let Some(spec) = icon {
        return Ok(Some(resolve_media_as_icon_html(spec)?));
    }
    if let Some(level) = level {
        return Ok(Some(icon_html_for_level(level)));
    }
    Ok(None)
}

/// Resolve decorative body image `src` (REQ-0032).
///
/// # Errors
///
/// Returns [`RunError::WindowCreate`] when a path-based image cannot be read.
pub fn resolve_image_src(image: Option<&str>) -> Result<Option<ImageSrc>, RunError> {
    match image {
        None => Ok(None),
        Some(spec) => Ok(Some(resolve_media_as_src(spec)?)),
    }
}

fn resolve_media_as_icon_html(spec: &str) -> Result<IconHtml, RunError> {
    if spec.starts_with("data:") {
        return Ok(format!(
            r#"<img class="resolved-icon" src="{}" alt="" />"#,
            escape_attr(spec)
        ));
    }
    if looks_like_path(spec) {
        let src = load_path_as_data_uri(spec)?;
        return Ok(format!(
            r#"<img class="resolved-icon" src="{}" alt="" />"#,
            escape_attr(&src)
        ));
    }
    // Named icon (optional `:variant`). c.1 always renders variant 1; c.2 selects index.
    Ok(named_role_svg_markup(spec).to_string())
}

fn resolve_media_as_src(spec: &str) -> Result<ImageSrc, RunError> {
    if spec.starts_with("data:") {
        return Ok(spec.to_string());
    }
    if looks_like_path(spec) {
        return load_path_as_data_uri(spec);
    }
    // Named → embed production SVG as data URI for <img>.
    Ok(svg_to_data_uri(named_role_svg_markup(spec)))
}

/// Resolve a named role to production SVG markup (variant 1 in c.1).
///
/// Unknown names fall back to `info` until c.2 validation errors land.
fn named_role_svg_markup(spec: &str) -> &'static str {
    let base = match schema_icons::parse_icon_spec(spec) {
        Ok((base, _)) => base,
        Err(()) => named_icon_base(spec).to_string(),
    };
    let role = if schema_icons::variant_count(&base) > 0 {
        base.as_str()
    } else {
        "info"
    };
    icons::svg_markup(role, 1).expect("c.1 bundles variant 1 for every catalog role")
}

fn named_icon_base(spec: &str) -> &str {
    spec.split_once(':').map(|(base, _)| base).unwrap_or(spec)
}

fn looks_like_path(spec: &str) -> bool {
    spec.contains('/')
        || spec.contains('\\')
        || spec.starts_with('.')
        || Path::new(spec).extension().is_some()
}

fn load_path_as_data_uri(path: &str) -> Result<String, RunError> {
    let bytes = fs::read(path).map_err(|err| RunError::WindowCreate {
        message: format!("failed to load media path '{path}': {err}"),
    })?;
    let mime = mime_for_path(path);
    if mime == "image/svg+xml" {
        let text = String::from_utf8_lossy(&bytes);
        return Ok(svg_to_data_uri(&text));
    }
    Ok(format!("data:{mime};base64,{}", base64_encode(&bytes)))
}

fn mime_for_path(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

fn svg_to_data_uri(svg: &str) -> String {
    // Prefer URL-encoding for SVG so we avoid base64 dependency for icons.
    let encoded = urlencoding_minimal(svg);
    format!("data:image/svg+xml;charset=utf-8,{encoded}")
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).copied().map_or(0, u32::from);
        let b2 = chunk.get(2).copied().map_or(0, u32::from);
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_level_svgs_are_distinct() {
        let info = icon_html_for_level(MessageLevel::Info);
        let warning = icon_html_for_level(MessageLevel::Warning);
        let error = icon_html_for_level(MessageLevel::Error);
        let question = icon_html_for_level(MessageLevel::Question);
        assert!(info.contains(r#"data-icon-role="info""#));
        assert!(info.contains(r#"data-icon-variant="1""#));
        assert!(warning.contains(r#"data-icon-role="warning""#));
        assert!(error.contains(r#"data-icon-role="error""#));
        assert!(question.contains(r#"data-icon-role="question""#));
        assert!(!info.contains("data-placeholder-level"));
        assert_ne!(info, warning);
        assert_ne!(warning, error);
        assert_ne!(error, question);
    }

    #[test]
    fn placeholder_assets_retained_for_regression() {
        let info = placeholder_svg_for_level(MessageLevel::Info);
        let warning = placeholder_svg_for_level(MessageLevel::Warning);
        let error = placeholder_svg_for_level(MessageLevel::Error);
        let question = placeholder_svg_for_level(MessageLevel::Question);
        assert!(info.contains(r#"data-placeholder-level="info""#));
        assert!(warning.contains(r#"data-placeholder-level="warning""#));
        assert!(error.contains(r#"data-placeholder-level="error""#));
        assert!(question.contains(r#"data-placeholder-level="question""#));
        assert_ne!(info, warning);
    }

    #[test]
    fn named_icon_with_variant_maps_to_production_variant_one() {
        let html = resolve_level_icon_html(None, Some("warning:2"))
            .expect("named")
            .expect("some");
        assert!(html.contains(r#"data-icon-role="warning""#));
        assert!(html.contains(r#"data-icon-variant="1""#));
        assert!(!html.contains("data-placeholder-level"));
    }

    #[test]
    fn named_success_and_loading_roles_resolve() {
        let success = resolve_level_icon_html(None, Some("success"))
            .expect("ok")
            .expect("some");
        let loading = resolve_level_icon_html(None, Some("loading"))
            .expect("ok")
            .expect("some");
        assert!(success.contains(r#"data-icon-role="success""#));
        assert!(loading.contains(r#"data-icon-role="loading""#));
    }

    #[test]
    fn data_uri_icon_embeds_img() {
        let html = resolve_level_icon_html(None, Some("data:image/png;base64,AA=="))
            .expect("data")
            .expect("some");
        assert!(html.contains(r#"src="data:image/png;base64,AA==""#));
    }

    #[test]
    fn icon_wins_over_level() {
        let html = resolve_level_icon_html(Some(MessageLevel::Info), Some("error"))
            .expect("ok")
            .expect("some");
        assert!(html.contains(r#"data-icon-role="error""#));
        assert!(!html.contains(r#"data-icon-role="info""#));
    }

    #[test]
    fn missing_path_returns_run_error() {
        let err = resolve_level_icon_html(None, Some("/nonexistent/wyvern-icon-missing.svg"))
            .expect_err("io");
        assert!(matches!(err, RunError::WindowCreate { .. }));
    }
}
