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

mod expand;
mod list;
mod preexec;

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

#[doc(inline)]
pub use expand::{
    build_match_context, expand_and_validate, expand_command_host, expand_preexec_args,
    infer_wizard_root, last_created_tmpdir, relpath_from_ui_root, ExpandedInvocation,
    HostOverrides, MatchContext,
};
#[doc(inline)]
pub use list::{format_extensions_list, run_extensions_command, ExtensionsCmdError};
#[doc(inline)]
pub use preexec::{binary_on_path, create_tmpdir, run_preexec, PathRequiresProbe, RequiresProbe};

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

/// Validated, non-empty extension identifier.
///
/// Constructed via `serde` `try_from` — guaranteed non-empty after trim.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtensionId(String);

impl ExtensionId {
    /// Returns the id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ExtensionId {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("extension id must not be empty or whitespace".into());
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

impl<'de> serde::Deserialize<'de> for ExtensionId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
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
    /// Optional subprocess step before command expand.
    #[serde(default)]
    pub preexec: Option<PreexecSpec>,
    /// Command + host template expansion.
    #[serde(default)]
    pub expand: Option<ExpandSpec>,
}

/// Match fields from the registry schema.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MatchSpec {
    /// Single positional ends with this suffix (`.md`).
    #[serde(default)]
    pub positional_suffix: Option<String>,
    /// Exact basename match (`wizard.json`).
    #[serde(default)]
    pub filename: Option<String>,
    /// First N argv tokens (`["compose", "render"]`).
    #[serde(default)]
    pub argv_prefix: Option<Vec<String>>,
    /// Token after prefix matches this suffix.
    #[serde(default)]
    pub arg_suffix: Option<String>,
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
pub struct PreexecSpec {
    /// Executable name or path (phase-1 expanded).
    #[serde(default)]
    pub cmd: String,
    /// Argv tokens (phase-1 expanded; `{arg:name:repeat}` splices).
    #[serde(default)]
    pub args: Vec<String>,
    /// Binaries that must be on `PATH` or the extension does not match.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Stdout capture mode (`markdown` only in Phase F).
    #[serde(default)]
    pub stdout: Option<StdoutCapture>,
}

/// Expand templates for command JSON and host overrides.
#[derive(Debug, Clone, Default, Deserialize)]
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
pub struct HostExpandSpec {
    /// Template for [`HostOverrides::ui_root`].
    #[serde(default)]
    pub ui_root: Option<String>,
}

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

