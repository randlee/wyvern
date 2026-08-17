//! Argv match kinds and help-prefix matching for CLI extensions.

use std::path::Path;

use super::{ExtensionDef, ExtensionRegistry, MatchSpec, MatchToken};

/// Successful argv match against one extension.
#[derive(Debug, Clone)]
pub enum ExtensionMatch<'a> {
    /// Single positional suffix or exact filename.
    Suffix {
        /// Matched extension.
        ext: &'a ExtensionDef,
        /// Matched file path token.
        path: &'a str,
    },
    /// Prefix-only (for example `compose render --root …`).
    Prefix {
        /// Matched extension.
        ext: &'a ExtensionDef,
        /// Tokens after the prefix.
        args_after_prefix: &'a [String],
    },
    /// Prefix plus a suffix-matching path token.
    PrefixSuffix {
        /// Matched extension.
        ext: &'a ExtensionDef,
        /// Matched file path token.
        path: &'a str,
        /// Tokens after the prefix (includes the path).
        args_after_prefix: &'a [String],
    },
}

impl<'a> ExtensionMatch<'a> {
    /// Extension that matched.
    #[must_use]
    pub fn extension(&self) -> &'a ExtensionDef {
        match self {
            Self::Suffix { ext, .. }
            | Self::Prefix { ext, .. }
            | Self::PrefixSuffix { ext, .. } => ext,
        }
    }

    /// Matched file path, if this match kind has one.
    #[must_use]
    pub fn path(&self) -> Option<&'a str> {
        match self {
            Self::Suffix { path, .. } | Self::PrefixSuffix { path, .. } => Some(*path),
            Self::Prefix { .. } => None,
        }
    }

    /// Tokens after an argv prefix (empty for suffix-only matches).
    #[must_use]
    pub fn args_after_prefix(&self) -> &'a [String] {
        match self {
            Self::Prefix {
                args_after_prefix, ..
            }
            | Self::PrefixSuffix {
                args_after_prefix, ..
            } => args_after_prefix,
            Self::Suffix { .. } => &[],
        }
    }
}

impl ExtensionDef {
    pub(super) fn match_spec_argv<'a>(&'a self, argv: &'a [String]) -> Option<ExtensionMatch<'a>> {
        let spec = &self.match_spec;
        if let Some(prefix) = &spec.argv_prefix {
            if argv.len() < prefix.len()
                || !prefix
                    .iter()
                    .zip(argv.iter())
                    .all(|(expected, got)| expected.as_str() == got)
            {
                return None;
            }
            let rest = &argv[prefix.len()..];
            if let Some(suffix) = &spec.arg_suffix {
                let path = rest
                    .iter()
                    .find(|token| ends_with_suffix(token, suffix.as_str()))?;
                return Some(ExtensionMatch::PrefixSuffix {
                    ext: self,
                    path: path.as_str(),
                    args_after_prefix: rest,
                });
            }
            return Some(ExtensionMatch::Prefix {
                ext: self,
                args_after_prefix: rest,
            });
        }
        if argv.len() != 1 {
            return None;
        }
        let token = argv[0].as_str();
        if let Some(filename) = &spec.filename {
            let base = Path::new(token).file_name()?.to_str()?;
            if base == filename.as_str() {
                return Some(ExtensionMatch::Suffix {
                    ext: self,
                    path: token,
                });
            }
            return None;
        }
        if let Some(suffix) = &spec.positional_suffix {
            if ends_with_suffix(token, suffix.as_str()) {
                return Some(ExtensionMatch::Suffix {
                    ext: self,
                    path: token,
                });
            }
        }
        None
    }
}

pub(crate) fn ends_with_suffix(token: &str, suffix: &str) -> bool {
    let Some(start) = token.len().checked_sub(suffix.len()) else {
        return false;
    };
    // `str::get` is `None` when `start` is not a UTF-8 char boundary, so a
    // multi-byte token cannot panic the way a raw byte slice would.
    token
        .get(start..)
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

/// Match an extension prefix whose remaining tokens are only `--help` / `-h`.
///
/// Ignores `preexec.requires` and does not require an `arg_suffix` path token.
/// When two prefixes match, the longest prefix wins.
#[must_use]
pub fn match_extension_help<'a>(
    registry: &'a ExtensionRegistry,
    argv: &'a [String],
) -> Option<&'a ExtensionDef> {
    let mut best: Option<(&'a ExtensionDef, usize)> = None;
    for ext in registry.extensions() {
        let Some(prefix) = &ext.match_spec.argv_prefix else {
            continue;
        };
        if prefix.is_empty() || argv.len() < prefix.len() {
            continue;
        }
        if !prefix
            .iter()
            .zip(argv.iter())
            .all(|(expected, got)| expected.as_str() == got)
        {
            continue;
        }
        if !is_help_only_tokens(&argv[prefix.len()..]) {
            continue;
        }
        let len = prefix.len();
        if best.is_none_or(|(_, best_len)| len > best_len) {
            best = Some((ext, len));
        }
    }
    best.map(|(ext, _)| ext)
}

/// Returns whether every token is `--help` or `-h` (and at least one is present).
#[must_use]
pub fn is_help_only_tokens(tokens: &[String]) -> bool {
    !tokens.is_empty()
        && tokens
            .iter()
            .all(|token| token == "--help" || token == "-h")
}

#[must_use]
pub fn match_kind_summary(spec: &MatchSpec) -> String {
    if let Some(prefix) = &spec.argv_prefix {
        let prefix_s = prefix
            .iter()
            .map(MatchToken::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(suffix) = &spec.arg_suffix {
            return format!("prefix+suffix: {prefix_s} {suffix}");
        }
        return format!("prefix: {prefix_s}");
    }
    if let Some(filename) = &spec.filename {
        return format!("filename: {filename}");
    }
    if let Some(suffix) = &spec.positional_suffix {
        return format!("suffix: {suffix}");
    }
    "match: (none)".to_string()
}
