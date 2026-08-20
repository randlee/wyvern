//! Static lint analysis for wizard packages (`wyvern wizard lint`).
//!
//! All functions are **pure** — no file I/O. Callers load file content and
//! pass strings here; the CLI wiring lives in `wyvern` crate `wizard_cmd.rs`.
//!
//! # Lint codes
//!
//! | Code              | Rule                                                        |
//! |-------------------|-------------------------------------------------------------|
//! | `WIZARD-LINT-001` | Non-entry page missing back button                          |
//! | `WIZARD-LINT-002` | Terminal page missing cancel button                         |
//! | `WIZARD-LINT-003` | `wizard-nav.js` chrome opt-in present but nav region absent |
//! | `WIZARD-LINT-004` | Non-terminal page (with chrome opt-in) missing next button  |
//!
//! Normative contracts sourced from `ui/shared/wizard-nav.js` sprint d.7.

/// Stable lint finding codes for `wyvern wizard lint`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintCode {
    /// Non-entry page missing `[data-wizard-back]` or `[data-testid="wizard-back"]`.
    W001MissingBack,
    /// Terminal page missing a cancel button (`data-wizard-cancel`,
    /// `data-testid="wizard-cancel"`, or `<button>Cancel</button>`).
    W002MissingCancel,
    /// Page opts into `wizard-nav.js` chrome but has no `[data-wizard-nav]` nav region.
    W003MissingNavRegion,
    /// Non-terminal page with chrome opt-in missing `[data-wizard-next]` or
    /// `[data-testid="wizard-next"]`.
    W004MissingNext,
}

impl LintCode {
    /// Stable string code for display (e.g. `WIZARD-LINT-001`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::W001MissingBack => "WIZARD-LINT-001",
            Self::W002MissingCancel => "WIZARD-LINT-002",
            Self::W003MissingNavRegion => "WIZARD-LINT-003",
            Self::W004MissingNext => "WIZARD-LINT-004",
        }
    }
}

impl std::fmt::Display for LintCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single lint finding for one wizard page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    /// Stable finding code.
    pub code: LintCode,
    /// Page identifier (from wizard nav descriptor).
    pub page_id: String,
    /// File path relative to the wizard package root.
    pub file: String,
    /// Human-readable message.
    pub message: String,
}

impl LintFinding {
    /// One-line display string for stdout reports.
    #[must_use]
    pub fn display_line(&self) -> String {
        format!(
            "{}: {} (page: {}) {}",
            self.code, self.file, self.page_id, self.message
        )
    }
}

/// Page role used by lint rules to select which checks apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageRole {
    /// Entry page (depth 0 from `wizard.json`). Back-button rule does not apply.
    Entry,
    /// Reachable non-entry page. Must have a back button.
    Inner,
}

/// Minimal page descriptor passed to [`lint_page`].
#[derive(Debug, Clone)]
pub struct PageInfo<'a> {
    /// Page identifier from the wizard nav descriptor.
    pub id: &'a str,
    /// File path relative to the wizard package root.
    pub file: &'a str,
    /// HTML content of this page.
    pub html: &'a str,
    /// Entry or inner page role.
    pub role: PageRole,
}

/// Apply all lint rules to a single page and return any findings.
#[must_use]
pub fn lint_page(page: &PageInfo<'_>) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let terminal = is_terminal_page(page.html);
    let chrome = has_wizard_chrome_script(page.html);

    // W001: non-entry pages must have a back button.
    if page.role == PageRole::Inner && !has_back_button(page.html) {
        findings.push(LintFinding {
            code: LintCode::W001MissingBack,
            page_id: page.id.to_string(),
            file: page.file.to_string(),
            message: concat!(
                "non-entry page is missing a back button; ",
                "add data-wizard-back or data-testid=\"wizard-back\""
            )
            .to_string(),
        });
    }

    // W002: terminal pages must have a cancel button.
    if terminal && !has_cancel_button(page.html) {
        findings.push(LintFinding {
            code: LintCode::W002MissingCancel,
            page_id: page.id.to_string(),
            file: page.file.to_string(),
            message: concat!(
                "terminal page is missing a cancel button; ",
                "add data-wizard-cancel, data-testid=\"wizard-cancel\", ",
                "or a <button> with visible text \"Cancel\""
            )
            .to_string(),
        });
    }

    // W003: wizard-nav.js chrome opt-in without a nav region.
    if chrome && !has_nav_region(page.html) {
        findings.push(LintFinding {
            code: LintCode::W003MissingNavRegion,
            page_id: page.id.to_string(),
            file: page.file.to_string(),
            message: concat!(
                "page opts into wizard-nav.js (data-wizard-chrome) ",
                "but has no [data-wizard-nav] nav region"
            )
            .to_string(),
        });
    }

    // W004: non-terminal pages with chrome opt-in must have a next button.
    if !terminal && chrome && !has_next_button(page.html) {
        findings.push(LintFinding {
            code: LintCode::W004MissingNext,
            page_id: page.id.to_string(),
            file: page.file.to_string(),
            message: concat!(
                "non-terminal page with wizard-nav.js opt-in is missing a next button; ",
                "add data-wizard-next or data-testid=\"wizard-next\""
            )
            .to_string(),
        });
    }

    findings
}

