//! Validated identifier newtypes for the extension registry.

/// Validated, non-empty extension identifier.
///
/// Constructed via `serde` `try_from` — guaranteed non-empty after trim.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ExtensionId(String);

impl ExtensionId {
    /// Returns the id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Failure from [`ExtensionId::try_from`].
///
/// A string newtype because this conversion is used exclusively via
/// `serde::Deserialize`, where `de::Error::custom` wraps the message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionIdError(String);

impl std::fmt::Display for ExtensionIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ExtensionIdError {}

impl TryFrom<String> for ExtensionId {
    type Error = ExtensionIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ExtensionIdError(
                "extension id must not be empty or whitespace".into(),
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl std::fmt::Display for ExtensionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for ExtensionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for ExtensionId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ExtensionId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Declared `{arg:name}` flag name (no leading dashes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
pub struct ArgName(String);

impl ArgName {
    /// Wrap a non-empty flag name after trim.
    ///
    /// Returns `None` when `name` is empty or whitespace.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Option<Self> {
        Self::try_from(name.into()).ok()
    }

    /// Returns the flag name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ArgName {
    type Error = ExtensionIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ExtensionIdError(
                "arg name must not be empty or whitespace".into(),
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl std::fmt::Display for ArgName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for ArgName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for ArgName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for ArgName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ArgName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Bare PATH binary name (non-empty, no path separators).
///
/// Constructed via `serde` `try_from` at registry load.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
pub struct BinaryName(String);

impl BinaryName {
    /// Returns the binary name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BinaryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for BinaryName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for BinaryName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for BinaryName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for BinaryName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl TryFrom<String> for BinaryName {
    type Error = ExtensionIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ExtensionIdError(
                "binary name must not be empty or whitespace".into(),
            ));
        }
        if trimmed.contains('/') || trimmed.contains('\\') {
            return Err(ExtensionIdError(format!(
                "'{trimmed}' looks like a path; use bare binary name"
            )));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl<'de> serde::Deserialize<'de> for BinaryName {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
    }
}

/// Non-empty match token (suffix, filename, or argv prefix element).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchToken(String);

impl MatchToken {
    /// Returns the token as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MatchToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for MatchToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MatchToken {
    type Error = ExtensionIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ExtensionIdError(
                "match token must not be empty or whitespace".into(),
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl<'de> serde::Deserialize<'de> for MatchToken {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for ExtensionId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
    }
}
