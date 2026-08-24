//! Validated picker filter patterns and start paths (RBP-004).
//!
//! Construction is the single validation boundary for HTTP picker overrides
//! and `input` `filter` / `start_path` strings. [`Command`](crate::Command)
//! field types stay `String` (ADR-0026: no Command variant change).

use std::fmt;
use std::ops::Deref;

/// Failure constructing a [`FilterPattern`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterPatternError {
    /// Pattern was empty or whitespace-only.
    Empty,
    /// Pattern contained a NUL byte.
    ContainsNul,
}

impl fmt::Display for FilterPatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("filter pattern must be a non-empty string"),
            Self::ContainsNul => f.write_str("filter pattern must not contain NUL bytes"),
        }
    }
}

impl std::error::Error for FilterPatternError {}

/// Validated file-picker extension pattern (non-empty, no NUL).
///
/// Construct via [`Self::try_new`] at trust boundaries (command JSON and
/// `POST /api/picker/file` body overrides).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilterPattern(String);

impl FilterPattern {
    /// Construct a non-empty, NUL-free filter pattern.
    ///
    /// # Errors
    ///
    /// Returns [`FilterPatternError::Empty`] when `value` is empty or
    /// whitespace-only, or [`FilterPatternError::ContainsNul`] when it
    /// contains a NUL byte.
    pub fn try_new(value: impl Into<String>) -> Result<Self, FilterPatternError> {
        let value = value.into();
        if value.is_empty() || value.trim().is_empty() {
            return Err(FilterPatternError::Empty);
        }
        if value.contains('\0') {
            return Err(FilterPatternError::ContainsNul);
        }
        Ok(Self(value))
    }

    /// Borrow the pattern as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for FilterPattern {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for FilterPattern {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FilterPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<str> for FilterPattern {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for FilterPattern {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Failure constructing a [`PickerPath`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerPathError {
    /// Path was an empty string.
    Empty,
    /// Path contained a NUL byte.
    ContainsNul,
}

impl fmt::Display for PickerPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("picker path must be a non-empty string"),
            Self::ContainsNul => f.write_str("picker path must not contain NUL bytes"),
        }
    }
}

impl std::error::Error for PickerPathError {}

/// Validated picker start path (non-empty, no NUL).
///
/// Construct via [`Self::try_new`] at trust boundaries (command JSON
/// `start_path` and `POST /api/picker/*` body overrides).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PickerPath(String);

impl PickerPath {
    /// Construct a non-empty, NUL-free picker path.
    ///
    /// # Errors
    ///
    /// Returns [`PickerPathError::Empty`] when `value` is empty, or
    /// [`PickerPathError::ContainsNul`] when it contains a NUL byte.
    pub fn try_new(value: impl Into<String>) -> Result<Self, PickerPathError> {
        let value = value.into();
        if value.is_empty() {
            return Err(PickerPathError::Empty);
        }
        if value.contains('\0') {
            return Err(PickerPathError::ContainsNul);
        }
        Ok(Self(value))
    }

    /// Borrow the path as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for PickerPath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for PickerPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PickerPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<str> for PickerPath {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for PickerPath {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_pattern_try_new_rejects_empty_and_whitespace() {
        assert_eq!(
            FilterPattern::try_new("").unwrap_err(),
            FilterPatternError::Empty
        );
        assert_eq!(
            FilterPattern::try_new("   ").unwrap_err(),
            FilterPatternError::Empty
        );
        assert_eq!(FilterPattern::try_new("*.txt").unwrap().as_str(), "*.txt");
    }

    #[test]
    fn filter_pattern_try_new_rejects_nul() {
        assert_eq!(
            FilterPattern::try_new("*.t\0xt").unwrap_err(),
            FilterPatternError::ContainsNul
        );
    }

    #[test]
    fn picker_path_try_new_rejects_empty_and_nul() {
        assert_eq!(PickerPath::try_new("").unwrap_err(), PickerPathError::Empty);
        assert_eq!(
            PickerPath::try_new("/tmp\0").unwrap_err(),
            PickerPathError::ContainsNul
        );
        assert_eq!(PickerPath::try_new("/tmp").unwrap().as_str(), "/tmp");
    }

    #[test]
    fn picker_path_allows_whitespace_only() {
        assert_eq!(PickerPath::try_new("  ").unwrap().as_str(), "  ");
    }
}
