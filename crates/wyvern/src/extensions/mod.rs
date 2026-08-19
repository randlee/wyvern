//! CLI extension registry: load, merge, match, expand, and dispatch.
//!
//! Extensions are an argv preprocessor inside the `wyvern` crate (ADR-0022).
//! They produce existing [`wyvern_schema::Command`] JSON only — no new host
//! dialog types. Phase E `--interactive` reuses this module; MCP tools consume
//! pre-expanded Command JSON.
//!
//! # Examples
//!
//! ```rust,no_run
//! use std::path::Path;
//! use wyvern::extensions::{build_match_context, expand_and_validate, ExtensionRegistry};
//!
//! let registry = ExtensionRegistry::load(Path::new("share/wyvern/extensions.json"), None)?;
//! let argv = vec!["doc.md".to_string()];
//! if let Some(matched) = registry.match_argv(&argv) {
//!     let ctx = build_match_context(&matched, matched.extension());
//!     let expanded = expand_and_validate(matched.extension(), &ctx)?;
//!     assert_eq!(expanded.command["type"], "markdown");
//! }
//! # Ok::<(), wyvern::extensions::ExtensionError>(())
//! ```

mod catalog;
mod diagnostics;
mod expand;
mod extends;
mod ids;
mod list;
mod match_logic;
mod preexec;
mod share_resolve;

use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

#[doc(inline)]
pub use catalog::{
    build_skill_record, build_skill_records, format_skill_card, skill_help_command, SkillArg,
    SkillRecord, SkillRequire,
};
#[doc(inline)]
pub use diagnostics::{
    classify_near_miss, emit_near_miss, MatchOutcome, NearMissKind, SkippedExtension,
};
#[doc(inline)]
pub use expand::{
    build_match_context, expand_and_validate, expand_command_host, expand_preexec_args,
    infer_wizard_root, last_created_tmpdir, relpath_from_ui_root, ExpandedInvocation,
    HostOverrides, MatchContext,
};
#[doc(inline)]
pub use ids::{ArgName, BinaryName, ExtensionId, ExtensionIdError, MatchToken};
#[doc(inline)]
pub use list::{
    extensions_usage_message, format_extensions_list, run_extensions_command, ExtensionsCmdError,
};
#[doc(inline)]
pub use match_logic::{
    is_help_only_tokens, match_extension_help, match_kind_summary, ExtensionMatch,
};
#[doc(inline)]
pub use preexec::{
    binary_on_path, create_tmpdir, run_preexec, run_script, PathRequiresProbe, PreexecFailureKind,
    RequiresProbe, ScriptError, ScriptOutput, ScriptRequest,
};

pub(crate) use match_logic::ends_with_suffix;

/// Shipped defaults compiled into the binary (dev + `cargo install`).
pub const SHIPPED_EXTENSIONS_JSON: &str = include_str!("../../../../share/wyvern/extensions.json");

/// Embedded `share/wyvern/**` assets (`extensions.json`, packaged UI extras).
#[derive(rust_embed::RustEmbed)]
#[folder = "../../share/wyvern"]
pub struct ShareAssets;

/// Embedded `scripts/ext/**` preexec helpers.
#[derive(rust_embed::RustEmbed)]
#[folder = "../../scripts/ext"]
pub struct ScriptAssets;

/// Merged, `extends`-resolved extension registry.
#[derive(Debug, Clone)]
pub struct ExtensionRegistry {
    extensions: Vec<ExtensionDef>,
}

/// Whether a catalog entry came from shipped defaults or a project file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    /// Built-in `share/wyvern/extensions.json` (or compiled fallback).
    #[default]
    Shipped,
    /// Project `.wyvern/extensions.json` (trusted preexec).
    Project,
}

impl SkillSource {
    /// Stable wire / help label (`shipped` or `project`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shipped => "shipped",
            Self::Project => "project",
        }
    }
}

