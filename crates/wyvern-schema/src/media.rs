//! Validated media references for dialog `icon` / `image` fields.

use std::fmt;
use std::ops::Deref;

use serde::Serialize;

/// Validated icon or image reference (path, URL, data URI, or named template hint).
///
/// Construct only after structural validation ([`crate::validate`]) so downstream
/// code can treat the value as a checked media reference rather than a raw string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct MediaRef(String);

impl MediaRef {
    /// Wrap a validated media reference string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the reference as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for MediaRef {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for MediaRef {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for MediaRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for MediaRef {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for MediaRef {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq<str> for MediaRef {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for MediaRef {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_string() {
        let media = MediaRef::new("warning");
        let json = serde_json::to_string(&media).expect("serialize");
        assert_eq!(json, "\"warning\"");
    }
}
