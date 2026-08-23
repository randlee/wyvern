# QA Findings Update

Generated: 2026-08-23T21:03:25Z
QA Pass: qa-plan-h-1
Sprint/Task: Phase H (plan) / qa-plan-h-1
Branch: `plan/phase-H-xhtml-reporting`
Commit: `920ac6d3c6ced8ff092e90986ff24acf3a4fa3aa`
PR: #0
Verdict: **FAIL**

## Machine Status (JSON)
```json
{
  "sprint": "Phase H (plan)",
  "task": "qa-plan-h-1",
  "branch": "plan/phase-H-xhtml-reporting",
  "commit": "920ac6d3c6ced8ff092e90986ff24acf3a4fa3aa",
  "pr": 0,
  "verdict": "FAIL",
"reviewer_spawn_gate": "pass",
  "reviewer_manifest": [{"agent":"req-qa","task_id":"6bd81268-955b-408a-a4b2-39b46c7f58af","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"FAIL","findings":{"blocking":6,"important":4,"minor":2}},{"agent":"arch-qa","task_id":"6e300ec0-13ca-4101-8d81-29c501a88b56","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"PASS","findings":{"blocking":0,"important":0,"minor":0}},{"agent":"rust-best-practices-agent","task_id":"f73b6cd5-8573-41c4-8be1-09d96b9ef572","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"findings","findings":{"blocking":0,"important":5,"minor":3}},{"agent":"rust-service-hardening-agent","task_id":"d7ab8466-c1e1-4662-ac0a-dc909e221c5f","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"findings","findings":{"blocking":0,"important":1,"minor":5}}],
  "missing_reviewers": [],
  "unparsed_reviewers": [],
  "aggregation_source": "reviewer_fenced_json_union_todo_scan",
"evidence_chain": {"qa_pass":"qa-plan-h-1","commit":"920ac6d3c6ced8ff092e90986ff24acf3a4fa3aa","pr_comment_url":"local:.cursor/phase-H-plan-qa-report.md","coordinator_task_id":"cursor-quality-mgr","triage":{"ttl_paths":[]}},
"deliverables": {
    "complete": 4,
    "total": 14,
    "percent": 28.6
  },
  "findings": {
    "blocking": 6,
    "important": 10,
    "minor": 10
  },
  "blocking_ids": ["ATM-QA-001","ATM-QA-002","ATM-QA-003","ATM-QA-004","ATM-QA-005","ATM-QA-006"],
  "merge_readiness": "blocked",
  "merge_reason": "6 blocking findings, deliverable completion 28.6%, 26 open findings aggregated from reviewers",
  "next_action": "triage_and_fix",
  "owner": "parent-orchestrator"
}
```

## Findings Summary
- Reviewer spawn gate: **pass**
- Deliverables: 4/14 (28.6%)
- Blocking: 6
- Important: 10
- Minor: 10

## Reviewer Manifest
```json
[{"agent":"req-qa","task_id":"6bd81268-955b-408a-a4b2-39b46c7f58af","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"FAIL","findings":{"blocking":6,"important":4,"minor":2}},{"agent":"arch-qa","task_id":"6e300ec0-13ca-4101-8d81-29c501a88b56","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"PASS","findings":{"blocking":0,"important":0,"minor":0}},{"agent":"rust-best-practices-agent","task_id":"f73b6cd5-8573-41c4-8be1-09d96b9ef572","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"findings","findings":{"blocking":0,"important":5,"minor":3}},{"agent":"rust-service-hardening-agent","task_id":"d7ab8466-c1e1-4662-ac0a-dc909e221c5f","spawn_actor":"parent-orchestrator","fenced_json_received":true,"verdict":"findings","findings":{"blocking":0,"important":1,"minor":5}}]
```
## Evidence Chain
```json
{"qa_pass":"qa-plan-h-1","commit":"920ac6d3c6ced8ff092e90986ff24acf3a4fa3aa","pr_comment_url":"local:.cursor/phase-H-plan-qa-report.md","coordinator_task_id":"cursor-quality-mgr","triage":{"ttl_paths":[]}}
```
## Blocking Findings
- **ATM-QA-001** (Blocking): Phase H introduces `type: report` but ADR-0022 and Phase F contract still forbid new schema variants/host dialog types — add h.1 deliverable to amend architecture.md and cli-extensions-contract.md.
- **ATM-QA-002** (Blocking): Normative `{arg:review:flag}` template syntax is undocumented and unimplemented in Phase F runtime — implement/document or replace with existing mechanism.
- **ATM-QA-003** (Blocking): `{title_from_manifest}` and `{mode_from_preexec}` expand placeholders have no implementation path — specify concrete expand mechanism in h.2.
- **ATM-QA-004** (Blocking): Contract limits `/shared/*` to review mode but h.1/h.2 frame profiles require `/shared/report-base.css` in view mode — amend contract Host HTTP surface.
- **ATM-QA-005** (Blocking): No sprint deliverable adds numbered REQs for report command, host routes, finish JSON, or extensions — add h.1/h.3 REQ traceability deliverables.
- **ATM-QA-006** (Blocking): README claims ADR in contract but only ADR candidate stub exists — add h.1 deliverable to promote formal ADR in docs/architecture.md.

