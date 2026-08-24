//! `wyvern wizard lint` — static lint analysis for wizard packages.
//!
//! Walks a wizard package (directory containing `wizard.json` or a path to
//! `wizard.json` directly), builds the reachable page graph via static analysis
//! of linked local JavaScript, and applies the normative nav-button rules from
//! `ui/shared/wizard-nav.js` sprint d.7.
//!
//! # Exit codes
//!
//! | Code | Meaning                       |
//! |------|-------------------------------|
//! |  0   | All checked pages are clean   |
//! |  1   | One or more findings reported |
//! |  2   | Usage / bad arguments         |

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::path::{Component, Path, PathBuf};

use wyvern_schema::{FieldName, WizardPageFieldError, WizardPageHtml, WizardPageId};
use wyvern_wizard::{
    add_edge, extract_local_script_srcs, extract_next_hops, extract_next_wizard_refs,
    lint_dataflow, lint_page, merge_html_dataflow_overlay, parse_dataflow_from_json,
    DataflowLintInput, DataflowSpec, GraphPage, LintFinding, PageInfo, PageRole, WizardPageGraph,
};

use crate::error::{BuiltinDomain, EmitError, UsageErrorKind};
use crate::extensions::resolve_wyvern_share;
use crate::workflow::Allowlist;

// ── Public API ────────────────────────────────────────────────────────────────

/// Result of a successful `wizard lint` invocation.
#[derive(Debug)]
pub enum WizardCmdResult {
    /// All checked pages produced no findings. Carry the stdout summary.
    Clean(String),
    /// One or more findings were found. Carry the full report for stdout.
    Findings(String),
}

/// Failure from `wyvern wizard …`.
#[derive(Debug)]
pub enum WizardCmdError {
    /// Bad argv (exit 2).
    Usage {
        /// Discriminated usage class for structured stderr recovery.
        kind: UsageErrorKind,
        /// Plain-text usage message.
        message: String,
    },
    /// I/O, parse, or field-validation failure — emit via [`emit_wizard_lint_stage_error`].
    Stage(WizardLintStageError),
    /// Emit-boundary serialize failure.
    Emit(EmitError),
}

/// Structured `wizard lint` stage failure (RBP-001 / RBP-F001).
///
/// Carries path and field context so [`emit_wizard_lint_stage_error`] can map
/// each variant to `IoError` / `ParseError` / `ValidationError` with a stable
/// subcode and recovery steps.
#[derive(Debug)]
pub enum WizardLintStageError {
    /// Filesystem read or path resolution failed.
    Io {
        /// Path that could not be read or resolved.
        path: PathBuf,
        /// Human-readable detail.
        message: String,
    },
    /// `wizard.json` was not valid JSON.
    Parse {
        /// `wizard.json` path.
        path: PathBuf,
        /// Parser detail.
        message: String,
    },
    /// `wizard.json` fields failed newtype or shape checks.
    Validation {
        /// `wizard.json` path.
        path: PathBuf,
        /// Field that failed (e.g. `page.id`).
        field: FieldName,
        /// Human-readable detail, including [`WizardPageFieldError`] when applicable.
        message: String,
    },
}

impl fmt::Display for WizardLintStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for WizardLintStageError {}

impl WizardLintStageError {
    /// Human-readable detail for stdout / tests.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Io { message, .. }
            | Self::Parse { message, .. }
            | Self::Validation { message, .. } => message,
        }
    }

    /// Process exit code for `wyvern wizard lint` stage failures (always 1).
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        1
    }

    /// Stable sub-discriminator under the mapped [`wyvern_schema::ErrorCode`].
    #[must_use]
    pub const fn subcode(&self) -> &'static str {
        match self {
            Self::Io { .. } => "wizard_lint_io",
            Self::Parse { .. } => "wizard_lint_parse",
            Self::Validation { .. } => "wizard_lint_validation",
        }
    }
}