// ── HTML attribute scanners (pure string search) ─────────────────────────────

/// `true` if the HTML contains a wizard back-button element.
///
/// Accepts `data-wizard-back` or `data-testid="wizard-back"` / `'wizard-back'`.
#[must_use]
pub fn has_back_button(html: &str) -> bool {
    html.contains("data-wizard-back")
        || html.contains("data-testid=\"wizard-back\"")
        || html.contains("data-testid='wizard-back'")
}

/// `true` if the HTML contains a wizard cancel-button element.
///
/// Accepts `data-wizard-cancel`, `data-testid="wizard-cancel"`,
/// or any `<button>` whose visible text is exactly `"Cancel"`.
#[must_use]
pub fn has_cancel_button(html: &str) -> bool {
    html.contains("data-wizard-cancel")
        || html.contains("data-testid=\"wizard-cancel\"")
        || html.contains("data-testid='wizard-cancel'")
        || has_cancel_text_button(html)
}

/// `true` if any `<button>…</button>` in the HTML has visible text `"Cancel"`.
///
/// Searches for `>Cancel<` inside `<button` … `</button>` element spans.
#[must_use]
pub fn has_cancel_text_button(html: &str) -> bool {
    let mut pos = 0;
    while let Some(rel) = html[pos..].find("<button") {
        let start = pos + rel;
        let Some(end_rel) = html[start..].find("</button>") else {
            break;
        };
        let element_end = start + end_rel + "</button>".len();
        let element = &html[start..element_end];
        // Extract inner content after `>`
        if let Some(tag_end) = element.find('>') {
            let inner_content = &element[tag_end + 1..element.len() - "</button>".len()];
            let trimmed = inner_content.trim();
            if trimmed == "Cancel" || inner_content.contains(">Cancel<") {
                return true;
            }
        }
        pos = start + 1;
    }
    false
}

/// `true` if the page root sets `data-wizard-terminal="true"`.
#[must_use]
pub fn is_terminal_page(html: &str) -> bool {
    html.contains("data-wizard-terminal=\"true\"") || html.contains("data-wizard-terminal='true'")
}

/// `true` if the HTML has a `[data-wizard-nav]` navigation region.
#[must_use]
pub fn has_nav_region(html: &str) -> bool {
    html.contains("data-wizard-nav")
}

/// `true` if the page loads `wizard-nav.js` with the `data-wizard-chrome` opt-in.
#[must_use]
pub fn has_wizard_chrome_script(html: &str) -> bool {
    html.contains("wizard-nav.js") && html.contains("data-wizard-chrome")
}

/// `true` if the HTML contains a wizard next/finish button element.
///
/// Accepts `data-wizard-next` or `data-testid="wizard-next"` / `'wizard-next'`.
#[must_use]
pub fn has_next_button(html: &str) -> bool {
    html.contains("data-wizard-next")
        || html.contains("data-testid=\"wizard-next\"")
        || html.contains("data-testid='wizard-next'")
}

// ── JS static analysis ────────────────────────────────────────────────────────

/// A next-page hop extracted from a JavaScript source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHop {
    /// Page identifier (`id` field in the descriptor).
    pub id: String,
    /// HTML path (`html` field in the descriptor), relative to wizard root.
    pub html: String,
}

