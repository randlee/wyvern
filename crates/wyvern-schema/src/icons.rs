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
/// Does **not** check that `role` is a known catalog name or that `N` is in
/// range — use [`NamedIconSpec::parse`] for full validation.
///
/// # Errors
///
/// Returns `Err(())` when the suffix after `:` is present but not a valid `u32`
/// (for example `"warning:abc"`). Callers map that to [`crate::ValidationError`].
#[expect(
    clippy::result_unit_err,
    reason = "c.1 catalog API; validation maps Err(()) to ValidationError in c.2"
)]
pub fn parse_icon_spec(spec: &str) -> Result<(String, u32), ()> {
    let (base, variant) = match spec.split_once(':') {
        None => (spec, 1u32),
        Some((b, v)) => (b, v.parse::<u32>().map_err(|_| ())?),
    };
    Ok((base.to_string(), variant))
}

/// Validated named icon reference (`role` or `role:N`) against the catalog.
///
/// Construct via [`NamedIconSpec::parse`] so window-layer defense-in-depth and
/// schema validation share one typed boundary (RBP-F002).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedIconSpec {
    role: String,
    variant: u32,
}

impl NamedIconSpec {
    /// Parse and validate a named icon spec against [`ROLES`] / [`variant_count`].
    ///
    /// # Errors
    ///
    /// Returns `Err(())` when the spec is malformed, the role is unknown, or
    /// the variant is out of range (including `0`).
    #[expect(
        clippy::result_unit_err,
        reason = "shared with parse_icon_spec; callers map to ValidationError / RunError"
    )]
    pub fn parse(spec: &str) -> Result<Self, ()> {
        let (role, variant) = parse_icon_spec(spec)?;
        if !ROLES.contains(&role.as_str()) {
            return Err(());
        }
        let max = variant_count(&role);
        if variant == 0 || variant > max {
            return Err(());
        }
        Ok(Self { role, variant })
    }

    /// Semantic role name (`info`, `warning`, …).
    pub fn role(&self) -> &str {
        &self.role
    }

    /// 1-based variant index.
    pub fn variant(&self) -> u32 {
        self.variant
    }
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

    #[test]
    fn named_icon_spec_accepts_catalog_entries() {
        let spec = NamedIconSpec::parse("warning:2").expect("ok");
        assert_eq!(spec.role(), "warning");
        assert_eq!(spec.variant(), 2);
        assert!(NamedIconSpec::parse("success").is_ok());
    }

    #[test]
    fn named_icon_spec_rejects_unknown_or_oor() {
        assert!(NamedIconSpec::parse("nope").is_err());
        assert!(NamedIconSpec::parse("warning:99").is_err());
        assert!(NamedIconSpec::parse("warning:0").is_err());
        assert!(NamedIconSpec::parse("warning:abc").is_err());
    }
}
