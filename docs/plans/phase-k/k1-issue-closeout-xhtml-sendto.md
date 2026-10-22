---
id: k.1
title: Regression tests for #115 / #139 (issues already closed)
status: complete
branch: feature/phase-K-k1-issue-closeout
worktree: ../wyvern-worktrees/feature/phase-K-k1-issue-closeout
target: integrate/phase-K
depends_on: none
github_issues: [115, 139]
---

# Sprint k.1 — XHTML suffix + Send-To contract regression

## Goal

Add **regression tests** and **`issue-closeout-evidence.md`** mapping closed issues
[#115](https://github.com/randlee/wyvern/issues/115) and
[#139](https://github.com/randlee/wyvern/issues/139) to concrete tests and CI jobs.
Issues are already closed on GitHub; k.1 does **not** re-close them.

## Hard dependencies

- `develop` @ v0.7.0+
- Phase H `xhtml-suffix`; CI job `atm-send-to-contract`

## Deliverables

| Path | Purpose |
|------|---------|
| `crates/wyvern/tests/extensions_xhtml_suffix.rs` | `.xhtml` suffix routing + expansion smoke |
| `docs/plans/phase-k/issue-closeout-evidence.md` | Matrix: issue → test name → CI job URL |
| `docs/plans/phase-k/k1-issue-closeout-xhtml-sendto.md` | This sprint doc |

## Required work

1. **#115:** Fixture under repo; assert `xhtml-suffix` matches; headless path shows
   `"type":"report"` in expanded command (no `PARSE_ERROR` / “No shipped extension matches”).
2. **#139:** Evidence doc maps each issue #139 checklist row to `atm-send-to-contract`
   steps and pinned ref in `release/atm-core-send-to-pin.toml`.
3. Share sync unchanged unless regression fails `scripts/check-share-sync.sh`.

## Non-closure

- Re-opening or re-closing GitHub issues (already closed pre-k.1).

## Acceptance criteria

1. `WYVERN_VIEWER=none wyvern <fixture>.xhtml` exits **0**; expanded JSON includes
   `"type":"report"` (no `PARSE_ERROR`, no “No shipped extension matches”).
2. Rust test **`extensions_xhtml_suffix`** (or named equivalent) registered in CI.
3. Sprint PR **`atm-send-to-contract`** job green.
4. **`issue-closeout-evidence.md`** maps #115 and #139 each to: (a) test or job name
   added/verified this sprint, (b) CI workflow run URL from the sprint PR.

## Validation

```bash
cargo test -p wyvern-cli extensions_xhtml_suffix
bash scripts/verify_atm_core_send_to_contract.sh
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