impl std::fmt::Display for SkillSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One registry entry after merge and `extends` resolution.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionDef {
    /// Stable extension id (merge key). Must be non-empty after trim.
    pub id: ExtensionId,
    /// Argv match rule.
    #[serde(rename = "match")]
    pub match_spec: MatchSpec,
    /// Optional parent id whose preexec/expand are reused.
    #[serde(default)]
    pub extends: Option<ExtensionId>,
    /// One-line agent-facing summary (optional; recommended on shipped skills).
    #[serde(default)]
    pub description: Option<String>,
    /// Copy-paste argv examples (optional; recommended on shipped skills).
    #[serde(default)]
    pub examples: Vec<String>,
    /// Optional subprocess step before command expand.
    #[serde(default)]
    pub preexec: Option<PreexecSpec>,
    /// Command + host template expansion.
    #[serde(default)]
    pub expand: Option<ExpandSpec>,
    /// Load origin; not present in registry JSON (`shipped` by default).
    #[serde(skip)]
    pub source: SkillSource,
}

/// Match fields from the registry schema.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchSpec {
    /// Single positional ends with this suffix (`.md`).
    #[serde(default)]
    pub positional_suffix: Option<MatchToken>,
    /// Exact basename match (`wizard.json`).
    #[serde(default)]
    pub filename: Option<MatchToken>,
    /// First N argv tokens (`["compose", "render"]`).
    #[serde(default)]
    pub argv_prefix: Option<Vec<MatchToken>>,
    /// Token after prefix matches this suffix.
    #[serde(default)]
    pub arg_suffix: Option<MatchToken>,
}

/// Stdout capture mode for [`PreexecSpec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StdoutCapture {
    /// Capture stdout as markdown text and inject as `{preexec.stdout}`.
    Markdown,
}

/// Preexec subprocess declaration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreexecSpec {
    /// Executable name or path (phase-1 expanded).
    #[serde(default)]
    pub cmd: String,
    /// Argv tokens (phase-1 expanded; `{arg:name:repeat}` splices).
    #[serde(default)]
    pub args: Vec<String>,
    /// Binaries that must be on `PATH` or the extension does not match.
    #[serde(default)]
    pub requires: Vec<BinaryName>,
    /// Stdout capture mode (`markdown` only in Phase F).
    #[serde(default)]
    pub stdout: Option<StdoutCapture>,
}

/// Expand templates for command JSON and host overrides.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpandSpec {
    /// Inline Command JSON after phase-2 substitution.
    #[serde(default)]
    pub command: Option<Value>,
    /// Load Command JSON from this path template.
    #[serde(default)]
    pub command_from_file: Option<String>,
    /// Host overrides (`ui_root` only in Phase F).
    #[serde(default)]
    pub host: Option<HostExpandSpec>,
}

/// Host template object (`ui_root` only).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostExpandSpec {
    /// Template for [`HostOverrides::ui_root`].
    #[serde(default)]
    pub ui_root: Option<String>,
}

/// Why [`ExtensionError::Template`] failed (RBP-F006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateErrorKind {
    /// Template contained `{` without a matching `}`.
    UnclosedBrace,
    /// `{name}` is not a known template variable.
    UnknownVariable,
    /// Variable is valid only in the other expansion phase.
    PhaseRestricted,
    /// Variable is known but not available in this context.
    Unavailable,
    /// Expand/preexec spec is incomplete or contradictory.
    InvalidSpec,
}