fn combine_stage_errors(mut errors: Vec<WizardLintStageError>) -> WizardLintStageError {
    if errors.len() == 1 {
        return errors.remove(0);
    }
    let combined = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    match errors.remove(0) {
        WizardLintStageError::Io { path, .. } => WizardLintStageError::Io {
            path,
            message: combined,
        },
        WizardLintStageError::Parse { path, .. } => WizardLintStageError::Parse {
            path,
            message: combined,
        },
        WizardLintStageError::Validation { path, field, .. } => WizardLintStageError::Validation {
            path,
            field,
            message: combined,
        },
    }
}

/// Usage text for `wyvern wizard --help` / `wyvern wizard lint --help`.
#[must_use]
pub fn wizard_usage_message() -> String {
    concat!(
        "Usage: wyvern wizard lint <path> [<path>...]\n",
        "       wyvern wizard lint --help\n",
        "\n",
        "Commands:\n",
        "  lint <path...>   Lint one or more wizard packages for missing nav buttons\n",
        "\n",
        "Arguments:\n",
        "  <path>   Directory containing wizard.json, or path to wizard.json itself\n",
        "\n",
        "Exit codes:\n",
        "  0   All pages are clean\n",
        "  1   One or more findings\n",
        "  2   Usage / bad arguments\n",
        "\n",
        "Findings codes:\n",
        "  WIZARD-LINT-001  Non-entry page missing back button\n",
        "  WIZARD-LINT-002  Terminal page missing cancel button\n",
        "  WIZARD-LINT-003  wizard-nav.js chrome opt-in but no nav region\n",
        "  WIZARD-LINT-004  Non-terminal page (with chrome opt-in) missing next button\n",
        "  WIZARD-LINT-005  config.dataflow requires unsatisfied or type conflict\n",
        "  WIZARD-LINT-006  Terminal post_input not covered by exports\n",
        "  WIZARD-LINT-007  next_wizard input keys undeclared on target\n",
        "  WIZARD-LINT-008  Local JS reads a key no page exports\n",
        "\n",
        "See also: wyvern --help\n",
    )
    .to_string()
}

/// Run `wyvern wizard …`; returns a [`WizardCmdResult`] on success.
///
/// # Errors
///
/// Returns [`WizardCmdError`] for bad argv or I/O failures.
pub fn run_wizard_command(args: &[String]) -> Result<WizardCmdResult, WizardCmdError> {
    if args
        .first()
        .is_some_and(|t| t == "--help" || t == "-h" || t == "help")
    {
        return Ok(WizardCmdResult::Clean(wizard_usage_message()));
    }

    match args.first().map(String::as_str) {
        Some("lint") => run_lint(&args[1..]),
        Some("--help") | Some("-h") | Some("help") | None => {
            // Handled above; this arm is unreachable but keeps match exhaustive.
            Ok(WizardCmdResult::Clean(wizard_usage_message()))
        }
        Some(other) => Err(WizardCmdError::Usage {
            kind: UsageErrorKind::UnknownSubcommand {
                domain: BuiltinDomain::Wizard,
                token: other.to_string(),
            },
            message: format!(
                "unknown wizard subcommand '{other}'\n{}",
                wizard_usage_message()
            ),
        }),
    }
}

// ── lint subcommand ───────────────────────────────────────────────────────────

fn run_lint(args: &[String]) -> Result<WizardCmdResult, WizardCmdError> {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        return Ok(WizardCmdResult::Clean(wizard_usage_message()));
    }

    if args.is_empty() {
        return Err(WizardCmdError::Usage {
            kind: UsageErrorKind::Generic,
            message: format!(
                "wyvern wizard lint requires at least one <path> argument\n{}",
                wizard_usage_message()
            ),
        });
    }

    let mut all_findings: Vec<LintFinding> = Vec::new();
    let mut errors: Vec<WizardLintStageError> = Vec::new();
    let mut total_pages: usize = 0;

    for path_str in args {
        match lint_package(path_str) {
            Ok((findings, page_count)) => {
                total_pages += page_count;
                all_findings.extend(findings);
            }
            Err(err) => {
                errors.push(err);
            }
        }
    }

    // I/O / parse / validation errors are reported on stderr and set exit 1.
    if !errors.is_empty() {
        return Err(WizardCmdError::Stage(combine_stage_errors(errors)));
    }

    if all_findings.is_empty() {
        let noun = if total_pages == 1 { "page" } else { "pages" };
        let checked = if args.len() == 1 {
            format!("Checked {total_pages} {noun} — no findings.\n")
        } else {
            format!(
                "Checked {} package(s), {total_pages} {noun} total — no findings.\n",
                args.len()
            )
        };
        return Ok(WizardCmdResult::Clean(checked));
    }

    let report = format_findings(&all_findings, args.len(), total_pages);
    Ok(WizardCmdResult::Findings(report))
}

