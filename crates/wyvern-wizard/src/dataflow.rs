//! Dataflow lint rules WIZARD-LINT-005–008 against `config.dataflow`.
//!
//! Pure analysis — no file I/O. See `references/core/dataflow-contracts.md`.

use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::Value;

use crate::lint::{is_terminal_page, LintCode, LintFinding};

/// Parsed `config.dataflow` version 1.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataflowSpec {
    /// Page id → declaration.
    pub pages: HashMap<String, PageDataflow>,
}

/// Per-page dataflow declaration (merged JSON + optional HTML meta).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageDataflow {
    /// Keys this page exports onto stack / finish `data`.
    pub exports: HashMap<String, String>,
    /// Keys required from prior pages on every reachable path.
    pub requires: Vec<String>,
    /// Page may finish the wizard.
    pub terminal: bool,
    /// Shape `workflow.post` reads from finish `data` (terminal only).
    pub post_input: HashMap<String, String>,
}

/// One page in the statically discovered wizard graph.
#[derive(Debug, Clone)]
pub struct GraphPage {
    /// Page id from nav descriptor.
    pub id: String,
    /// HTML path relative to wizard root.
    pub file: String,
    /// HTML content.
    pub html: String,
}

/// Reachable page graph for dataflow path checks.
#[derive(Debug, Clone, Default)]
pub struct WizardPageGraph {
    /// Entry page id from `wizard.json`.
    pub entry_id: String,
    /// Page id → node.
    pub pages: HashMap<String, GraphPage>,
    /// Outgoing edges (from page id → successor page ids).
    pub edges: HashMap<String, Vec<String>>,
}

/// Inputs for dataflow lint (all content pre-loaded by the CLI).
#[derive(Debug)]
pub struct DataflowLintInput<'a> {
    /// Parsed `config.dataflow` (required — caller skips lint when absent).
    pub spec: &'a DataflowSpec,
    /// Statically reachable page graph.
    pub graph: &'a WizardPageGraph,
    /// Local `*.js` sources keyed by path relative to wizard root.
    pub js_files: &'a HashMap<String, String>,
    /// Whether `wizard.json` declares `workflow.post`.
    pub has_workflow_post: bool,
    /// Resolved target packages for WIZARD-LINT-007 (path → spec).
    pub next_wizard_targets: &'a HashMap<String, DataflowSpec>,
}

/// Parse `config.dataflow` from wizard.json content. Returns `None` when absent.
#[must_use]
pub fn parse_dataflow_from_json(json: &str) -> Option<DataflowSpec> {
    let value: Value = serde_json::from_str(json).ok()?;
    parse_dataflow_value(&value)
}

/// Parse `config.dataflow` from a parsed wizard.json value.
#[must_use]
pub fn parse_dataflow_value(wizard: &Value) -> Option<DataflowSpec> {
    let dataflow = wizard.get("config")?.get("dataflow")?;
    let pages_val = dataflow.get("pages")?.as_object()?;

    let mut pages = HashMap::new();
    for (page_id, page_val) in pages_val {
        let obj = page_val.as_object()?;
        let mut decl = PageDataflow::default();

        if let Some(exports) = obj.get("exports").and_then(|v| v.as_object()) {
            for (key, ty) in exports {
                if let Some(token) = ty.as_str() {
                    decl.exports.insert(key.clone(), token.to_string());
                }
            }
        }

        if let Some(requires) = obj.get("requires").and_then(|v| v.as_array()) {
            for item in requires {
                if let Some(key) = item.as_str() {
                    decl.requires.push(key.to_string());
                }
            }
        }

        if let Some(terminal) = obj.get("terminal").and_then(|v| v.as_bool()) {
            decl.terminal = terminal;
        }

        if let Some(post) = obj.get("post_input").and_then(|v| v.as_object()) {
            for (key, ty) in post {
                if let Some(token) = ty.as_str() {
                    decl.post_input.insert(key.clone(), token.to_string());
                }
            }
        }

        pages.insert(page_id.clone(), decl);
    }

    Some(DataflowSpec { pages })
}

