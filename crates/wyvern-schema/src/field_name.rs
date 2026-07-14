//! Validated JSON field path names for error reporting.

use std::fmt;
use std::ops::Deref;

/// Error when constructing a [`FieldName`] from an empty string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldNameError;

impl fmt::Display for FieldNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("field name must not be empty")
    }
}

impl std::error::Error for FieldNameError {}

/// A command JSON field path used in validation and stderr errors.
///
/// Invariant: the inner string is non-empty after construction. Prefer
/// [`FieldName::try_new`] at trust boundaries; [`FieldName::new`] maps empty
/// input to `"_"` so emit helpers never panic (RBP-F006).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldName(String);

impl FieldName {
    /// Wrap a field path (e.g. `title`, `type`, `file`).
    ///
    /// Empty strings are replaced with `"_"` so callers that already validated
    /// the payload cannot panic at the emit boundary. Use [`try_new`] when an
    /// empty path should be treated as an error.
    pub fn new(value: impl Into<String>) -> Self {
        match Self::try_new(value) {
            Ok(name) => name,
            Err(FieldNameError) => Self("_".into()),
        }
    }

    /// Construct a field name, rejecting empty strings.
    ///
    /// # Errors
    ///
    /// Returns [`FieldNameError`] when `value` is empty after conversion.
    pub fn try_new(value: impl Into<String>) -> Result<Self, FieldNameError> {
        let value = value.into();
        if value.is_empty() {
            return Err(FieldNameError);
        }
        Ok(Self(value))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_rejects_empty() {
        assert_eq!(FieldName::try_new("").unwrap_err(), FieldNameError);
        assert_eq!(FieldName::try_new("title").unwrap().as_str(), "title");
    }

    #[test]
    fn new_maps_empty_to_underscore() {
        assert_eq!(FieldName::new("").as_str(), "_");
        assert_eq!(FieldName::new("file").as_str(), "file");
    }
}
