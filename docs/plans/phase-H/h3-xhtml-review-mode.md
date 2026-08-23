---
id: h.3
title: XHTML review mode (--review)
status: planning
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
| `scripts/ext/xhtml_report.py` | `--mode review` frame profile |
| `ui/shared/report-review.js` | POST `/api/report/finish` |
| `ui/shared/report-base.css` | Review footer layout |
| `crates/wyvern-host/src/routes/report.rs` | `POST /api/report/finish` |
| `crates/wyvern-schema/src/result.rs` | Report finish `data` shape docs/tests |
| `share/wyvern/extensions.json` | `{arg:review:flag}` on `report-xhtml` |
| `crates/wyvern/tests/extensions_xhtml_review.rs` | Review expand + finish integration |
| `crates/wyvern-host/tests/report_review_finish.rs` | API contract tests |

### Finish contract (normative)

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

## Required validation

```bash
cargo test -p wyvern-cli extensions_xhtml_review
cargo test -p wyvern-host report_review_finish
rg -n 'wizard-nav' ui/shared/report-review.js share/wyvern/extensions.json  # expect no wizard-nav on report path
```

## Non-closure

- `workflow.post` persistence of review outcomes (callers/agents handle JSON)
- MCP tool wrapper — Phase E

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md) § Review finish JSON
