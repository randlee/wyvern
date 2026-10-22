---
id: k.2
title: checksums.txt release contract and #141 closeout
status: complete
branch: feature/phase-K-k2-checksums
worktree: ../wyvern-worktrees/feature/phase-K-k2-checksums
target: integrate/phase-K
depends_on: k.1
github_issues: [141]
---

# Sprint k.2 — `checksums.txt` contract (#141)

## Goal

Close **[#141](https://github.com/randlee/wyvern/issues/141)** with documented kit
contract, automated test coverage, and an explicit **legacy v0.5.0** policy decision.

## Hard dependencies

- k.1 merged to `integrate/phase-K`

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/release-checksums.md` | Consumer contract (format, asset coverage, atm-core cross-check) |
| `docs/plans/phase-k/issue-closeout-evidence.md` | Add #141 policy decision + atm-core pin finding |
| `.github/scripts/tests/test_release_artifacts.py` | Assert checksum step covers manifest assets |
| `docs/plans/phase-k/k2-checksums-release-contract.md` | This sprint doc |

## Required work

1. Verify v0.7.0+ releases include `checksums.txt` for all kit binary archives.
2. Cross-link from `release/install.json` or `docs/RELEASE_SECRETS.md` → `docs/release-checksums.md`.
3. **Before close:** inspect atm-core bootstrap pins (or `release/atm-core-send-to-pin.toml`
   + atm-core docs) for legacy names (`wyvern-windows.zip`, etc.). Record finding in
   `issue-closeout-evidence.md` and choose **backfill** vs **wontfix** with rationale.
4. Close #141 with link to `docs/release-checksums.md` §contract and v0.7.0 example.

## Acceptance criteria

1. `docs/release-checksums.md` exists; lists sha256 line format and asset coverage rule.
2. Automated test fails if a manifest asset pattern lacks checksum generation.
3. **Policy AC:** `issue-closeout-evidence.md` records atm-core legacy pin check result
   and the chosen #141 close policy (not conditional prose).
4. #141 closed with URL to `docs/release-checksums.md` and v0.7.0 `checksums.txt` snippet.
5. `release-preflight.yml` / release gate CI green (no regressions from k.2 changes).

## Validation

```bash
python3 -m pytest .github/scripts/tests/test_release_artifacts.py -q
gh release view v0.7.0 --repo randlee/wyvern --json assets --jq '.assets[].name'
```