## Detailed Findings
### Blocking

- **ATM-QA-001** | Blocking | cross-doc-conflict | docs/plans/phase-H/xhtml-reporting-contract.md — Phase H introduces `type: report` but ADR-0022 and Phase F CLI extensions contract still forbid new schema variants and host dialog types; no sprint deliverable amends those documents. **Fix:** Add h.1 deliverable to amend docs/architecture.md (ADR-0022) and docs/plans/phase-F/cli-extensions-contract.md.
- **ATM-QA-002** | Blocking | implementation-drift | docs/plans/phase-H/xhtml-reporting-contract.md:190 — Normative `{arg:review:flag}` is undocumented/unimplemented; Phase F only supports `{arg:name}` and `{arg:name:repeat}`. **Fix:** Implement/document `{arg:name:flag}` in h.3 or replace with existing argv/preexec mechanism.
- **ATM-QA-003** | Blocking | implementation-drift | docs/plans/phase-H/xhtml-reporting-contract.md:197 — `{title_from_manifest}` and `{mode_from_preexec}` have no implementation path. **Fix:** Use `command_from_file` pattern or document new template vars with h.2 runtime deliverables.
- **ATM-QA-004** | Blocking | cross-doc-conflict | docs/plans/phase-H/xhtml-reporting-contract.md:54 — Contract limits `/shared/*` to review mode but basic-single/array frames require `/shared/report-base.css` in view mode. **Fix:** Amend Host HTTP surface and add h.1 deliverable for static mount.
- **ATM-QA-005** | Blocking | requirements | docs/plans/phase-H/README.md — No REQs for report command, host routes, finish JSON, or extensions in crate/top-level requirements. **Fix:** Add h.1/h.3 REQ-Hxxx deliverables following Phase G pattern.
- **ATM-QA-006** | Blocking | deliverable-missing | docs/plans/phase-H/README.md:105 — README claims ADR in contract but only candidate stub exists. **Fix:** Add h.1 deliverable for formal ADR-0025 in docs/architecture.md.

### Important

