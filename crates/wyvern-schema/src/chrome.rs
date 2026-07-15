//! Validated chrome field newtypes.
//!
//! Construction is the single validation boundary for chrome `title` /
//! `status` strings. Phase A wire semantics: `title` is required as a string
//! (empty string allowed); `status` is optional and, when present, must be a
//! string (empty string allowed).

use std::fmt;
use std::ops::Deref;

/// Validated chrome window title (Phase A: any string, including empty).
///
/// Construct only via [`ChromeTitle::new`] (or `From`) at the validation
/// boundary so downstream code can treat the value as already checked.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChromeTitle(String);

impl ChromeTitle {
    /// Wrap a validated title string.
    ///
    /// Phase A does not reject empty titles; the field must simply be present
    /// as a JSON string during [`crate::validate`].
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the title as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for ChromeTitle {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ChromeTitle {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ChromeTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for ChromeTitle {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ChromeTitle {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq<str> for ChromeTitle {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ChromeTitle {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Validated optional chrome status line (Phase A: any string when present).
///
/// Construct only via [`ChromeStatus::new`] (or `From`) at the validation
/// boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChromeStatus(String);

impl ChromeStatus {
    /// Wrap a validated status string.
    ///
    /// Phase A does not reject empty status strings when the field is present.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the status as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for ChromeStatus {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ChromeStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ChromeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for ChromeStatus {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ChromeStatus {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq<str> for ChromeStatus {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ChromeStatus {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_title_allows_empty_phase_a() {
        let title = ChromeTitle::new("");
        assert_eq!(title.as_str(), "");
        assert_eq!(title, "");
    }

    #[test]
    fn chrome_status_round_trips() {
        let status = ChromeStatus::from("Ready");
        assert_eq!(status.as_str(), "Ready");
        assert_eq!(&*status, "Ready");
    }
}