/// Structured extension-engine failure.
#[derive(Debug)]
pub enum ExtensionError {
    /// Registry JSON or schema is invalid.
    InvalidRegistry {
        /// Human-readable load failure.
        message: String,
    },
    /// Required `{arg:name}` flag was missing.
    MissingArg {
        /// Flag name without leading dashes.
        name: String,
    },
    /// Unexpected token after a successful prefix match.
    UnexpectedArg {
        /// Offending token.
        token: String,
    },
    /// Path-derived template used without a matched path.
    PathVarWithoutPath {
        /// Template variable name.
        var: String,
    },
    /// Template substitution failed.
    Template {
        /// Substitution failure detail.
        message: String,
    },
    /// Preexec process failed or could not be spawned.
    Preexec {
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
            Self::MissingArg { name } => write!(f, "missing required extension argument --{name}"),
            Self::UnexpectedArg { token } => {
                write!(f, "unexpected argument after extension match: {token}")
            }
            Self::PathVarWithoutPath { var } => {
                write!(f, "template {{{var}}} requires a matched file path")
            }
            Self::Template { message } => write!(f, "extension template error: {message}"),
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
            Self::MissingArg { .. }
            | Self::UnexpectedArg { .. }
            | Self::PathVarWithoutPath { .. }
            | Self::Template { .. } => wyvern_schema::ErrorCode::ValidationError.exit_code(),
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
        } else {
            parse_registry_str(SHIPPED_EXTENSIONS_JSON, "shipped defaults")?
        };
        let project_exts = match project {
            Some(path) if path.is_file() => parse_registry_file(path)?,
            _ => Vec::new(),
        };
        let merged = merge_by_id(default_exts, project_exts);
        let extensions = apply_extends(merged)?;
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
        let extensions = apply_extends(parse_registry_str(json, "memory")?)?;
        Ok(Self { extensions })
    }

    /// Walk merged extensions in order; first match wins.
    ///
    /// After `extends` resolution, an extension whose `preexec.requires` binaries
    /// are absent on `PATH` does not match (fallthrough).
    #[must_use]
    pub fn match_argv<'a>(&'a self, argv: &'a [String]) -> Option<ExtensionMatch<'a>> {
        self.match_argv_with(argv, &PathRequiresProbe)
    }

    /// [`Self::match_argv`] with an injectable [`RequiresProbe`].
    #[must_use]
    pub fn match_argv_with<'a>(
        &'a self,
        argv: &'a [String],
        probe: &dyn RequiresProbe,
    ) -> Option<ExtensionMatch<'a>> {
        self.extensions
            .iter()
            .find_map(|ext| ext.match_argv(argv, probe))
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
    pub fn requires(&self) -> &[String] {
        self.preexec
            .as_ref()
            .map(|p| p.requires.as_slice())
            .unwrap_or(&[])
    }

    fn match_argv<'a>(
        &'a self,
        argv: &'a [String],
        probe: &dyn RequiresProbe,
    ) -> Option<ExtensionMatch<'a>> {
        if !self.requires().iter().all(|bin| probe.binary_on_path(bin)) {
            return None;
        }
        let spec = &self.match_spec;
        if let Some(prefix) = &spec.argv_prefix {
            if argv.len() < prefix.len() || argv[..prefix.len()] != prefix[..] {
                return None;
            }
            let rest = &argv[prefix.len()..];
            if let Some(suffix) = &spec.arg_suffix {
                let path = rest.iter().find(|token| ends_with_suffix(token, suffix))?;
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
            if base == filename {
                return Some(ExtensionMatch::Suffix {
                    ext: self,
                    path: token,
                });
            }
            return None;
        }
        if let Some(suffix) = &spec.positional_suffix {
            if ends_with_suffix(token, suffix) {
                return Some(ExtensionMatch::Suffix {
                    ext: self,
                    path: token,
                });
            }
        }
        None
    }
}

fn ends_with_suffix(token: &str, suffix: &str) -> bool {
    token
        .to_ascii_lowercase()
        .ends_with(&suffix.to_ascii_lowercase())
}