/// Merge optional HTML `<meta name="wyvern-dataflow-*">` overlay into `spec`.
pub fn merge_html_dataflow_overlay(spec: &mut DataflowSpec, page_id: &str, html: &str) {
    let entry = spec.pages.entry(page_id.to_string()).or_default();

    if let Some(exports) = parse_meta_kv(html, "wyvern-dataflow-exports") {
        for (key, ty) in exports {
            entry.exports.insert(key, ty);
        }
    }

    if let Some(requires) = parse_meta_requires(html, "wyvern-dataflow-requires") {
        for key in requires {
            if !entry.requires.contains(&key) {
                entry.requires.push(key);
            }
        }
    }
}

/// Apply WIZARD-LINT-005–008 when `config.dataflow` is declared.
#[must_use]
pub fn lint_dataflow(input: &DataflowLintInput<'_>) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    findings.extend(lint_requires(input));
    findings.extend(lint_post_input(input));
    findings.extend(lint_next_wizard(input));
    findings.extend(lint_js_reads(input));
    findings.extend(lint_dag_export_contract(input));
    findings
}

// ── WIZARD-LINT-005 — unsatisfied requires ───────────────────────────────────

fn lint_requires(input: &DataflowLintInput<'_>) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    for (page_id, decl) in &input.spec.pages {
        if decl.requires.is_empty() {
            continue;
        }
        let Some(page) = input.graph.pages.get(page_id) else {
            continue;
        };

        let paths = all_paths_to(input.graph, page_id);
        if paths.is_empty() {
            continue;
        }

        for required_key in &decl.requires {
            for path in &paths {
                let prior_exports = exports_on_path(input.spec, path, page_id);
                if !prior_exports.contains_key(required_key) {
                    findings.push(LintFinding {
                        code: LintCode::W005UnsatisfiedRequire,
                        page_id: page_id.clone(),
                        file: page.file.clone(),
                        message: format!(
                            "requires key '{required_key}' is not exported on every reachable \
                             path to this page (missing on path {})",
                            path.join(" → ")
                        ),
                    });
                }
            }
        }

        // Type conflicts between export declarations for the same key.
        for (key, ty) in &decl.exports {
            if let Some(conflict) = export_type_conflict(input.spec, page_id, key, ty) {
                findings.push(LintFinding {
                    code: LintCode::W005UnsatisfiedRequire,
                    page_id: page_id.clone(),
                    file: page.file.clone(),
                    message: format!(
                        "conflicting export type for '{key}': declared as '{ty}' but also '{conflict}'"
                    ),
                });
            }
        }
    }

    findings
}

fn export_type_conflict(spec: &DataflowSpec, page_id: &str, key: &str, ty: &str) -> Option<String> {
    let decl = spec.pages.get(page_id)?;
    // Scan all pages that also export this key — conflict if types differ.
    for (other_id, other) in &spec.pages {
        if other_id == page_id {
            continue;
        }
        if let Some(other_ty) = other.exports.get(key) {
            if other_ty != ty {
                return Some(other_ty.clone());
            }
        }
    }
    // Same page: duplicate keys in exports map would overwrite — check HTML meta
    // is merged; if decl has one type, no self-conflict.
    let _ = decl;
    None
}