// ── Package analysis ──────────────────────────────────────────────────────────

/// Lint a single wizard package rooted at `path_str`.
///
/// Returns `(findings, page_count)` on success, or a structured stage error.
fn lint_package(path_str: &str) -> Result<(Vec<LintFinding>, usize), WizardLintStageError> {
    let input = Path::new(path_str);

    // Resolve wizard.json and wizard root directory.
    let (wizard_json_path, wizard_dir) = resolve_wizard_paths(input)?;

    // Parse wizard.json to get the entry page descriptor.
    let json_content =
        std::fs::read_to_string(&wizard_json_path).map_err(|e| WizardLintStageError::Io {
            path: wizard_json_path.clone(),
            message: format!("error: cannot read '{}': {e}", wizard_json_path.display()),
        })?;

    let (entry_id, entry_html) = parse_wizard_json_entry(&json_content, &wizard_json_path)?;

    // BFS to build the reachable page graph.
    let (pages, graph) = build_page_graph(&wizard_dir, &entry_id, &entry_html)?;

    // Nav lint for each page. Borrow newtypes as `&str` only at the wyvern_wizard boundary.
    let mut findings: Vec<LintFinding> = pages
        .iter()
        .flat_map(|p| {
            lint_page(&PageInfo {
                id: p.id.as_str(),
                file: p.rel_path.as_str(),
                html: &p.html,
                role: p.role,
            })
        })
        .collect();

    // Dataflow lint when config.dataflow is declared.
    if let Some(mut spec) = parse_dataflow_from_json(&json_content) {
        for page in &pages {
            merge_html_dataflow_overlay(&mut spec, page.id.as_str(), &page.html);
        }
        let js_files = collect_local_js_files(&wizard_dir, &pages);
        let has_workflow_post = wizard_json_has_workflow_post(&json_content);
        let next_wizard_targets = build_next_wizard_targets(&wizard_dir, &js_files);
        findings.extend(lint_dataflow(&DataflowLintInput {
            spec: &spec,
            graph: &graph,
            js_files: &js_files,
            has_workflow_post,
            next_wizard_targets: &next_wizard_targets,
        }));
    }

    let page_count = pages.len();
    Ok((findings, page_count))
}

/// Resolve `input` to `(wizard_json_path, wizard_dir)`.
fn resolve_wizard_paths(input: &Path) -> Result<(PathBuf, PathBuf), WizardLintStageError> {
    if input.is_dir() {
        let json = input.join("wizard.json");
        if json.is_file() {
            return Ok((json, input.to_path_buf()));
        }
        return Err(WizardLintStageError::Io {
            path: json,
            message: format!(
                "error: '{}' is a directory but contains no wizard.json",
                input.display()
            ),
        });
    }
    if input.is_file() {
        let dir = input.parent().unwrap_or(Path::new("."));
        return Ok((input.to_path_buf(), dir.to_path_buf()));
    }
    Err(WizardLintStageError::Io {
        path: input.to_path_buf(),
        message: format!(
            "error: '{}' not found (expected directory or wizard.json path)",
            input.display()
        ),
    })
}