/// Structured extension-engine failure.
#[derive(Debug)]
pub enum ExtensionError {
    /// Registry JSON or schema is invalid.
    InvalidRegistry {
        /// Human-readable load failure.
        message: String,
    },
    /// One or more required `{arg:name}` flags were missing.
    MissingArgs {
        /// Missing flags including leading dashes (`--root`).
        missing: Vec<String>,
        /// All declared `{arg:*}` names (no dashes).
        declared: std::collections::BTreeSet<String>,
        /// Extension that required the flags.
        extension_id: ExtensionId,
        /// Copy-paste example from the skill card.
        example: String,
        /// Invocation-prefix help (`wyvern compose render --help`), not the id.
        help_command: String,
    },
    /// Unexpected token after a successful prefix match.
    UnexpectedArg {
        /// Offending token.
        token: String,
        /// Declared `{arg:*}` names (no dashes).
        declared: std::collections::BTreeSet<String>,
        /// Extension that matched argv.
        extension_id: ExtensionId,
        /// Invocation-prefix help (`wyvern compose render --help`), not the id.
        help_command: String,
    },
    /// Path-derived template used without a matched path.
    PathVarWithoutPath {
        /// Template variable name.
        var: String,
    },
    /// Template expansion failure (see [`TemplateErrorKind`] for the sub-mode).
    Template {
        /// Discriminated substitution failure class.
        kind: TemplateErrorKind,
        /// Substitution failure detail.
        message: String,
    },
    /// Preexec process failed or could not be spawned.
    Preexec {
        /// Spawn-not-found, nonzero-exit, or timeout (`None` for other spawn I/O).
        kind: Option<PreexecFailureKind>,
        /// Human-readable subprocess failure.
        message: String,
        /// Original error if available.
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
    /// Expanded command failed [`wyvern_schema::validate`].
    InvalidCommand {
        /// Schema validation error.
        source: wyvern_schema::ValidationError,
    },
    /// Filesystem failure while loading or expanding.
    Io {
        /// Human-readable I/O error description.
        message: String,
        /// Original I/O error if available.
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRegistry { message } => write!(f, "invalid extension registry: {message}"),
            Self::MissingArgs { missing, .. } => {
                write!(
                    f,
                    "missing required extension arguments {}",
                    missing.join(", ")
                )
            }
            Self::UnexpectedArg { token, .. } => {
                write!(f, "unexpected argument after extension match: {token}")
            }
            Self::PathVarWithoutPath { var } => {
                write!(f, "template {{{var}}} requires a matched file path")
            }
            Self::Template { message, .. } => write!(f, "extension template error: {message}"),
            Self::Preexec { message, .. } => write!(f, "extension preexec failed: {message}"),
            Self::InvalidCommand { source } => {
                write!(f, "expanded command failed validation: {source}")
            }
            Self::Io { message, .. } => write!(f, "extension I/O error: {message}"),
        }
    }
}

impl std::error::Error for ExtensionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidCommand { source } => Some(source),
            Self::Io { source, .. } => source.as_deref().map(|e| e as _),
            Self::Preexec { source, .. } => source.as_deref().map(|e| e as _),
            _ => None,
        }
    }
}