fn exports_on_path(spec: &DataflowSpec, path: &[String], target: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for page_id in path {
        if page_id == target {
            break;
        }
        if let Some(decl) = spec.pages.get(page_id) {
            for (k, v) in &decl.exports {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    out
}

fn all_paths_to(graph: &WizardPageGraph, target: &str) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    let mut stack: Vec<(String, Vec<String>, HashSet<String>)> = Vec::new();
    stack.push((
        graph.entry_id.clone(),
        vec![graph.entry_id.clone()],
        HashSet::from([graph.entry_id.clone()]),
    ));

    while let Some((current, path, visited)) = stack.pop() {
        if current == target {
            paths.push(path);
            continue;
        }
        if let Some(nexts) = graph.edges.get(&current) {
            for next in nexts {
                if visited.contains(next) {
                    continue;
                }
                let mut next_path = path.clone();
                next_path.push(next.clone());
                let mut next_visited = visited.clone();
                next_visited.insert(next.clone());
                stack.push((next.clone(), next_path, next_visited));
            }
        }
    }

    paths
}

// ── WIZARD-LINT-006 — post_input vs exports ──────────────────────────────────

fn lint_post_input(input: &DataflowLintInput<'_>) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    if !input.has_workflow_post {
        return findings;
    }

    for (page_id, decl) in &input.spec.pages {
        if decl.post_input.is_empty() {
            continue;
        }

        let Some(page) = input.graph.pages.get(page_id) else {
            continue;
        };

        let html_terminal = is_terminal_page(&page.html);
        if !decl.terminal && !html_terminal {
            continue;
        }

        for key in decl.post_input.keys() {
            if !decl.exports.contains_key(key) {
                findings.push(LintFinding {
                    code: LintCode::W006PostInputMismatch,
                    page_id: page_id.clone(),
                    file: page.file.clone(),
                    message: format!(
                        "terminal post_input key '{key}' is not listed in this page's exports"
                    ),
                });
            }
        }
    }

    findings
}

// ── WIZARD-LINT-007 — next_wizard input vs target ────────────────────────────

/// A `wizardNextWizard` / finish `next_wizard` reference extracted from JS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextWizardRef {
    /// Literal path string from JS.
    pub path: String,
    /// Input keys copied to the target wizard.
    pub input_keys: Vec<String>,
    /// Source JS file (relative path).
    pub file: String,
}

/// Extract `wizardNextWizard` object literals from JavaScript source.
#[must_use]
pub fn extract_next_wizard_refs(js: &str, file: &str) -> Vec<NextWizardRef> {
    const ANCHORS: &[&str] = &["wizardNextWizard", "next_wizard"];
    let mut refs = Vec::new();
    let mut pos = 0;

    while pos < js.len() {
        let rest = &js[pos..];
        let Some(rel) = ANCHORS.iter().filter_map(|p| rest.find(p)).min() else {
            break;
        };
        let abs = pos + rel;
        let window = &js[abs..js.len().min(abs + 2048)];
        if let Some(obj) = extract_object_after_colon(window) {
            if let Some(path) = extract_js_string_field(obj, "path") {
                let input_keys = extract_input_keys(obj);
                refs.push(NextWizardRef {
                    path,
                    input_keys,
                    file: file.to_string(),
                });
            }
        }
        pos = abs + 1;
    }

    refs
}

fn lint_next_wizard(input: &DataflowLintInput<'_>) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    for (file, js) in input.js_files {
        for nw in extract_next_wizard_refs(js, file) {
            let Some(target_spec) = input.next_wizard_targets.get(&nw.path) else {
                continue;
            };

            let entry_requires = entry_page_requires(target_spec);
            for key in &nw.input_keys {
                if !entry_requires.contains(key) {
                    findings.push(LintFinding {
                        code: LintCode::W007NextWizardInput,
                        page_id: String::new(),
                        file: nw.file.clone(),
                        message: format!(
                            "next_wizard input key '{key}' is not declared as requires on \
                             target '{path}'",
                            path = nw.path
                        ),
                    });
                }
            }
        }
    }

    findings
}

fn entry_page_requires(spec: &DataflowSpec) -> HashSet<String> {
    // Target entry is the first page in spec that has requires, or union all requires
    // on pages reachable from entry — for v1, union all declared requires.
    let mut keys = HashSet::new();
    for decl in spec.pages.values() {
        for key in &decl.requires {
            keys.insert(key.clone());
        }
    }
    keys
}

// ── WIZARD-LINT-008 — JS reads undeclared keys ───────────────────────────────

/// Literal data keys read from stack / page_data in JavaScript.
#[must_use]
pub fn extract_data_reads(js: &str) -> HashSet<String> {
    let mut keys = HashSet::new();
    for cap in regex_lite_scan(js) {
        keys.insert(cap);
    }
    keys
}