/// Extract typed `(entry_id, entry_html)` from wizard.json content.
///
/// Keeps [`WizardPageId`] / [`WizardPageHtml`] so callers do not strip newtypes
/// at the parse boundary (RBP-004 / RBP-F003).
fn parse_wizard_json_entry(
    json: &str,
    path: &Path,
) -> Result<(WizardPageId, WizardPageHtml), WizardLintStageError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| WizardLintStageError::Parse {
            path: path.to_path_buf(),
            message: format!("error: '{}': invalid JSON: {e}", path.display()),
        })?;

    let page = value
        .get("page")
        .ok_or_else(|| WizardLintStageError::Validation {
            path: path.to_path_buf(),
            field: FieldName::new("page"),
            message: format!(
                "error: '{}': missing 'page' field in wizard.json",
                path.display()
            ),
        })?;

    let id = parse_page_newtype_field(page, path, "id", WizardPageId::try_new)?;
    let html = parse_page_newtype_field(page, path, "html", WizardPageHtml::try_new)?;

    Ok((id, html))
}

fn parse_page_newtype_field<T>(
    page: &serde_json::Value,
    path: &Path,
    field: &str,
    ctor: impl FnOnce(String) -> Result<T, WizardPageFieldError>,
) -> Result<T, WizardLintStageError> {
    let field_name = format!("page.{field}");
    let raw = page.get(field).and_then(|v| v.as_str()).ok_or_else(|| {
        WizardLintStageError::Validation {
            path: path.to_path_buf(),
            field: FieldName::new(field_name.clone()),
            message: format!(
                "error: '{}': wizard.json {field_name} must be a string",
                path.display()
            ),
        }
    })?;
    ctor(raw.to_string()).map_err(|err| WizardLintStageError::Validation {
        path: path.to_path_buf(),
        field: FieldName::new(field_name.clone()),
        message: format!(
            "error: '{}': wizard.json {field_name}: {err}",
            path.display()
        ),
    })
}

// ── BFS page graph ────────────────────────────────────────────────────────────

struct PageNode {
    id: WizardPageId,
    /// Path relative to the wizard root (for display).
    rel_path: WizardPageHtml,
    html: String,
    role: PageRole,
}

/// Build the reachable page graph via BFS, starting from the entry page.
///
/// Keeps [`WizardPageId`] / [`WizardPageHtml`] on [`PageNode`] and the BFS
/// queue. String keys are produced with [`WizardPageId::as_str`] /
/// [`WizardPageHtml::as_str`] only when filling [`WizardPageGraph`]
/// (wyvern_wizard API boundary).
fn build_page_graph(
    wizard_dir: &Path,
    entry_id: &WizardPageId,
    entry_html: &WizardPageHtml,
) -> Result<(Vec<PageNode>, WizardPageGraph), WizardLintStageError> {
    let mut visited: HashSet<WizardPageHtml> = HashSet::new();
    let mut queue: VecDeque<(WizardPageId, WizardPageHtml, PageRole, Option<WizardPageId>)> =
        VecDeque::new();
    let mut pages: Vec<PageNode> = Vec::new();
    let mut graph = WizardPageGraph {
        entry_id: entry_id.as_str().to_string(),
        pages: HashMap::new(),
        edges: HashMap::new(),
    };

    queue.push_back((entry_id.clone(), entry_html.clone(), PageRole::Entry, None));

    while let Some((id, html_rel, role, from_id)) = queue.pop_front() {
        if visited.contains(&html_rel) {
            continue;
        }
        visited.insert(html_rel.clone());

        if let Some(from) = from_id {
            add_edge(&mut graph, from.as_str(), id.as_str());
        }

        let html_abs = wizard_dir.join(html_rel.as_str());
        let html_content =
            std::fs::read_to_string(&html_abs).map_err(|e| WizardLintStageError::Io {
                path: html_abs.clone(),
                message: format!("error: cannot read page '{}': {e}", html_abs.display()),
            })?;

        // Discover next-page hops from linked local JS files.
        let html_dir = html_abs.parent().unwrap_or(wizard_dir);
        let srcs = extract_local_script_srcs(&html_content);
        for src in srcs {
            let script_abs = normalize_path(&html_dir.join(&src));
            let Ok(js) = std::fs::read_to_string(&script_abs) else {
                continue;
            };
            for hop in extract_next_hops(&js) {
                let Ok(hop_html) = WizardPageHtml::try_new(hop.html) else {
                    continue;
                };
                if !visited.contains(&hop_html) {
                    let Ok(hop_id) = WizardPageId::try_new(hop.id) else {
                        continue;
                    };
                    queue.push_back((hop_id, hop_html, PageRole::Inner, Some(id.clone())));
                }
            }
        }

        graph.pages.insert(
            id.as_str().to_string(),
            GraphPage {
                id: id.as_str().to_string(),
                file: html_rel.as_str().to_string(),
                html: html_content.clone(),
            },
        );

        pages.push(PageNode {
            id,
            rel_path: html_rel,
            html: html_content,
            role,
        });
    }

    Ok((pages, graph))
}

