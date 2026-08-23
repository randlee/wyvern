---
id: h.5
title: Synthetic XHTML review example + CI
status: planning
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
| `docs/plans/phase-H/h5-synthetic-xhtml-example.md` | This sprint doc |

### Synthetic data rules

- Panels MUST mimic atm-core benchmark fragment shape (`<section xmlns="http://www.w3.org/1999/xhtml" class="benchmark-run">`).
- Data MUST be **fabricated** (no live atm-core paths required).
- `proposed-fix.xhtml` MUST differ visibly from fail panels (status PASS, revised
  numbers) so manual review demos are obvious.

### CI job (add to existing workflow)

After `cargo build -p wyvern-cli`:

```bash
wyvern wizard lint share/wyvern/examples/xhtml-review  # nav lint N/A — expect skip or clean if lint extended; document outcome
wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json  # expand-only or --viewer none smoke
python3 scripts/ext/xhtml_report.py --validate-manifest share/wyvern/examples/xhtml-review/review-view.json
```

If wizard lint does not apply to report examples, document explicit exclusion in
README and skip lint step in CI (report ≠ wizard).

## Acceptance criteria

1. `wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json`
   exits 0 with `--viewer none`.
2. `wyvern report-xhtml --review …/review-review.json` expand test passes; finish
   API exercised in host integration test with synthetic HTML fixture.
3. README documents single-panel shortcut:
   `wyvern share/wyvern/examples/xhtml-review/panels/fail-1.xhtml`.
4. h.4 skill links to this example tree.
5. Embedded share sync check passes.

## Required validation

```bash
scripts/check-share-sync.sh
cargo test -p wyvern-cli examples_xhtml_review
cargo test -p wyvern-cli extensions_xhtml_single extensions_xhtml_array extensions_xhtml_review
```

## Non-closure

- Welcome guide topic for xhtml-review (optional follow-up)
- Copying live atm-core report directories into wyvern repo

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md)
- [h4-wyvern-reporting-skill.md](h4-wyvern-reporting-skill.md)
