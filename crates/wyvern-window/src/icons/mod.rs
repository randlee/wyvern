//! Compile-time embed of the curated icon bundle (REQ-0030 / ADR-0015).
//!
//! Bytes are baked via [`include_bytes!`]; built-in icons never touch the
//! filesystem at runtime. Role / variant bounds come from
//! [`wyvern_schema::icons`].

use wyvern_schema::icons;

/// Embedded bytes for `role` at 1-based `index`, if that variant ships.
pub fn variant_bytes(role: &str, index: u32) -> Option<&'static [u8]> {
    if index < 1 || index > icons::variant_count(role) {
        return None;
    }
    match (role, index) {
        ("info", 1) => Some(include_bytes!("../../assets/icons/info/1.svg")),
        ("info", 2) => Some(include_bytes!("../../assets/icons/info/2.svg")),
        ("warning", 1) => Some(include_bytes!("../../assets/icons/warning/1.svg")),
        ("warning", 2) => Some(include_bytes!("../../assets/icons/warning/2.svg")),
        ("error", 1) => Some(include_bytes!("../../assets/icons/error/1.svg")),
        ("error", 2) => Some(include_bytes!("../../assets/icons/error/2.svg")),
        ("question", 1) => Some(include_bytes!("../../assets/icons/question/1.svg")),
        ("question", 2) => Some(include_bytes!("../../assets/icons/question/2.svg")),
        ("success", 1) => Some(include_bytes!("../../assets/icons/success/1.svg")),
        ("success", 2) => Some(include_bytes!("../../assets/icons/success/2.svg")),
        ("loading", 1) => Some(include_bytes!("../../assets/icons/loading/1.svg")),
        ("loading", 2) => Some(include_bytes!("../../assets/icons/loading/2.svg")),
        _ => None,
    }
}

/// UTF-8 SVG markup for `role` at 1-based `index`, if that variant ships.
pub fn svg_markup(role: &str, index: u32) -> Option<&'static str> {
    variant_bytes(role, index).and_then(|b| std::str::from_utf8(b).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::icons::ROLES;

    #[test]
    fn every_role_embeds_two_distinct_variants() {
        for role in ROLES {
            let v1 = svg_markup(role, 1).expect("variant 1");
            let v2 = svg_markup(role, 2).expect("variant 2");
            assert!(
                v1.contains(&format!(r#"data-icon-role="{role}""#)),
                "missing role marker in {role}/1"
            );
            assert!(
                v2.contains(&format!(r#"data-icon-role="{role}""#)),
                "missing role marker in {role}/2"
            );
            assert!(v1.contains(r#"data-icon-variant="1""#));
            assert!(v2.contains(r#"data-icon-variant="2""#));
            assert!(!v1.contains("data-placeholder-level"));
            assert!(!v2.contains("data-placeholder-level"));
            assert_ne!(v1, v2, "{role} variants must differ");
        }
    }

    #[test]
    fn out_of_range_returns_none() {
        assert!(svg_markup("info", 0).is_none());
        assert!(svg_markup("info", 3).is_none());
        assert!(svg_markup("unknown", 1).is_none());
    }
}