fn parse_registry_file(path: &Path) -> Result<Vec<ExtensionDef>, ExtensionError> {
    let text = std::fs::read_to_string(path).map_err(|err| ExtensionError::Io {
        message: format!("could not read '{}': {err}", path.display()),
        source: Some(Box::new(err)),
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
        if let Some(preexec) = &ext.preexec {
            for bin in &preexec.requires {
                let t = bin.trim();
                if t.is_empty() {
                    return Err(ExtensionError::InvalidRegistry {
                        message: "preexec.requires: binary name must not be empty".into(),
                    });
                }
                if t.contains('/') || t.contains('\\') {
                    return Err(ExtensionError::InvalidRegistry {
                        message: format!(
                            "preexec.requires: '{t}' looks like a path; use bare binary name"
                        ),
                    });
                }
            }
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

fn apply_extends(exts: Vec<ExtensionDef>) -> Result<Vec<ExtensionDef>, ExtensionError> {
    let mut resolved = Vec::with_capacity(exts.len());
    for ext in &exts {
        resolved.push(resolve_one(ext, &exts, &mut Vec::new())?);
    }
    Ok(resolved)
}

fn resolve_one(
    ext: &ExtensionDef,
    all: &[ExtensionDef],
    stack: &mut Vec<ExtensionId>,
) -> Result<ExtensionDef, ExtensionError> {
    let Some(ref parent_id) = ext.extends else {
        return Ok(ext.clone());
    };
    if stack.iter().any(|id| id == &ext.id) {
        return Err(ExtensionError::InvalidRegistry {
            message: format!("circular extends involving '{}'", ext.id),
        });
    }
    stack.push(ext.id.clone());
    let parent = all
        .iter()
        .find(|candidate| candidate.id == *parent_id)
        .ok_or_else(|| ExtensionError::InvalidRegistry {
            message: format!("extension '{}' extends unknown id '{parent_id}'", ext.id),
        })?;
    let parent = resolve_one(parent, all, stack)?;
    stack.pop();
    Ok(inherit_from(ext, &parent))
}

fn inherit_from(child: &ExtensionDef, parent: &ExtensionDef) -> ExtensionDef {
    let preexec = match (&child.preexec, &parent.preexec) {
        (None, parent_pre) => parent_pre.clone(),
        (Some(child_pre), Some(parent_pre)) => {
            let mut merged = child_pre.clone();
            if merged.requires.is_empty() {
                merged.requires.clone_from(&parent_pre.requires);
            }
            Some(merged)
        }
        (Some(child_pre), None) => Some(child_pre.clone()),
    };
    ExtensionDef {
        id: child.id.clone(),
        match_spec: child.match_spec.clone(),
        extends: child.extends.clone(),
        preexec,
        expand: child.expand.clone().or_else(|| parent.expand.clone()),
    }
}

/// Resolve `{wyvern_share}` to a unified directory (dev workspace or embed).
///
/// Layout: `extensions.json`, `scripts/ext/*`, `ext/csv/**`.
#[must_use]
pub fn resolve_wyvern_share() -> PathBuf {
    resolve_wyvern_share_with(
        std::env::var("WYVERN_SHARE").ok().as_deref(),
        std::env::current_dir().ok().as_deref(),
        std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(Path::parent),
        Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        true,
    )
}

/// Resolve `{wyvern_share}` from injectable inputs (no process-global mutation).
#[must_use]
pub fn resolve_wyvern_share_with(
    share_var: Option<&str>,
    cwd: Option<&Path>,
    exe_dir: Option<&Path>,
    manifest_dir: Option<&Path>,
    use_embedded: bool,
) -> PathBuf {
    if let Some(path) = share_var {
        tracing::debug!(path, "resolve_wyvern_share: using WYVERN_SHARE override");
        return PathBuf::from(path);
    }
    for start in [cwd, exe_dir, manifest_dir].into_iter().flatten() {
        if let Some(workspace) = find_workspace_root(start) {
            if let Some(unified) = materialize_workspace_share(&workspace) {
                tracing::debug!(
                    path = %unified.display(),
                    "resolve_wyvern_share: materialized workspace share"
                );
                return unified;
            }
            tracing::debug!(
                workspace = %workspace.display(),
                "resolve_wyvern_share: workspace found but materialize failed; trying next start"
            );
        }
    }
    if use_embedded {
        if let Some(extracted) = extract_embedded_share() {
            tracing::debug!(
                path = %extracted.display(),
                "resolve_wyvern_share: using embedded extract"
            );
            return extracted;
        }
        tracing::debug!("resolve_wyvern_share: embedded extract failed; using relative fallback");
    }
    tracing::debug!("resolve_wyvern_share: falling back to share/wyvern");
    PathBuf::from("share/wyvern")
}

/// Walk `start` and parents looking for `share/wyvern/extensions.json`.
#[must_use]
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join("share/wyvern/extensions.json").is_file() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn materialize_workspace_share(workspace: &Path) -> Option<PathBuf> {
    // Use pid-suffixed dir to avoid races between concurrent test processes.
    let dest = workspace.join(format!("target/wyvern-share-{}", std::process::id()));
    let marker = dest.join("extensions.json");
    if file_nonempty(&marker) {
        return Some(dest);
    }
    copy_dir_contents(&workspace.join("share/wyvern"), &dest)?;
    let scripts_src = workspace.join("scripts/ext");
    if scripts_src.is_dir() {
        copy_dir_contents(&scripts_src, &dest.join("scripts/ext"))?;
    }
    file_nonempty(&dest.join("extensions.json")).then_some(dest)
}

fn file_nonempty(path: &Path) -> bool {
    path.is_file() && path.metadata().is_ok_and(|m| m.len() > 0)
}

fn copy_dir_contents(src: &Path, dest: &Path) -> Option<()> {
    if !src.is_dir() {
        return Some(());
    }
    if let Err(e) = std::fs::create_dir_all(dest) {
        tracing::warn!(
            "copy_dir_contents: failed to copy {} to {}: {e}",
            src.display(),
            dest.display()
        );
        return None;
    }
    let entries = match std::fs::read_dir(src) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!(
                "copy_dir_contents: failed to copy {} to {}: {e}",
                src.display(),
                dest.display()
            );
            return None;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                tracing::warn!(
                    "copy_dir_contents: failed to copy {} to {}: {e}",
                    src.display(),
                    dest.display()
                );
                return None;
            }
        };
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir_contents(&from, &to)?;
        } else {
            if file_nonempty(&to) {
                continue;
            }
            copy_file_replace(&from, &to)?;
        }
    }
    Some(())
}