- **ATM-QA-007** | Important | acceptance-gap | docs/plans/phase-H/h1-xhtml-single-panel.md:73 — REQ-0137 parity: missing extensions_catalog/help_surface test updates and full registry JSON with description/examples. **Fix:** Add h.1/h.2 deliverables for parity tests and normative registry entries.
- **ATM-QA-008** | Important | cross-doc-conflict | docs/plans/phase-H/README.md:93 — Phase acceptance references `extensions_xhtml_report` test but sprints name three separate test files. **Fix:** Align README with sprint test file names.
- **ATM-QA-009** | Important | acceptance-gap | docs/plans/phase-H/h5-synthetic-xhtml-example.md:53 — h.5 CI smoke uses `--validate-manifest` but h.2 preexec deliverables omit it. **Fix:** Add subcommand to h.2 or move to h.5 with explicit deliverable.
- **ATM-QA-010** | Important | acceptance-gap | docs/plans/phase-H/h2-xhtml-panel-array.md:58 — h.2 validation references fixture path not in h.2 deliverables (owned by h.5). **Fix:** Add fixture to h.2 or use inline temp manifest in test harness.
- **RBP-F001** | Important | RBP-001 | docs/plans/phase-H/README.md:79 — No Error Inventory for report-specific failure modes (preexec, manifest validation, host routing, finish). **Fix:** Add normative Error Inventory to xhtml-reporting-contract.md.
- **RBP-F002** | Important | RBP-001 | docs/plans/phase-H/xhtml-reporting-contract.md:55 — `/api/report/finish` lacks HTTP/CLI error behavior spec (POST in view mode, malformed body, duplicate finish). **Fix:** Add finish-route error inventory following wizard.rs pattern.
- **RBP-F003** | Important | RBP-002 | docs/plans/phase-H/xhtml-reporting-contract.md:38 — View/review lifecycle enforced by convention, not typed model. **Fix:** Specify ReportMode enum and typestate on ReportCommand/session.
- **RBP-F004** | Important | RBP-004 | docs/plans/phase-H/h1-xhtml-single-panel.md:29 — ReportCommand planned as validate-only with no domain newtypes for path/title/mode/role. **Fix:** Define ReportPagePath, ReportTitle, ReportMode, PanelRole newtypes.
- **RBP-F005** | Important | RBP-010 | docs/plans/phase-H/h2-xhtml-panel-array.md:51 — Finish capability gated procedurally, no type-level proof tying host routes to validated manifest. **Fix:** Introduce PreparedReportUi/ValidatedManifest capability token.
- **RSH-001** | Important | config_validation | docs/plans/phase-H/xhtml-reporting-contract.md:55 — POST /api/report/finish lacks inbound validation rules (required fields, unknown-key rejection, comments bounds, duplicate-finish). **Fix:** Add normative finish-request schema with field constraints and error codes.

### Minor

- **ATM-QA-011** | Minor | plan | docs/plans/phase-H/README.md:79 — Phase H README lacks numbered phase acceptance criteria (unlike Phase B/G). **Fix:** Add numbered Phase acceptance criteria section.
- **ATM-QA-012** | Minor | cross-doc-conflict | docs/requirements.md:42 — Top-level requirements index omits .xhtml and report-xhtml. **Fix:** Update command-surface summary when REQs land.
- **RBP-F006** | Minor | RBP-001 | docs/plans/phase-H/h2-xhtml-panel-array.md:51 — Manifest/preexec failure guidance limited to human stderr without structured cause/recovery. **Fix:** Document in error inventory with ExtensionError mapping.
- **RBP-F007** | Minor | RBP-005 | docs/plans/phase-H/h1-xhtml-single-panel.md:29 — No borrowed-access ergonomics spec for report newtypes. **Fix:** Follow wizard_page_newtype Deref/AsRef pattern.
- **RBP-F008** | Minor | RBP-004 | docs/plans/phase-H/review-manifest.schema.json:38 — Panel role JSON enum has no corresponding Rust PanelRole type. **Fix:** Add PanelRole enum in wyvern-schema.
- **RSH-002** | Minor | graceful_shutdown | docs/plans/phase-H/h3-xhtml-review-mode.md:63 — Review acceptance omits RSH-007 result-token consume-before-shutdown pattern. **Fix:** Add acceptance criteria requiring SessionState::complete semantics.
- **RSH-003** | Minor | backpressure | docs/plans/phase-H/review-manifest.schema.json:21 — panels array has minItems:1 but no maxItems or size budget. **Fix:** Add maxItems and total stitched HTML byte budget.
- **RSH-004** | Minor | backpressure | docs/plans/phase-H/xhtml-reporting-contract.md:131 — Review comments have no documented size limit. **Fix:** Specify maxLength for comments in finish JSON and DOM maxlength.
- **RSH-005** | Minor | timeouts | docs/plans/phase-H/README.md:46 — Pipeline diagram omits session_timeout, REQUEST_TIMEOUT, preexec timeout inheritance. **Fix:** Add Host hardening inheritance subsection.
- **RSH-006** | Minor | retry_scope | docs/plans/phase-H/h3-xhtml-review-mode.md:27 — report-review.js lacks no-retry/single-submit policy for terminal finish POST. **Fix:** Document exactly-one finish POST per button click.

### Coordinator TODO Scan

- No TODO/FIXME/XXX/HACK comments found in `docs/plans/phase-H/` plan docs at commit 920ac6d.

## Resolved Since Last Pass
- none

## Merge Readiness
- Status: **blocked**
- Reason: 6 blocking findings, deliverable completion 28.6%, 26 open findings aggregated from reviewers

## Next Action
- Action: triage_and_fix
- Owner: parent-orchestrator