impl ExtensionError {
    /// Stable process exit code for this failure.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidRegistry { .. } => wyvern_schema::ErrorCode::ParseError.exit_code(),
            Self::Io { .. } | Self::Preexec { .. } => wyvern_schema::ErrorCode::IoError.exit_code(),
            Self::InvalidCommand { source } => source.exit_code(),
            Self::MissingArgs { .. }
            | Self::UnexpectedArg { .. }
            | Self::PathVarWithoutPath { .. }
            | Self::Template { .. } => wyvern_schema::ErrorCode::ValidationError.exit_code(),
        }
    }

    pub(crate) fn template(kind: TemplateErrorKind, message: impl Into<String>) -> Self {
        Self::Template {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    version: u32,
    #[serde(default)]
    extensions: Vec<ExtensionDef>,
}

impl ExtensionRegistry {
    /// Load shipped defaults and an optional project registry, then merge.
    ///
    /// Later files override earlier entries by `id` (in-place). `extends` is
    /// resolved after merge. User config (`~/.config/wyvern/extensions.json`)
    /// is not loaded in Phase F.
    ///
    /// # Errors
    ///
    /// Returns [`ExtensionError::InvalidRegistry`] for unreadable or invalid JSON.
    pub fn load(defaults: &Path, project: Option<&Path>) -> Result<Self, ExtensionError> {
        let default_exts = if defaults.is_file() {
            parse_registry_file(defaults)?
        } else if std::env::var_os("WYVERN_SHARE").is_some() {
            return Err(ExtensionError::InvalidRegistry {
                message: format!(
                    "WYVERN_SHARE is set but '{}' is missing or not a file",
                    defaults.display()
                ),
            });
        } else {
            parse_registry_str(SHIPPED_EXTENSIONS_JSON, "shipped defaults")?
        };
        let project_exts = match project {
            Some(path) if path.is_file() => {
                let mut exts = parse_registry_file(path)?;
                for ext in &mut exts {
                    ext.source = SkillSource::Project;
                }
                exts
            }
            _ => Vec::new(),
        };
        let merged = merge_by_id(default_exts, project_exts);
        let extensions = extends::apply_extends(merged)?;
        Ok(Self { extensions })
    }

    /// Load shipped defaults plus `.wyvern/extensions.json` from the cwd.
    ///
    /// # Errors
    ///
    /// Returns [`ExtensionError::InvalidRegistry`] when a registry file is invalid.
    pub fn load_default() -> Result<Self, ExtensionError> {
        let defaults = resolve_wyvern_share().join("extensions.json");
        let project = std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(".wyvern").join("extensions.json"));
        let project = project.filter(|p| p.is_file());
        Self::load(&defaults, project.as_deref())
    }

    /// Parse a registry from an in-memory JSON string (tests and shipped fallback).
    ///
    /// # Errors
    ///
    /// Returns [`ExtensionError::InvalidRegistry`] when `json` is not a v1 registry.
    pub fn from_json_str(json: &str) -> Result<Self, ExtensionError> {
        let extensions = extends::apply_extends(parse_registry_str(json, "memory")?)?;
        Ok(Self { extensions })
    }

    /// Walk merged extensions in order; first match wins.
    ///
    /// After `extends` resolution, an extension whose `preexec.requires` binaries
    /// are absent on `PATH` does not match (fallthrough).
    #[must_use]
    pub fn match_argv<'a>(&'a self, argv: &'a [String]) -> Option<ExtensionMatch<'a>> {
        self.match_with_diagnostics(argv).matched
    }

    /// [`Self::match_argv`] with an injectable [`RequiresProbe`].
    #[must_use]
    pub fn match_argv_with<'a>(
        &'a self,
        argv: &'a [String],
        probe: &dyn RequiresProbe,
    ) -> Option<ExtensionMatch<'a>> {
        self.match_with_diagnostics_with(argv, probe).matched
    }

    /// Match argv and record extensions skipped for missing `requires`.
    #[must_use]
    pub fn match_with_diagnostics<'a>(&'a self, argv: &'a [String]) -> MatchOutcome<'a> {
        self.match_with_diagnostics_with(argv, &PathRequiresProbe)
    }

    /// [`Self::match_with_diagnostics`] with an injectable [`RequiresProbe`].
    #[must_use]
    pub fn match_with_diagnostics_with<'a>(
        &'a self,
        argv: &'a [String],
        probe: &dyn RequiresProbe,
    ) -> MatchOutcome<'a> {
        let mut skipped = Vec::new();
        for ext in &self.extensions {
            let Some(candidate) = ext.match_spec_argv(argv) else {
                continue;
            };
            let missing: Vec<BinaryName> = ext
                .requires()
                .iter()
                .filter(|bin| !probe.binary_on_path(bin.as_str()))
                .cloned()
                .collect();
            if missing.is_empty() {
                return MatchOutcome {
                    matched: Some(candidate),
                    skipped,
                };
            }
            skipped.push(SkippedExtension {
                id: ext.id.clone(),
                missing,
            });
        }
        MatchOutcome {
            matched: None,
            skipped,
        }
    }

    /// Merged extensions in match order.
    #[must_use]
    pub fn extensions(&self) -> &[ExtensionDef] {
        &self.extensions
    }
}

impl ExtensionDef {
    /// Required binaries advertised for `extensions list` and match-time skip.
    #[must_use]
    pub fn requires(&self) -> &[BinaryName] {
        self.preexec
            .as_ref()
            .map(|p| p.requires.as_slice())
            .unwrap_or(&[])
    }
}

fn parse_registry_file(path: &Path) -> Result<Vec<ExtensionDef>, ExtensionError> {
    const MAX_REGISTRY_BYTES: usize = 1024 * 1024;
    let file = std::fs::File::open(path).map_err(|err| ExtensionError::Io {
        message: format!("could not read '{}': {err}", path.display()),
        source: Some(Box::new(err)),
    })?;
    let mut buf = Vec::new();
    let n = file
        .take(MAX_REGISTRY_BYTES as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|err| ExtensionError::Io {
            message: format!("could not read '{}': {err}", path.display()),
            source: Some(Box::new(err)),
        })?;
    if n > MAX_REGISTRY_BYTES {
        return Err(ExtensionError::InvalidRegistry {
            message: format!(
                "registry file '{}' exceeds maximum of {MAX_REGISTRY_BYTES} bytes",
                path.display()
            ),
        });
    }
    let text = String::from_utf8(buf).map_err(|err| ExtensionError::InvalidRegistry {
        message: format!(
            "registry file '{}' is not valid UTF-8: {err}",
            path.display()
        ),
    })?;
    parse_registry_str(&text, &path.display().to_string())
}