fn regex_lite_scan(js: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let patterns = ["data.", "page_data.", ".data."];
    for pat in patterns {
        let mut pos = 0;
        while let Some(rel) = js[pos..].find(pat) {
            let start = pos + rel + pat.len();
            if let Some(key) = read_ident(&js[start..]) {
                if !key.is_empty() && key != "length" && key != "push" {
                    keys.push(key);
                }
            }
            pos = pos + rel + 1;
        }
    }
    keys.sort_unstable();
    keys.dedup();
    keys
}

fn read_ident(s: &str) -> Option<String> {
    let mut end = 0;
    for (i, ch) in s.char_indices() {
        if i == 0 {
            if !ch.is_ascii_alphabetic() && ch != '_' {
                return None;
            }
        } else if !ch.is_ascii_alphanumeric() && ch != '_' {
            break;
        }
        end = i + ch.len_utf8();
    }
    if end == 0 {
        return None;
    }
    Some(s[..end].to_string())
}

fn lint_js_reads(input: &DataflowLintInput<'_>) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let all_exports = all_exported_keys(input.spec);

    for (file, js) in input.js_files {
        for key in extract_data_reads(js) {
            if all_exports.contains(&key) {
                continue;
            }
            // Config reads (templates, hook_state, layouts) are not stack exports.
            if is_likely_config_read(js, &key) {
                continue;
            }
            findings.push(LintFinding {
                code: LintCode::W008UndeclaredRead,
                page_id: String::new(),
                file: file.clone(),
                message: format!(
                    "local JS reads stack/page_data key '{key}' but no reachable page exports it"
                ),
            });
        }
    }

    findings
}

fn all_exported_keys(spec: &DataflowSpec) -> HashSet<String> {
    let mut keys = HashSet::new();
    for decl in spec.pages.values() {
        for key in decl.exports.keys() {
            keys.insert(key.clone());
        }
    }
    keys
}

fn is_likely_config_read(js: &str, key: &str) -> bool {
    js.contains(&format!("config.{key}"))
        || js.contains(&format!("config && config.{key}"))
        || js.contains(&format!("config && Array.isArray(config.{key}"))
}

// ── workspace-canvas dag export-contract (reuse 006) ─────────────────────────

const DAG_FIELDS: &[&str] = &["layout_id", "nodes", "edges"];

fn lint_dag_export_contract(input: &DataflowLintInput<'_>) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    for (page_id, decl) in &input.spec.pages {
        if !decl.exports.contains_key("dag") {
            continue;
        }
        let Some(page) = input.graph.pages.get(page_id) else {
            continue;
        };

        let js_blob: String = input.js_files.values().cloned().collect();
        for field in DAG_FIELDS {
            if !js_blob.contains(field) {
                findings.push(LintFinding {
                    code: LintCode::W006PostInputMismatch,
                    page_id: page_id.clone(),
                    file: page.file.clone(),
                    message: format!(
                        "exports 'dag' but local JS does not mention required dag field '{field}'"
                    ),
                });
            }
        }
    }

    findings
}

// ── HTML meta parsing ────────────────────────────────────────────────────────

fn parse_meta_kv(html: &str, name: &str) -> Option<Vec<(String, String)>> {
    let needle = format!("name=\"{name}\"");
    let needle_sq = format!("name='{name}'");
    let content =
        extract_meta_content(html, &needle).or_else(|| extract_meta_content(html, &needle_sq))?;
    Some(parse_kv_list(&content))
}

