//! Validated button labels for protocol results.

use std::fmt;
use std::ops::Deref;

use serde::Serialize;

/// Button label returned on stdout for chrome and dialog types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ButtonLabel(String);

impl ButtonLabel {
    /// OS chrome close / dismiss without an explicit button press.
    pub fn dismissed() -> Self {
        Self("dismissed".into())
    }

    /// Wrap an arbitrary button label (later phases).
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the label as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ButtonLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ButtonLabel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ButtonLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<str> for ButtonLabel {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ButtonLabel {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dismissed_serializes_to_wire_shape() {
        let label = ButtonLabel::dismissed();
        let json = serde_json::to_string(&label).expect("serialize");
        assert_eq!(json, "\"dismissed\"");
    }
}