fn build_next_wizard_targets(
    wizard_dir: &Path,
    js_files: &HashMap<String, String>,
) -> HashMap<String, DataflowSpec> {
    let allowlist = Allowlist {
        share_root: resolve_wyvern_share(),
        cwd: std::env::current_dir().unwrap_or_else(|_| wizard_dir.to_path_buf()),
        wizard_dir: wizard_dir.to_path_buf(),
    };

    let mut paths = HashSet::new();
    for (file, js) in js_files {
        for nw in extract_next_wizard_refs(js, file) {
            paths.insert(nw.path);
        }
    }

    let mut targets = HashMap::new();
    for path in paths {
        let Ok(wizard_json) = allowlist.resolve_allowed(&path) else {
            continue;
        };
        let Ok(json) = std::fs::read_to_string(&wizard_json) else {
            continue;
        };
        if let Some(spec) = parse_dataflow_from_json(&json) {
            targets.insert(path, spec);
        }
    }
    targets
}

fn collect_local_js_files(wizard_dir: &Path, pages: &[PageNode]) -> HashMap<String, String> {
    let mut files: HashMap<String, String> = HashMap::new();
    for page in pages {
        let html_abs = wizard_dir.join(page.rel_path.as_str());
        let html_dir = html_abs.parent().unwrap_or(wizard_dir);
        let srcs = extract_local_script_srcs(&page.html);
        for src in srcs {
            let script_abs = normalize_path(&html_dir.join(&src));
            let Ok(rel) = script_abs.strip_prefix(wizard_dir) else {
                continue;
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if files.contains_key(&rel_str) {
                continue;
            }
            if let Ok(js) = std::fs::read_to_string(&script_abs) {
                files.insert(rel_str, js);
            }
        }
    }
    files
}

fn wizard_json_has_workflow_post(json: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return false;
    };
    value
        .get("workflow")
        .and_then(|w| w.get("post"))
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Normalize path components by resolving `..` without touching the filesystem.
fn normalize_path(path: &Path) -> PathBuf {
    let mut out: Vec<Component<'_>> = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // Only pop a non-root normal component.
                match out.last() {
                    Some(Component::Normal(_)) => {
                        out.pop();
                    }
                    _ => out.push(component),
                }
            }
            c => out.push(c),
        }
    }
    out.iter().collect()
}

// ── Formatting ────────────────────────────────────────────────────────────────

