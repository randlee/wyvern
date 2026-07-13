//! Validated JSON field path names for error reporting.

use std::fmt;
use std::ops::Deref;

/// A command JSON field path used in validation and stderr errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldName(String);

impl FieldName {
    /// Wrap a field path (e.g. `title`, `type`, `file`).
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the field name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for FieldName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for FieldName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for FieldName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for FieldName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl PartialEq<str> for FieldName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for FieldName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl serde::Serialize for FieldName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
