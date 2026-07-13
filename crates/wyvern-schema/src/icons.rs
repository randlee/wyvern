//! Built-in icon role catalog (REQ-0030).
//!
//! Pure validation / naming helpers shared by schema validation and window
//! embed lookup. Asset bytes live in `wyvern-window` (ADR-0011 / ADR-0015).

/// Semantic roles shipped in the curated icon bundle.
pub const ROLES: &[&str] = &["info", "warning", "error", "question", "success", "loading"];

/// Max 1-based variant index for `role`, or `0` when unknown.
pub fn variant_count(role: &str) -> u32 {
    match role {
        "info" | "warning" | "error" | "question" | "success" | "loading" => 2,
        _ => 0,
    }
}

/// Parse `"role"` or `"role:N"` where `N` is a 1-based variant index.
///
/// # Errors
///
/// Returns `Err(())` when the suffix after `:` is present but not a valid `u32`
/// (for example `"warning:abc"`). Callers map that to [`crate::ValidationError`].
pub fn parse_icon_spec(spec: &str) -> Result<(String, u32), ()> {
    let (base, variant) = match spec.split_once(':') {
        None => (spec, 1u32),
        Some((b, v)) => (b, v.parse::<u32>().map_err(|_| ())?),
    };
    Ok((base.to_string(), variant))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_lists_six_names() {
        assert_eq!(ROLES.len(), 6);
        assert!(ROLES.contains(&"info"));
        assert!(ROLES.contains(&"loading"));
    }

    #[test]
    fn variant_count_known_and_unknown() {
        assert_eq!(variant_count("warning"), 2);
        assert_eq!(variant_count("nope"), 0);
    }

    #[test]
    fn parse_icon_spec_default_and_index() {
        assert_eq!(parse_icon_spec("error"), Ok(("error".into(), 1)));
        assert_eq!(parse_icon_spec("warning:2"), Ok(("warning".into(), 2)));
    }

    #[test]
    fn parse_icon_spec_rejects_non_numeric_suffix() {
        assert_eq!(parse_icon_spec("warning:abc"), Err(()));
    }
}