fn format_findings(findings: &[LintFinding], pkg_count: usize, page_count: usize) -> String {
    let mut out = String::new();
    for f in findings {
        out.push_str(&f.display_line());
        out.push('\n');
    }
    let pkg_noun = if pkg_count == 1 {
        "package"
    } else {
        "packages"
    };
    let page_noun = if page_count == 1 { "page" } else { "pages" };
    let finding_noun = if findings.len() == 1 {
        "finding"
    } else {
        "findings"
    };
    out.push_str(&format!(
        "\n{} {} in {} {} ({} {} checked)\n",
        findings.len(),
        finding_noun,
        pkg_count,
        pkg_noun,
        page_count,
        page_noun,
    ));
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_help_flag_returns_usage() {
        let result = run_wizard_command(&["--help".into()]).expect("ok");
        match result {
            WizardCmdResult::Clean(text) => {
                assert!(text.contains("wyvern wizard lint"), "{text}");
                assert!(text.contains("WIZARD-LINT-001"), "{text}");
            }
            WizardCmdResult::Findings(_) => panic!("expected clean / usage"),
        }
    }

    #[test]
    fn wizard_lint_help_returns_usage() {
        let result = run_wizard_command(&["lint".into(), "--help".into()]).expect("ok");
        match result {
            WizardCmdResult::Clean(text) => {
                assert!(text.contains("wyvern wizard lint"), "{text}");
            }
            WizardCmdResult::Findings(_) => panic!("expected clean"),
        }
    }

    #[test]
    fn unknown_wizard_subcommand_is_usage_error() {
        let err = run_wizard_command(&["dump".into()]).expect_err("should err");
        match err {
            WizardCmdError::Usage { kind, message } => {
                assert!(
                    matches!(
                        kind,
                        UsageErrorKind::UnknownSubcommand { domain, ref token }
                            if domain == BuiltinDomain::Wizard && token == "dump"
                    ),
                    "{kind:?}"
                );
                assert!(message.contains("unknown wizard subcommand"), "{message}");
            }
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn wizard_lint_no_args_is_generic_usage_error() {
        let err = run_wizard_command(&["lint".into()]).expect_err("should err");
        match err {
            WizardCmdError::Usage { kind, message } => {
                assert_eq!(kind, UsageErrorKind::Generic);
                assert!(message.contains("requires at least one"), "{message}");
            }
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn wizard_usage_message_mentions_lint_codes() {
        let text = wizard_usage_message();
        assert!(text.contains("WIZARD-LINT-001"), "{text}");
        assert!(text.contains("WIZARD-LINT-002"), "{text}");
        assert!(text.contains("WIZARD-LINT-003"), "{text}");
        assert!(text.contains("WIZARD-LINT-004"), "{text}");
        assert!(text.contains("WIZARD-LINT-005"), "{text}");
        assert!(text.contains("WIZARD-LINT-008"), "{text}");
    }

    #[test]
    fn parse_wizard_json_entry_keeps_page_newtypes() {
        let json = r#"{"page":{"id":"start","html":"pages/start.html"}}"#;
        let (id, html) = parse_wizard_json_entry(json, Path::new("wizard.json")).expect("ok");
        assert_eq!(id.as_str(), "start");
        assert_eq!(html.as_str(), "pages/start.html");
    }

    #[test]
    fn parse_wizard_json_entry_preserves_field_error_detail() {
        let json = r#"{"page":{"id":"","html":"pages/start.html"}}"#;
        let err = parse_wizard_json_entry(json, Path::new("wizard.json")).expect_err("empty id");
        match err {
            WizardLintStageError::Validation { field, message, .. } => {
                assert_eq!(field.as_str(), "page.id");
                assert!(
                    message.contains("wizard page field must be a non-empty string"),
                    "{message}"
                );
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn parse_wizard_json_entry_invalid_json_is_parse() {
        let err = parse_wizard_json_entry("{", Path::new("wizard.json")).expect_err("parse");
        assert!(matches!(err, WizardLintStageError::Parse { .. }), "{err:?}");
    }

    #[test]
    fn lint_missing_path_is_io_stage_error() {
        let err =
            run_wizard_command(&["lint".into(), "/no/such/wizard-pkg".into()]).expect_err("io");
        match err {
            WizardCmdError::Stage(stage) => {
                assert!(
                    matches!(stage, WizardLintStageError::Io { .. }),
                    "{stage:?}"
                );
                assert!(stage.message().contains("not found"), "{}", stage.message());
                assert_eq!(stage.exit_code(), 1);
                assert_eq!(stage.subcode(), "wizard_lint_io");
            }
            other => panic!("expected Stage, got {other:?}"),
        }
    }
}