/// Extract local script `src` attributes from an HTML document.
///
/// Returns only relative paths (not starting with `/` or a URL scheme).
/// Skips shared wizard infrastructure assets (`wizard-nav.js`, `wyvern-api.js`).
#[must_use]
pub fn extract_local_script_srcs(html: &str) -> Vec<String> {
    let mut srcs = Vec::new();
    let mut pos = 0;
    while let Some(rel) = html[pos..].find("<script") {
        let start = pos + rel;
        let tag_end = html[start..]
            .find('>')
            .map(|r| start + r)
            .unwrap_or(html.len());
        let tag = &html[start..tag_end];
        if let Some(src) = extract_attr_value(tag, "src") {
            // Skip absolute paths (shared assets and external URLs).
            if !src.starts_with('/')
                && !src.contains("://")
                && !src.is_empty()
                && !src.ends_with("wizard-nav.js")
                && !src.ends_with("wyvern-api.js")
            {
                srcs.push(src);
            }
        }
        pos = start + 7; // advance past "<script"
    }
    srcs
}

/// Extract next-page hops from a JavaScript source.
///
/// Scans for `wizardNextDescriptor` and `wyvernWizardNext(` anchors, then
/// extracts `{ id, html }` pairs from the surrounding object literals.
/// Duplicate hops (same `html` path) are de-duplicated.
#[must_use]
pub fn extract_next_hops(js: &str) -> Vec<PageHop> {
    const ANCHORS: &[&str] = &["wizardNextDescriptor", "wyvernWizardNext("];
    let mut hops: Vec<PageHop> = Vec::new();
    let mut pos = 0;

    while pos < js.len() {
        let rest = &js[pos..];
        let Some(rel) = ANCHORS.iter().filter_map(|p| rest.find(p)).min() else {
            break;
        };
        let abs = pos + rel;

        // Look for the object literal `{` within the next 1 KiB.
        let window = &js[abs..js.len().min(abs + 1024)];
        if let Some(brace_rel) = window.find('{') {
            if let Some(hop) = try_parse_descriptor(&window[brace_rel..]) {
                if !hops.iter().any(|h| h.html == hop.html) {
                    hops.push(hop);
                }
            }
        }
        pos = abs + 1;
    }

    hops
}

/// Try to extract `{ id: "...", html: "..." }` from the start of `s`.
fn try_parse_descriptor(s: &str) -> Option<PageHop> {
    let inner = extract_object_inner(s)?;
    let id = extract_js_field_value(inner, "id")?;
    let html = extract_js_field_value(inner, "html")?;
    // Only accept if it looks like an HTML path (ends with .html).
    if html.ends_with(".html") {
        Some(PageHop { id, html })
    } else {
        None
    }
}

