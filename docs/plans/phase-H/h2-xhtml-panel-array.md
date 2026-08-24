---
id: h.2
title: XHTML panel array (basic multi-pane frame)
status: planning
branch: feature/phase-H-h2-xhtml-panel-array
worktree: ../wyvern-worktrees/feature/phase-H-h2-xhtml-panel-array
target: integrate/phase-H
---

# Sprint h.2 — XHTML panel array

## Goal

Display an ordered array of XHTML panels in one scrollable **basic** report frame
for ad-hoc agent review (failures + proposed fix side-by-side).

## Hard dependencies

- h.1 merged to `integrate/phase-H` (`type: "report"`, preexec script shell)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-H/h2-xhtml-panel-array.md` | This sprint doc |
| `share/wyvern/extensions.json` | `report-xhtml` extension only |
| `crates/wyvern/src/extensions/catalog.rs` | `expands_to` reads `type` from emitted command JSON (not default `wizard`) |
| `crates/wyvern/src/cli_args.rs` | `usage_message()` — `wyvern report-xhtml <manifest.json>` line |
| `crates/wyvern/tests/extensions_catalog.rs` | REQ-0137 parity for `report-xhtml` (`description`, `examples`, prefix) |
| `scripts/ext/xhtml_report.py` | Extends h.1 script: `--mode array`, manifest reader, `--validate-manifest`; writes `{tmpdir}/report-command.json` |
| `docs/plans/phase-H/review-manifest.schema.json` | JSON schema for manifests |
| `crates/wyvern/tests/fixtures/xhtml-review/view.json` | Minimal 2-panel **test-only** fixture (h.5 ships separate examples under `share/`) |
| `crates/wyvern/tests/extensions_xhtml_array.rs` | Manifest → expand tests |
| `ui/shared/report-base.css` | `.pane`, `.pane--proposal` styles |

### Manifest schema (normative)

See [xhtml-reporting-contract.md](xhtml-reporting-contract.md) § Manifest.
Schema file at `review-manifest.schema.json` for CI validation.

### CLI

```bash
wyvern report-xhtml path/to/review.json
```

Manifest `mode` defaults to `"view"`. Preexec writes `{tmpdir}/pages/view.xhtml`
using **basic-array** frame profile.

## Acceptance criteria

1. Manifest with ≥2 `.xhtml` panels opens one report view; all panes visible in
   document order.
2. `role: "proposal"` panes receive distinct styling (`.pane--proposal`).
3. Missing panel path → preexec non-zero; stderr names missing file; no host launch.
4. Single-panel manifest works (degenerate array of 1).
5. `wyvern report-xhtml --help` documents manifest path + manifest fields.
6. `extensions list --json` for `report-xhtml` shows `expands_to: "report"` (never `wizard`).
7. `{path}` binds manifest file via `arg_suffix: ".json"`; non-`.json` token after prefix does not match.

## Required validation

```bash
cargo test -p wyvern-cli --test extensions_xhtml_array
cargo test -p wyvern-cli --test extensions_catalog req_0137
python3 -m json.tool docs/plans/phase-H/review-manifest.schema.json >/dev/null
python3 scripts/ext/xhtml_report.py --validate-manifest crates/wyvern/tests/fixtures/xhtml-review/view.json
WYVERN_VIEWER=none wyvern report-xhtml crates/wyvern/tests/fixtures/xhtml-review/view.json
wyvern extensions list --json | jq -e '.[] | select(.id=="report-xhtml") | .expands_to == "report"'
```

## Non-closure

- `report-xhtml-review` extension registration (h.3 sole owner)
- `--review` / Approve-Cancel (h.3)
- Directory glob without manifest
- Paginated one-pane-at-a-time mode

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md)