fn parse_registry_str(text: &str, origin: &str) -> Result<Vec<ExtensionDef>, ExtensionError> {
    let file: RegistryFile =
        serde_json::from_str(text).map_err(|err| ExtensionError::InvalidRegistry {
            message: format!("invalid JSON in {origin}: {err}"),
        })?;
    if file.version != 1 {
        return Err(ExtensionError::InvalidRegistry {
            message: format!(
                "unsupported registry version {} in {origin} (expected 1)",
                file.version
            ),
        });
    }
    for ext in &file.extensions {
        if !has_match_field(&ext.match_spec) && ext.extends.is_none() {
            return Err(ExtensionError::InvalidRegistry {
                message: format!("extension '{}' in {origin} has no match fields", ext.id),
            });
        }
    }
    Ok(file.extensions)
}

fn has_match_field(spec: &MatchSpec) -> bool {
    spec.positional_suffix.is_some()
        || spec.filename.is_some()
        || spec.argv_prefix.as_ref().is_some_and(|p| !p.is_empty())
        || spec.arg_suffix.is_some()
}

fn merge_by_id(mut defaults: Vec<ExtensionDef>, project: Vec<ExtensionDef>) -> Vec<ExtensionDef> {
    for ext in project {
        if let Some(index) = defaults.iter().position(|existing| existing.id == ext.id) {
            defaults[index] = ext;
        } else {
            defaults.push(ext);
        }
    }
    defaults
}

#[doc(inline)]
pub use share_resolve::{find_workspace_root, resolve_wyvern_share, resolve_wyvern_share_with};

#[cfg(test)]
mod tests {
    use super::*;

    struct LocalAbsentProbe;

    impl RequiresProbe for LocalAbsentProbe {
        fn binary_on_path(&self, _name: &str) -> bool {
            false
        }
    }

    #[test]
    fn shipped_markdown_suffix_matches_md_path() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let argv = vec!["docs/readme.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        assert_eq!(matched.extension().id.as_str(), "markdown-suffix");
        assert_eq!(matched.path(), Some("docs/readme.md"));
    }