fn copy_file_replace(from: &Path, to: &Path) -> Option<()> {
    let tmp = to.with_file_name(format!(
        ".{}.part-{}",
        to.file_name()?.to_string_lossy(),
        std::process::id()
    ));
    std::fs::copy(from, &tmp).ok()?;
    match std::fs::rename(&tmp, to) {
        Ok(()) => Some(()),
        Err(_) => {
            let _ = std::fs::remove_file(&tmp);
            file_nonempty(to).then_some(())
        }
    }
}

fn write_embedded_atomically(out: &Path, data: &[u8]) -> Option<()> {
    if let Some(parent) = out.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
                out.display()
            );
            return None;
        }
    }
    let tmp_out = out.with_file_name(format!(
        ".{}.part-{}",
        out.file_name()?.to_string_lossy(),
        std::process::id()
    ));
    if let Err(e) = std::fs::write(&tmp_out, data) {
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            out.display()
        );
        return None;
    }
    if let Err(e) = std::fs::rename(&tmp_out, out) {
        let _ = std::fs::remove_file(&tmp_out);
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            out.display()
        );
        return None;
    }
    Some(())
}

fn extract_embedded_share() -> Option<PathBuf> {
    let dest = dirs::cache_dir()?
        .join("wyvern")
        .join(env!("CARGO_PKG_VERSION"))
        .join("share");
    if let Err(e) = std::fs::create_dir_all(&dest) {
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            dest.display()
        );
        return None;
    }
    for path in ShareAssets::iter() {
        let out = dest.join(path.as_ref());
        let file = ShareAssets::get(path.as_ref())?;
        write_embedded_atomically(&out, file.data.as_ref())?;
    }
    let scripts_dest = dest.join("scripts/ext");
    if let Err(e) = std::fs::create_dir_all(&scripts_dest) {
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            scripts_dest.display()
        );
        return None;
    }
    for path in ScriptAssets::iter() {
        let out = scripts_dest.join(path.as_ref());
        let file = ScriptAssets::get(path.as_ref())?;
        write_embedded_atomically(&out, file.data.as_ref())?;
    }
    dest.join("extensions.json").is_file().then_some(dest)
}

/// Short match-kind summary for `wyvern extensions list`.
#[must_use]
pub fn match_kind_summary(spec: &MatchSpec) -> String {
    if let Some(prefix) = &spec.argv_prefix {
        let prefix_s = prefix.join(" ");
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

#[cfg(test)]
mod tests {
    use super::*;

    struct AbsentProbe;

    impl RequiresProbe for AbsentProbe {
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
        assert_eq!(registry.extensions().len(), 4);
        let markdown = registry
            .extensions()
            .iter()
            .find(|ext| ext.id.as_str() == "markdown-suffix")
            .expect("markdown-suffix");
        assert_eq!(
            markdown.match_spec.positional_suffix.as_deref(),
            Some(".markdown")
        );
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
        assert!(registry.match_argv_with(&argv, &AbsentProbe).is_none());
        assert_eq!(registry.extensions()[0].requires(), ["sc-compose"]);
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
    }
}
