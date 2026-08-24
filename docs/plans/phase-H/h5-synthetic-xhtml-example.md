---
id: h.5
title: Synthetic XHTML review example + CI
status: complete
branch: feature/phase-H-h5-xhtml-example
worktree: ../wyvern-worktrees/feature/phase-H-h5-xhtml-example
target: integrate/phase-H
---

# Sprint h.5 — Synthetic example + CI smoke

## Goal

Ship a **working** synthetic example (atm-core-style benchmark panels) under
`share/wyvern/examples/` so skills, docs, and CI can point at real artifacts.

## Hard dependencies

- h.1–h.3 merged (all three frame profiles)
- h.4 skill may merge in parallel but example paths MUST appear in skill refs
  before phase closes

## Deliverables

| Path | Purpose |
|------|---------|
| `share/wyvern/examples/xhtml-review/panels/fail-1.xhtml` | Synthetic failure fragment |
| `share/wyvern/examples/xhtml-review/panels/fail-2.xhtml` | Synthetic failure fragment |
| `share/wyvern/examples/xhtml-review/panels/fail-3.xhtml` | Synthetic failure fragment |
| `share/wyvern/examples/xhtml-review/panels/proposed-fix.xhtml` | Synthetic proposal fragment |
| `share/wyvern/examples/xhtml-review/review-view.json` | View-mode manifest (3+1 panels) |
| `share/wyvern/examples/xhtml-review/review-review.json` | Review-mode manifest |
| `share/wyvern/examples/xhtml-review/README.md` | Operator walkthrough |
| `crates/wyvern/embedded/share/wyvern/examples/xhtml-review/` | Parity |
| `.github/workflows/ci.yml` | `report-xhtml` smoke job steps |
| `crates/wyvern/tests/examples_xhtml_review.rs` | Integration against synthetic tree |
| `.claude/skills/wyvern-reporting/SKILL.md` | Link `share/wyvern/examples/xhtml-review/` paths (h.5 owner) |
| `.claude/skills/wyvern-reporting/references/core/review-manifest.md` | Example manifest paths under `share/` |
| `.cursor/skills/wyvern-reporting/SKILL.md` | Cursor stub — same example links |
| `docs/plans/phase-H/h5-synthetic-xhtml-example.md` | This sprint doc |

### Synthetic data rules

- Panels MUST mimic atm-core benchmark fragment shape (`<section xmlns="http://www.w3.org/1999/xhtml" class="benchmark-run">`).
- Data MUST be **fabricated** (no live atm-core paths required).
- `proposed-fix.xhtml` MUST differ visibly from fail panels (status PASS, revised
  numbers) so manual review demos are obvious.

### CI job (add to existing workflow)

After `cargo build -p wyvern-cli`:

```bash
# Report examples MUST NOT use wizard-nav (contract § Boundaries)
test -z "$(rg -n 'wizard-nav' share/wyvern/examples/xhtml-review/ || true)"
WYVERN_VIEWER=none wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json
python3 scripts/ext/xhtml_report.py --validate-manifest share/wyvern/examples/xhtml-review/review-view.json
```

Report surfaces are excluded from `wyvern wizard lint` (not wizard packages). CI verifies
no `wizard-nav` references under `share/wyvern/examples/xhtml-review/`.

## Acceptance criteria

1. `wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json`
   exits 0 with `--viewer none`.
2. `wyvern report-xhtml --review …/review-review.json` expand test passes; finish
   API exercised by `cargo test -p wyvern-host report_review_finish` (h.3 fixture) **and**
   `cargo test -p wyvern-cli --test examples_xhtml_review` against the share example tree (h.5).
3. README documents single-panel shortcut:
   `wyvern share/wyvern/examples/xhtml-review/panels/fail-1.xhtml`.
4. h.5 updates `wyvern-reporting` skill refs to this example tree (h.4 AC #1 may cite
   conceptual/template paths only until h.5 lands).
5. Embedded share sync check passes.
6. `rg wizard-nav share/wyvern/examples/xhtml-review/` returns no matches (report ≠ wizard boundary).

## Required validation

```bash
scripts/check-share-sync.sh
cargo test -p wyvern-cli --test examples_xhtml_review
cargo test -p wyvern-cli --test extensions_xhtml_single
cargo test -p wyvern-cli --test extensions_xhtml_array
cargo test -p wyvern-cli --test extensions_xhtml_review
```

## Non-closure

- Welcome guide topic for xhtml-review (optional follow-up)
- Copying live atm-core report directories into wyvern repo

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md)
- [h4-wyvern-reporting-skill.md](h4-wyvern-reporting-skill.md)
