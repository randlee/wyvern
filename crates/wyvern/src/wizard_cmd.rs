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
use std::path::{Component, Path, PathBuf};

use wyvern_schema::{WizardPageHtml, WizardPageId};
use wyvern_wizard::{
    add_edge, extract_local_script_srcs, extract_next_hops, extract_next_wizard_refs,
    lint_dataflow, lint_page, merge_html_dataflow_overlay, parse_dataflow_from_json,
    DataflowLintInput, DataflowSpec, GraphPage, LintFinding, PageInfo, PageRole,
    WizardPageGraph,
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
    /// I/O or parse failure (exit 1) — emit via [`emit_wizard_lint_stage_error`].
    Stage {
        /// Human-readable detail (one or more lines).
        message: String,
        /// Process exit code (typically 1).
        exit_code: i32,
    },
    /// Emit-boundary serialize failure.
    Emit(EmitError),
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
    let mut errors: Vec<String> = Vec::new();
    let mut total_pages: usize = 0;

    for path_str in args {
        match lint_package(path_str) {
            Ok((findings, page_count)) => {
                total_pages += page_count;
                all_findings.extend(findings);
            }
            Err(msg) => {
                errors.push(msg);
            }
        }
    }

    // I/O errors are reported on stderr and set exit 1.
    if !errors.is_empty() {
        return Err(WizardCmdError::Stage {
            message: errors.join("\n"),
            exit_code: 1,
        });
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
/// Returns `(findings, page_count)` on success, or an error string on failure.
fn lint_package(path_str: &str) -> Result<(Vec<LintFinding>, usize), String> {
    let input = Path::new(path_str);

    // Resolve wizard.json and wizard root directory.
    let (wizard_json_path, wizard_dir) = resolve_wizard_paths(input)?;

    // Parse wizard.json to get the entry page descriptor.
    let json_content = std::fs::read_to_string(&wizard_json_path)
        .map_err(|e| format!("error: cannot read '{}': {e}", wizard_json_path.display()))?;

    let (entry_id, entry_html) = parse_wizard_json_entry(&json_content)
        .map_err(|msg| format!("error: '{}': {msg}", wizard_json_path.display()))?;

    // BFS to build the reachable page graph.
    let (pages, graph) = build_page_graph(&wizard_dir, &entry_id, &entry_html)?;

    // Nav lint for each page.
    let mut findings: Vec<LintFinding> = pages
        .iter()
        .flat_map(|p| {
            lint_page(&PageInfo {
                id: &p.id,
                file: &p.rel_path,
                html: &p.html,
                role: p.role,
            })
        })
        .collect();

    // Dataflow lint when config.dataflow is declared.
    if let Some(mut spec) = parse_dataflow_from_json(&json_content) {
        for page in &pages {
            merge_html_dataflow_overlay(&mut spec, &page.id, &page.html);
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
fn resolve_wizard_paths(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    if input.is_dir() {
        let json = input.join("wizard.json");
        if json.is_file() {
            return Ok((json, input.to_path_buf()));
        }
        return Err(format!(
            "error: '{}' is a directory but contains no wizard.json",
            input.display()
        ));
    }
    if input.is_file() {
        let dir = input.parent().unwrap_or(Path::new("."));
        return Ok((input.to_path_buf(), dir.to_path_buf()));
    }
    Err(format!(
        "error: '{}' not found (expected directory or wizard.json path)",
        input.display()
    ))
}

/// Extract `(entry_id, entry_html)` from wizard.json content.
fn parse_wizard_json_entry(json: &str) -> Result<(String, String), String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;

    let page = value
        .get("page")
        .ok_or("missing 'page' field in wizard.json")?;

    let id = WizardPageId::try_new(
        page
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("wizard.json page.id must be a string")?,
    )
    .map_err(|_| "wizard.json page.id must be non-empty".to_string())?;

    let html = WizardPageHtml::try_new(
        page
            .get("html")
            .and_then(|v| v.as_str())
            .ok_or("wizard.json page.html must be a string")?,
    )
    .map_err(|_| "wizard.json page.html must be non-empty".to_string())?;

    Ok((id.into_inner(), html.into_inner()))
}

// ── BFS page graph ────────────────────────────────────────────────────────────

struct PageNode {
    id: String,
    /// Path relative to the wizard root (for display).
    rel_path: String,
    html: String,
    role: PageRole,
}

/// Build the reachable page graph via BFS, starting from the entry page.
///
/// Returns page nodes for nav lint and a [`WizardPageGraph`] for dataflow lint.
fn build_page_graph(
    wizard_dir: &Path,
    entry_id: &str,
    entry_html: &str,
) -> Result<(Vec<PageNode>, WizardPageGraph), String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, String, PageRole, Option<String>)> = VecDeque::new();
    let mut pages: Vec<PageNode> = Vec::new();
    let mut graph = WizardPageGraph {
        entry_id: entry_id.to_string(),
        pages: HashMap::new(),
        edges: HashMap::new(),
    };

    queue.push_back((
        entry_id.to_string(),
        entry_html.to_string(),
        PageRole::Entry,
        None,
    ));

    while let Some((id, html_rel, role, from_id)) = queue.pop_front() {
        if visited.contains(&html_rel) {
            continue;
        }
        visited.insert(html_rel.clone());

        if let Some(from) = from_id {
            add_edge(&mut graph, &from, &id);
        }

        let html_abs = wizard_dir.join(&html_rel);
        let html_content = std::fs::read_to_string(&html_abs).map_err(|e| {
            format!(
                "error: cannot read page '{}': {e}",
                html_abs.display()
            )
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
                if !visited.contains(&hop.html) {
                    queue.push_back((
                        hop.id.clone(),
                        hop.html.clone(),
                        PageRole::Inner,
                        Some(id.clone()),
                    ));
                }
            }
        }

        graph.pages.insert(
            id.clone(),
            GraphPage {
                id: id.clone(),
                file: html_rel.clone(),
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
        let html_abs = wizard_dir.join(&page.rel_path);
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
}
