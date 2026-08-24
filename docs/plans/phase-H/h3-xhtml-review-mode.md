---
id: h.3
title: XHTML review mode (--review)
status: implemented
branch: feature/phase-H-h3-xhtml-review
worktree: ../wyvern-worktrees/feature/phase-H-h3-xhtml-review
target: integrate/phase-H
---

# Sprint h.3 — Review mode (`--review`)

## Goal

Add **`--review`** to `report-xhtml`: enriched frame with comment textarea and
Cancel / Approve buttons; structured finish JSON for agent loops.

## Hard dependencies

- h.2 merged (`report-xhtml` + manifest + basic-array frame)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-H/h3-xhtml-review-mode.md` | This sprint doc |
| `scripts/ext/xhtml_report.py` | `--mode review` frame profile; embeds manifest JSON in page |
| `ui/shared/report-review.js` | POST `/api/report/finish`; exactly one POST per button click (disable after submit) |
| `ui/shared/report-base.css` | Review footer layout |
| `crates/wyvern-host/src/routes/report.rs` | `POST /api/report/finish` + finish validation errors |
| `crates/wyvern-host/src/report_finish.rs` | `ReportFinishError` stable codes (`REPORT_FINISH_*`) |
| `crates/wyvern-host/src/report_session.rs` | `ValidatedReportManifest` capability token — finish handler requires proof of validated command panels |
| `crates/wyvern-schema/src/result.rs` | Extend `CommandResult::Report` finish `data` shape docs/tests (review finish) |
| `share/wyvern/extensions.json` | `report-xhtml-review` extension (longer prefix) |
| `crates/wyvern/src/cli_args.rs` | `usage_message()` — `wyvern report-xhtml --review <manifest.json>` |
| `crates/wyvern/tests/extensions_catalog.rs` | REQ-0137 parity for `report-xhtml-review` only |
| `crates/wyvern/tests/extensions_xhtml_review.rs` | Review expand + finish integration |
| `crates/wyvern-host/tests/report_review_finish.rs` | API contract tests |

### REQ traceability (h.3 lands)

| REQ | Summary |
|-----|---------|
| REQ-0144 | Review finish JSON (`approved`, `comments`, `panels`) on stdout |
| REQ-HOST-0142 | `POST /api/report/finish` registered only in review mode |

### Finish request (normative POST body)

```json
{
  "approved": true,
  "comments": "optional review notes",
  "panels": [{ "path": "panels/fail-1.xhtml", "label": "Fail 1", "role": "failure" }]
}
```

Unknown top-level keys rejected (HTTP 400). `comments` max 32_768 chars.

**`panels` authority (normative):** preexec writes manifest `panels` into
`{tmpdir}/report-command.json` (review-mode command JSON). The host treats that list as
**authoritative** — POST `panels` must match paths/metadata from the embedded manifest;
mismatch → HTTP 400. Preexec embeds manifest as
`<script id="manifest-data" type="application/json">…</script>`; `report-review.js`
copies embedded `panels` into the POST body (not free-form client input).

### Review session completion (normative)

| Event | Route | stdout shape |
|-------|-------|----------------|
| Approve / Cancel button | `POST /api/report/finish` | `{ "button": "finish", "data": { … } }` |
| OS-close / viewer exit / session timeout | `POST /api/result` | `{ "button": "dismissed" }` — **no** `data`; not a finish approval |
| View mode (h.1/h.2) | `POST /api/result` | unchanged `{ "button": "dismissed" }` |

While `mode=review`, `/api/result` remains enabled for OS-close only; it does **not**
race finish — session completes on first terminal action (finish or dismiss). Duplicate
finish POST → HTTP 409; duplicate `/api/result` after terminal action → HTTP 409
(inherit `SessionState::complete`).

### Finish response (stdout)

```json
{
  "button": "finish",
  "data": {
    "approved": true,
    "comments": "string",
    "panels": [{ "path": "…", "label": "…", "role": "…" }]
  }
}
```

Cancel → `approved: false`. Approve → `approved: true`. Empty comments allowed.

### CLI

```bash
wyvern report-xhtml --review path/to/review.json
wyvern report-xhtml path/to/review.json   # when manifest.mode is "review"
```

`--review` CLI flag overrides manifest `mode` to `"review"`.

## Acceptance criteria

1. Review page shows textarea + Cancel + Approve (`data-testid` on all three).
2. Approve → stdout JSON with `approved: true` and echo of manifest `panels`.
3. Cancel → `approved: false`; host exits cleanly.
4. View-only report (h.1/h.2 without review) unchanged — no review footer.
5. No wizard APIs loaded on review pages (`wizard-nav.js` absent).
6. `wyvern extensions list --json | jq -e '.[] | select(.id=="report-xhtml-review")'` succeeds (sole registry owner for review prefix).
7. Finish POST acknowledged before host shutdown (RSH-007 `SessionState::complete` / consume-before-shutdown).
8. `report-review.js` disables Approve/Cancel after first finish POST; no automatic retry on 409/5xx.
9. Review OS-close / timeout emits `{ "button": "dismissed" }` via `/api/result` (host test).

## Required validation

```bash
cargo test -p wyvern-cli --test extensions_xhtml_review
cargo test -p wyvern-cli --test extensions_catalog req_0137
cargo test -p wyvern-host report_review_finish
rg -n 'wizard-nav' ui/shared/report-review.js share/wyvern/extensions.json  # expect no wizard-nav on report path
```

## Non-closure

- `workflow.post` persistence of review outcomes (callers/agents handle JSON)
- MCP tool wrapper — Phase E

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md) § Review finish JSON