    #[test]
    fn unknown_suffix_does_not_match() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let argv = vec!["notes.txt".to_string()];
        assert!(registry.match_argv(&argv).is_none());
    }

    #[test]
    fn invalid_registry_json_is_structured_error() {
        let err = ExtensionRegistry::from_json_str("{not-json").expect_err("invalid");
        assert!(matches!(err, ExtensionError::InvalidRegistry { .. }));
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn project_override_replaces_same_id() {
        let dir = tempfile::tempdir().expect("tmp");
        let defaults = dir.path().join("defaults.json");
        std::fs::write(&defaults, SHIPPED_EXTENSIONS_JSON).expect("write");
        let project = dir.path().join("project.json");
        std::fs::write(
            &project,
            r#"{
              "version": 1,
              "extensions": [
                {
                  "id": "markdown-suffix",
                  "match": { "positional_suffix": ".markdown" },
                  "expand": { "command": { "type": "markdown", "file": "{path}" } }
                }
              ]
            }"#,
        )
        .expect("write project");
        let registry = ExtensionRegistry::load(&defaults, Some(&project)).expect("load");
        let overridden = registry
            .extensions()
            .iter()
            .find(|ext| ext.id.as_str() == "markdown-suffix")
            .expect("markdown-suffix");
        assert_eq!(overridden.source, SkillSource::Project);
        let shipped_len = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON)
            .expect("shipped")
            .extensions()
            .len();
        assert_eq!(
            registry.extensions().len(),
            shipped_len,
            "project override must replace same id in-place, not append"
        );
        let markdown = registry
            .extensions()
            .iter()
            .find(|ext| ext.id.as_str() == "markdown-suffix")
            .expect("markdown-suffix");
        assert_eq!(
            markdown
                .match_spec
                .positional_suffix
                .as_ref()
                .map(MatchToken::as_str),
            Some(".markdown")
        );
    }

    #[test]
    fn match_with_diagnostics_records_skipped_requires() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "needs-tool",
              "match": { "positional_suffix": ".csv" },
              "preexec": { "cmd": "python3", "requires": ["python3"] },
              "expand": { "command": { "type": "markdown", "content": "x" } }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["sample.csv".into()];
        let outcome = registry.match_with_diagnostics_with(&argv, &LocalAbsentProbe);
        assert!(outcome.matched.is_none());
        assert_eq!(outcome.skipped.len(), 1);
        assert_eq!(outcome.skipped[0].id, "needs-tool");
        assert_eq!(outcome.skipped[0].missing, ["python3"]);
        assert!(registry.match_argv_with(&argv, &LocalAbsentProbe).is_none());
    }

    #[test]
    fn requires_absent_skips_match() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "needs-tool",
              "match": { "argv_prefix": ["compose", "render"] },
              "preexec": { "cmd": "sc-compose", "requires": ["sc-compose"] },
              "expand": { "command": { "type": "markdown", "content": "x" } }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec![
            "compose".into(),
            "render".into(),
            "--root".into(),
            "r".into(),
        ];
        assert!(registry.match_argv_with(&argv, &LocalAbsentProbe).is_none());
        assert_eq!(
            registry.extensions()[0]
                .requires()
                .iter()
                .map(BinaryName::as_str)
                .collect::<Vec<_>>(),
            ["sc-compose"]
        );
    }

    #[test]
    fn extends_reuses_parent_expand() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "parent",
              "match": { "positional_suffix": ".csv" },
              "expand": { "command": { "type": "markdown", "file": "{path}" } }
            },
            {
              "id": "child",
              "extends": "parent",
              "match": { "argv_prefix": ["md"], "arg_suffix": ".csv" }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let child = registry
            .extensions()
            .iter()
            .find(|e| e.id.as_str() == "child")
            .expect("child");
        assert!(child.expand.is_some());
        let argv = vec!["md".into(), "report.csv".into()];
        let matched = registry.match_argv(&argv).expect("match");
        assert!(matches!(matched, ExtensionMatch::PrefixSuffix { .. }));
        assert_eq!(matched.path(), Some("report.csv"));
    }

    #[test]
    fn empty_extension_id_is_invalid() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "   ",
            "match": { "positional_suffix": ".md" },
            "expand": { "command": { "type": "markdown", "file": "{path}" } }
          }]
        }"#;
        let err = ExtensionRegistry::from_json_str(json).expect_err("empty id");
        assert!(matches!(err, ExtensionError::InvalidRegistry { .. }));
        assert!(ExtensionId::try_from(String::from("   ")).is_err());
        assert_eq!(
            ExtensionId::try_from(String::from("markdown-suffix"))
                .expect("valid")
                .as_str(),
            "markdown-suffix"
        );
    }

    #[test]
    fn arg_name_rejects_empty() {
        assert!(ArgName::new("").is_none());
        assert!(ArgName::new("   ").is_none());
        assert!(ArgName::try_from(String::from("   ")).is_err());
        assert_eq!(ArgName::new("root").expect("valid").as_str(), "root");
    }

    #[test]
    fn binary_name_rejects_empty_and_path() {
        assert!(BinaryName::try_from(String::from("   ")).is_err());
        assert!(BinaryName::try_from(String::from("bin/foo")).is_err());
        assert!(BinaryName::try_from(String::from("bin\\foo")).is_err());
        assert_eq!(
            BinaryName::try_from(String::from("sc-compose"))
                .expect("valid")
                .as_str(),
            "sc-compose"
        );
    }

    #[test]
    fn ends_with_suffix_does_not_panic_on_multibyte_token() {
        assert!(ends_with_suffix("café.md", ".md"));
        assert!(ends_with_suffix("café.MD", ".md"));
        assert!(ends_with_suffix("ファイル.md", ".md"));
        // "xé" is 3 bytes; a 2-byte suffix would split `é` under byte slicing.
        assert!(!ends_with_suffix("xé", "xx"));
        assert!(!ends_with_suffix("é", ".md"));
        assert!(!ends_with_suffix("ab", ".md"));
    }

    #[test]
    fn preexec_requires_rejects_empty_and_path() {
        let empty = r#"{
          "version": 1,
          "extensions": [{
            "id": "bad-empty",
            "match": { "positional_suffix": ".md" },
            "preexec": { "cmd": "true", "requires": ["  "] },
            "expand": { "command": { "type": "markdown", "file": "{path}" } }
          }]
        }"#;
        let err = ExtensionRegistry::from_json_str(empty).expect_err("empty requires");
        assert!(
            matches!(err, ExtensionError::InvalidRegistry { .. }),
            "{err}"
        );
        assert!(
            format!("{err}").contains("in memory"),
            "empty-requires error must include origin: {err}"
        );

        let pathish = r#"{
          "version": 1,
          "extensions": [{
            "id": "bad-path",
            "match": { "positional_suffix": ".md" },
            "preexec": { "cmd": "true", "requires": ["bin/foo"] },
            "expand": { "command": { "type": "markdown", "file": "{path}" } }
          }]
        }"#;
        let err = ExtensionRegistry::from_json_str(pathish).expect_err("path requires");
        assert!(
            matches!(err, ExtensionError::InvalidRegistry { .. }),
            "{err}"
        );
        assert!(
            format!("{err}").contains("in memory"),
            "path-requires error must include origin: {err}"
        );
    }

    #[test]
    fn inherit_from_requires_only_keeps_parent_cmd() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "parent",
              "match": { "positional_suffix": ".csv" },
              "preexec": {
                "cmd": "python3",
                "args": ["--parent-flag"],
                "requires": ["python3"]
              },
              "expand": { "command": { "type": "markdown", "file": "{path}" } }
            },
            {
              "id": "child",
              "extends": "parent",
              "match": { "argv_prefix": ["md"], "arg_suffix": ".csv" },
              "preexec": { "requires": ["python3"] }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let child = registry
            .extensions()
            .iter()
            .find(|e| e.id.as_str() == "child")
            .expect("child");
        let pre = child.preexec.as_ref().expect("inherited preexec");
        assert_eq!(pre.cmd, "python3");
        assert_eq!(pre.args, vec!["--parent-flag"]);
        assert_eq!(pre.requires.len(), 1);
        assert_eq!(pre.requires[0].as_str(), "python3");
    }

    #[test]
    fn unknown_host_viewer_field_fails_load() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "bad-host",
            "match": { "positional_suffix": ".md" },
            "expand": {
              "command": { "type": "markdown", "file": "{path}" },
              "host": { "ui_root": ".", "viewer": "none" }
            }
          }]
        }"#;
        let err = ExtensionRegistry::from_json_str(json).expect_err("viewer");
        assert!(
            matches!(err, ExtensionError::InvalidRegistry { .. }),
            "{err}"
        );
        assert!(
            format!("{err}").contains("viewer") || format!("{err}").contains("unknown"),
            "{err}"
        );
    }

    #[test]
    fn help_only_tokens_require_help_flags() {
        assert!(is_help_only_tokens(&["--help".into()]));
        assert!(is_help_only_tokens(&["-h".into()]));
        assert!(is_help_only_tokens(&["--help".into(), "-h".into()]));
        assert!(!is_help_only_tokens(&[]));
        assert!(!is_help_only_tokens(&["--help".into(), "data.csv".into()]));
        assert!(!is_help_only_tokens(&["data.csv".into()]));
    }

    #[test]
    fn match_extension_help_ignores_requires_and_suffix() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let compose = vec!["compose".into(), "render".into(), "--help".into()];
        let ext = match_extension_help(&registry, &compose).expect("compose help");
        assert_eq!(ext.id.as_str(), "compose-render");
        assert!(registry
            .match_argv_with(&compose, &LocalAbsentProbe)
            .is_none());

        let md = vec!["md".into(), "-h".into()];
        let ext = match_extension_help(&registry, &md).expect("md help");
        assert_eq!(ext.id.as_str(), "csv-md");

        let table = vec!["table".into(), "--help".into()];
        let ext = match_extension_help(&registry, &table).expect("table help");
        assert_eq!(ext.id.as_str(), "csv-table-alias");

        let incomplete = vec!["compose".into(), "--help".into()];
        assert!(match_extension_help(&registry, &incomplete).is_none());
        let suffix = vec!["doc.md".into(), "--help".into()];
        let ext = match_extension_help(&registry, &suffix).expect("suffix help");
        assert_eq!(ext.id.as_str(), "markdown-suffix");
        let filename = vec!["path/to/wizard.json".into(), "--help".into()];
        let ext = match_extension_help(&registry, &filename).expect("filename help");
        assert_eq!(ext.id.as_str(), "wizard-json-suffix");
    }
}