/// Extract the content between the outermost `{` and its matching `}`.
fn extract_object_inner(s: &str) -> Option<&str> {
    let rest = s.strip_prefix('{')?;
    let mut depth: usize = 1;
    let mut end = 0;
    for (i, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth == 0 {
        Some(&rest[..end])
    } else {
        None
    }
}

/// Extract a string value for a named field from a JS object body.
///
/// Recognises `"key": "value"`, `'key': "value"`, and bare `key: "value"` /
/// `key: 'value'` forms. The bare form uses a word-boundary check so `id:`
/// does not match inside `template_id:`.
fn extract_js_field_value(obj: &str, key: &str) -> Option<String> {
    // Quoted-key patterns come first to prefer exact matches.
    for pat in [format!("\"{key}\":"), format!("'{key}':")]
        .iter()
        .map(String::as_str)
    {
        if let Some(pos) = obj.find(pat) {
            let after = obj[pos + pat.len()..].trim_start();
            if let Some(val) = extract_quoted_string(after) {
                return Some(val);
            }
        }
    }

    // Bare `key:` with word-boundary check (preceding char not alphanumeric / _).
    let bare = format!("{key}:");
    let mut search = obj;
    while let Some(pos) = search.find(bare.as_str()) {
        let boundary_ok = search[..pos]
            .chars()
            .last()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        if boundary_ok {
            let after = search[pos + bare.len()..].trim_start();
            if let Some(val) = extract_quoted_string(after) {
                return Some(val);
            }
        }
        search = &search[pos + 1..];
    }

    None
}

/// Extract a double- or single-quoted string value from the start of `s`.
fn extract_quoted_string(s: &str) -> Option<String> {
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

/// Extract a quoted HTML attribute value from a tag fragment.
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── has_back_button ──────────────────────────────────────────────────────

    #[test]
    fn back_button_detects_data_wizard_back_attr() {
        let html = r#"<button data-wizard-back>Back</button>"#;
        assert!(has_back_button(html));
    }

    #[test]
    fn back_button_detects_testid_double_quotes() {
        let html = r#"<button data-testid="wizard-back">Back</button>"#;
        assert!(has_back_button(html));
    }

    #[test]
    fn back_button_detects_testid_single_quotes() {
        let html = r#"<button data-testid='wizard-back'>Back</button>"#;
        assert!(has_back_button(html));
    }

    #[test]
    fn back_button_absent_returns_false() {
        let html = r#"<button>Next</button>"#;
        assert!(!has_back_button(html));
    }

    // ── has_cancel_button ────────────────────────────────────────────────────

    #[test]
    fn cancel_button_detects_data_wizard_cancel() {
        let html = r#"<button data-wizard-cancel>Cancel</button>"#;
        assert!(has_cancel_button(html));
    }

    #[test]
    fn cancel_button_detects_testid() {
        let html = r#"<button data-testid="wizard-cancel">Cancel</button>"#;
        assert!(has_cancel_button(html));
    }

    #[test]
    fn cancel_button_detects_cancel_text_in_button() {
        let html = r#"<button type="button" class="secondary">Cancel</button>"#;
        assert!(has_cancel_button(html));
    }

    #[test]
    fn cancel_button_cancel_text_in_para_not_button_returns_false() {
        // "Cancel" appears in prose, not in a <button> element.
        let html = r#"<p>Finish copies. Cancel writes nothing.</p>"#;
        assert!(!has_cancel_button(html));
    }

    #[test]
    fn cancel_button_finish_button_only_returns_false() {
        let html = r#"<button data-wizard-next>Finish</button>"#;
        assert!(!has_cancel_button(html));
    }

    // ── is_terminal_page ────────────────────────────────────────────────────

    #[test]
    fn terminal_page_double_quotes() {
        let html = r#"<main data-wizard-terminal="true">"#;
        assert!(is_terminal_page(html));
    }

    #[test]
    fn terminal_page_single_quotes() {
        let html = r#"<main data-wizard-terminal='true'>"#;
        assert!(is_terminal_page(html));
    }

    #[test]
    fn non_terminal_page_returns_false() {
        let html = r#"<main data-testid="form">"#;
        assert!(!is_terminal_page(html));
    }

    // ── has_nav_region ───────────────────────────────────────────────────────

    #[test]
    fn nav_region_present() {
        let html = r#"<nav data-wizard-nav aria-label="Wizard navigation">"#;
        assert!(has_nav_region(html));
    }

    #[test]
    fn nav_region_absent() {
        let html = r#"<nav aria-label="Wizard navigation">"#;
        assert!(!has_nav_region(html));
    }

    // ── has_wizard_chrome_script ─────────────────────────────────────────────

    #[test]
    fn chrome_script_opted_in() {
        let html = r#"<script src="/shared/wizard-nav.js" data-wizard-chrome></script>"#;
        assert!(has_wizard_chrome_script(html));
    }

    #[test]
    fn chrome_script_without_opt_in_attr() {
        let html = r#"<script src="/shared/wizard-nav.js"></script>"#;
        assert!(!has_wizard_chrome_script(html));
    }

    // ── lint_page — W001 (missing back) ──────────────────────────────────────

    #[test]
    fn w001_fires_on_inner_page_without_back() {
        let page = PageInfo {
            id: "form",
            file: "pages/form.html",
            html: r#"<main><button data-wizard-next>Next</button></main>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(
            findings.iter().any(|f| f.code == LintCode::W001MissingBack),
            "{findings:?}"
        );
    }

    #[test]
    fn w001_silent_on_entry_page_without_back() {
        // Entry page (depth 0) is exempt from the back-button rule.
        let page = PageInfo {
            id: "pick",
            file: "pages/pick.html",
            html: r#"<main><button data-wizard-next>Next</button></main>"#,
            role: PageRole::Entry,
        };
        let findings = lint_page(&page);
        assert!(
            !findings.iter().any(|f| f.code == LintCode::W001MissingBack),
            "{findings:?}"
        );
    }

    #[test]
    fn w001_silent_when_back_present() {
        let page = PageInfo {
            id: "form",
            file: "pages/form.html",
            html: r#"<button data-wizard-back>Back</button><button data-wizard-next>Next</button>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(
            !findings.iter().any(|f| f.code == LintCode::W001MissingBack),
            "{findings:?}"
        );
    }

    // ── lint_page — W002 (missing cancel) ────────────────────────────────────

    #[test]
    fn w002_fires_on_terminal_page_without_cancel() {
        let page = PageInfo {
            id: "review",
            file: "pages/review.html",
            html: r#"<main data-wizard-terminal="true"><button data-wizard-next>Finish</button></main>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(
            findings
                .iter()
                .any(|f| f.code == LintCode::W002MissingCancel),
            "{findings:?}"
        );
    }

    #[test]
    fn w002_silent_on_terminal_page_with_data_wizard_cancel() {
        let page = PageInfo {
            id: "review",
            file: "pages/review.html",
            html: r#"<main data-wizard-terminal="true">
                <button data-wizard-cancel>Cancel</button>
                <button data-wizard-next>Finish</button>
            </main>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(
            !findings
                .iter()
                .any(|f| f.code == LintCode::W002MissingCancel),
            "{findings:?}"
        );
    }

    #[test]
    fn w002_silent_on_terminal_page_with_cancel_text_button() {
        let page = PageInfo {
            id: "review",
            file: "pages/review.html",
            html: r#"<main data-wizard-terminal="true">
                <button type="button" class="secondary">Cancel</button>
                <button data-wizard-next>Finish</button>
            </main>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(
            !findings
                .iter()
                .any(|f| f.code == LintCode::W002MissingCancel),
            "{findings:?}"
        );
    }

    #[test]
    fn w002_not_fired_on_non_terminal_page() {
        let page = PageInfo {
            id: "form",
            file: "pages/form.html",
            html: r#"<main><button data-wizard-back>Back</button><button data-wizard-next>Next</button></main>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(
            !findings
                .iter()
                .any(|f| f.code == LintCode::W002MissingCancel),
            "{findings:?}"
        );
    }

    // ── lint_page — W003 (missing nav region) ────────────────────────────────

    #[test]
    fn w003_fires_when_chrome_script_but_no_nav_region() {
        let page = PageInfo {
            id: "pick",
            file: "pages/pick.html",
            html: r#"<script src="/shared/wizard-nav.js" data-wizard-chrome></script>
                <button data-wizard-next>Next</button>"#,
            role: PageRole::Entry,
        };
        let findings = lint_page(&page);
        assert!(
            findings
                .iter()
                .any(|f| f.code == LintCode::W003MissingNavRegion),
            "{findings:?}"
        );
    }

    #[test]
    fn w003_silent_when_nav_region_present() {
        let page = PageInfo {
            id: "pick",
            file: "pages/pick.html",
            html: r#"<nav data-wizard-nav></nav>
                <script src="/shared/wizard-nav.js" data-wizard-chrome></script>"#,
            role: PageRole::Entry,
        };
        let findings = lint_page(&page);
        assert!(
            !findings
                .iter()
                .any(|f| f.code == LintCode::W003MissingNavRegion),
            "{findings:?}"
        );
    }

    // ── lint_page — W004 (missing next) ─────────────────────────────────────

    #[test]
    fn w004_fires_on_non_terminal_page_without_next() {
        let page = PageInfo {
            id: "form",
            file: "pages/form.html",
            html: r#"<nav data-wizard-nav><button data-wizard-back>Back</button></nav>
                <script src="/shared/wizard-nav.js" data-wizard-chrome></script>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(
            findings.iter().any(|f| f.code == LintCode::W004MissingNext),
            "{findings:?}"
        );
    }

    #[test]
    fn w004_silent_on_terminal_page_without_labeled_next() {
        // Terminal pages show "Finish" via wizard-nav.js runtime relabel; no W004.
        let page = PageInfo {
            id: "review",
            file: "pages/review.html",
            html: r#"<main data-wizard-terminal="true">
                <nav data-wizard-nav>
                    <button data-wizard-back>Back</button>
                    <button data-wizard-cancel>Cancel</button>
                    <button data-wizard-next>Finish</button>
                </nav>
                <script src="/shared/wizard-nav.js" data-wizard-chrome></script>
            </main>"#,
            role: PageRole::Inner,
        };
        let findings = lint_page(&page);
        assert!(findings.is_empty(), "expected clean, got: {findings:?}");
    }

    // ── extract_local_script_srcs ─────────────────────────────────────────────

    #[test]
    fn local_scripts_extracts_relative_src() {
        let html = r#"<script src="../app.js"></script>"#;
        let srcs = extract_local_script_srcs(html);
        assert_eq!(srcs, vec!["../app.js"]);
    }

    #[test]
    fn local_scripts_skips_shared_wizard_nav() {
        let html = r#"<script src="/shared/wizard-nav.js" data-wizard-chrome></script>"#;
        let srcs = extract_local_script_srcs(html);
        assert!(srcs.is_empty(), "{srcs:?}");
    }

    #[test]
    fn local_scripts_skips_absolute_paths() {
        let html = r#"<script src="/shared/wyvern-api.js"></script>"#;
        let srcs = extract_local_script_srcs(html);
        assert!(srcs.is_empty(), "{srcs:?}");
    }

    // ── extract_next_hops ────────────────────────────────────────────────────

    #[test]
    fn extract_hops_finds_wizard_next_descriptor() {
        let js = r#"
            global.wizardNextDescriptor = {
                id: "form",
                title: "Customize template",
                html: "pages/form.html"
            };
        "#;
        let hops = extract_next_hops(js);
        assert_eq!(hops.len(), 1, "{hops:?}");
        assert_eq!(hops[0].id, "form");
        assert_eq!(hops[0].html, "pages/form.html");
    }

    #[test]
    fn extract_hops_finds_wyvernwizardnext_call() {
        let js = r#"
            global.wyvernWizardNext(data, {
                id: "review",
                title: "Review template",
                html: "pages/review.html"
            });
        "#;
        let hops = extract_next_hops(js);
        assert_eq!(hops.len(), 1, "{hops:?}");
        assert_eq!(hops[0].id, "review");
        assert_eq!(hops[0].html, "pages/review.html");
    }

    #[test]
    fn extract_hops_deduplicates_same_html_path() {
        let js = r#"
            global.wizardNextDescriptor = { id: "form", html: "pages/form.html" };
            global.wyvernWizardNext(data, { id: "form", html: "pages/form.html" });
        "#;
        let hops = extract_next_hops(js);
        assert_eq!(hops.len(), 1, "expected dedup, got {hops:?}");
    }

    #[test]
    fn extract_hops_returns_multiple_distinct_hops() {
        let js = r#"
            if (x) {
                global.wizardNextDescriptor = { id: "form", html: "pages/form.html" };
            } else {
                global.wizardNextDescriptor = { id: "review", html: "pages/review.html" };
            }
        "#;
        let hops = extract_next_hops(js);
        assert_eq!(hops.len(), 2, "{hops:?}");
    }

    #[test]
    fn extract_hops_ignores_non_html_paths() {
        let js = r#"global.wizardNextDescriptor = { id: "cfg", html: "config" };"#;
        let hops = extract_next_hops(js);
        assert!(hops.is_empty(), "{hops:?}");
    }

    #[test]
    fn extract_js_field_value_bare_key_word_boundary() {
        // "id" should not match inside "template_id"
        let obj = r#"template_id: "ignored", id: "correct""#;
        let val = extract_js_field_value(obj, "id");
        assert_eq!(val.as_deref(), Some("correct"), "{val:?}");
    }

    #[test]
    fn lint_finding_display_line_format() {
        let f = LintFinding {
            code: LintCode::W002MissingCancel,
            page_id: "review".into(),
            file: "pages/review.html".into(),
            message: "terminal page is missing a cancel button".into(),
        };
        let line = f.display_line();
        assert!(line.contains("WIZARD-LINT-002"), "{line}");
        assert!(line.contains("pages/review.html"), "{line}");
        assert!(line.contains("review"), "{line}");
    }
}