fn parse_meta_requires(html: &str, name: &str) -> Option<Vec<String>> {
    let needle = format!("name=\"{name}\"");
    let needle_sq = format!("name='{name}'");
    let content =
        extract_meta_content(html, &needle).or_else(|| extract_meta_content(html, &needle_sq))?;
    Some(
        content
            .split(',')
            .map(|s| s.trim().split(':').next().unwrap_or("").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn extract_meta_content(html: &str, name_attr: &str) -> Option<String> {
    let pos = html.find(name_attr)?;
    let tag_start = html[..pos].rfind("<meta")?;
    let tag_end = html[tag_start..].find('>').map(|i| tag_start + i)?;
    let tag = &html[tag_start..=tag_end];
    extract_attr_value(tag, "content")
}

fn parse_kv_list(content: &str) -> Vec<(String, String)> {
    content
        .split(',')
        .filter_map(|pair| {
            let pair = pair.trim();
            if pair.is_empty() {
                return None;
            }
            let mut parts = pair.splitn(2, ':');
            let key = parts.next()?.trim().to_string();
            let ty = parts.next().unwrap_or("any").trim().to_string();
            Some((key, ty))
        })
        .collect()
}

fn extract_object_after_colon(s: &str) -> Option<&str> {
    let brace = s.find('{')?;
    extract_balanced_object(&s[brace..])
}

fn extract_balanced_object(s: &str) -> Option<&str> {
    let rest = s.strip_prefix('{')?;
    let mut depth = 1usize;
    let mut end = 0;
    for (i, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth == 0 {
        Some(&s[..=end])
    } else {
        None
    }
}

fn extract_js_string_field(obj: &str, key: &str) -> Option<String> {
    let patterns = [
        format!("\"{key}\":"),
        format!("'{key}':"),
        format!("{key}:"),
    ];
    for pat in &patterns {
        if let Some(pos) = obj.find(pat.as_str()) {
            let after = obj[pos + pat.len()..].trim_start();
            return extract_quoted(after);
        }
    }
    None
}

fn extract_input_keys(obj: &str) -> Vec<String> {
    let Some(input_pos) = obj.find("input").or_else(|| obj.find("\"input\"")) else {
        return Vec::new();
    };
    let window = &obj[input_pos..];
    let Some(brace) = window.find('{') else {
        return Vec::new();
    };
    let inner = extract_balanced_object(&window[brace..]).unwrap_or("");
    let mut keys = Vec::new();
    let trimmed = inner.trim_start_matches('{').trim_end_matches('}');
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(key) = part.split(':').next() {
            let key = key.trim().trim_matches('"').trim_matches('\'');
            if !key.is_empty() {
                keys.push(key.to_string());
            }
        }
    }
    keys
}

fn extract_quoted(s: &str) -> Option<String> {
    if let Some(rest) = s.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    if let Some(rest) = s.strip_prefix('\'') {
        let end = rest.find('\'')?;
        return Some(rest[..end].to_string());
    }
    None
}

fn extract_attr_value(tag: &str, attr: &str) -> Option<String> {
    for delim in ['"', '\''] {
        let pat = format!("{attr}={delim}");
        if let Some(pos) = tag.find(&pat) {
            let after = &tag[pos + pat.len()..];
            let end = after.find(delim)?;
            return Some(after[..end].to_string());
        }
    }
    None
}

/// Record an edge in the page graph (deduped).
pub fn add_edge(graph: &mut WizardPageGraph, from: &str, to: &str) {
    graph.edges.entry(from.to_string()).or_default();
    let edges = graph.edges.entry(from.to_string()).or_default();
    if !edges.contains(&to.to_string()) {
        edges.push(to.to_string());
    }
}

/// Topological order of page ids via BFS from entry (for diagnostics).
#[must_use]
pub fn page_order(graph: &WizardPageGraph) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(graph.entry_id.clone());

    while let Some(id) = queue.pop_front() {
        if !seen.insert(id.clone()) {
            continue;
        }
        out.push(id.clone());
        if let Some(nexts) = graph.edges.get(&id) {
            for next in nexts {
                queue.push_back(next.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dataflow_from_template_picker_shape() {
        let json = r#"{
            "type": "wizard",
            "page": { "id": "pick", "title": "T", "html": "pages/pick.html" },
            "config": {
                "dataflow": {
                    "version": 1,
                    "pages": {
                        "pick": {
                            "exports": { "template_id": "string", "variables": "object", "output_path": "string" }
                        },
                        "form": {
                            "requires": ["template_id"],
                            "exports": { "template_id": "string", "variables": "object", "output_path": "string" }
                        },
                        "review": {
                            "requires": ["template_id", "output_path"],
                            "terminal": true,
                            "post_input": { "template_id": "string", "variables": "object", "output_path": "string" },
                            "exports": { "template_id": "string", "variables": "object", "output_path": "string" }
                        }
                    }
                }
            }
        }"#;
        let spec = parse_dataflow_from_json(json).expect("dataflow");
        assert!(spec.pages.contains_key("pick"));
        assert_eq!(spec.pages["form"].requires, vec!["template_id"]);
        assert!(spec.pages["review"].terminal);
    }

    #[test]
    fn w005_fires_when_require_missing_on_path() {
        let mut spec = DataflowSpec::default();
        spec.pages.insert(
            "pick".into(),
            PageDataflow {
                exports: HashMap::from([("a".into(), "string".into())]),
                ..Default::default()
            },
        );
        spec.pages.insert(
            "form".into(),
            PageDataflow {
                requires: vec!["missing".into()],
                ..Default::default()
            },
        );

        let mut graph = WizardPageGraph {
            entry_id: "pick".into(),
            ..Default::default()
        };
        graph.pages.insert(
            "pick".into(),
            GraphPage {
                id: "pick".into(),
                file: "pages/pick.html".into(),
                html: String::new(),
            },
        );
        graph.pages.insert(
            "form".into(),
            GraphPage {
                id: "form".into(),
                file: "pages/form.html".into(),
                html: String::new(),
            },
        );
        add_edge(&mut graph, "pick", "form");

        let input = DataflowLintInput {
            spec: &spec,
            graph: &graph,
            js_files: &HashMap::new(),
            has_workflow_post: false,
            next_wizard_targets: &HashMap::new(),
        };
        let findings = lint_dataflow(&input);
        assert!(
            findings
                .iter()
                .any(|f| f.code == LintCode::W005UnsatisfiedRequire),
            "{findings:?}"
        );
    }

    #[test]
    fn w006_fires_when_post_input_not_exported() {
        let mut spec = DataflowSpec::default();
        spec.pages.insert(
            "review".into(),
            PageDataflow {
                terminal: true,
                post_input: HashMap::from([("out".into(), "string".into())]),
                exports: HashMap::new(),
                ..Default::default()
            },
        );

        let mut graph = WizardPageGraph {
            entry_id: "review".into(),
            pages: HashMap::from([(
                "review".into(),
                GraphPage {
                    id: "review".into(),
                    file: "pages/review.html".into(),
                    html: r#"<main data-wizard-terminal="true"></main>"#.into(),
                },
            )]),
            edges: HashMap::new(),
        };

        let input = DataflowLintInput {
            spec: &spec,
            graph: &graph,
            js_files: &HashMap::new(),
            has_workflow_post: true,
            next_wizard_targets: &HashMap::new(),
        };
        let findings = lint_dataflow(&input);
        assert!(
            findings
                .iter()
                .any(|f| f.code == LintCode::W006PostInputMismatch),
            "{findings:?}"
        );
        let _ = &mut graph;
    }

    #[test]
    fn extract_data_reads_finds_template_id() {
        let js = r#"var data = stack[i].data; if (data.template_id) {}"#;
        let keys = extract_data_reads(js);
        assert!(keys.contains("template_id"), "{keys:?}");
    }

    #[test]
    fn extract_next_wizard_refs_finds_path_and_input() {
        let js = r#"
            window.wizardNextWizard = {
                path: "{wyvern_share}/examples/template-picker/wizard.json",
                input: { seed: "x" }
            };
        "#;
        let refs = extract_next_wizard_refs(js, "app.js");
        assert_eq!(refs.len(), 1);
        assert!(refs[0].path.contains("template-picker"));
        assert!(refs[0].input_keys.contains(&"seed".to_string()));
    }
}
