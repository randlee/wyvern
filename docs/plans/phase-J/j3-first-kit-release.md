---
id: j.3
title: First kit-managed release
status: planning
branch: feature/phase-J-j3-first-release
worktree: ../wyvern-worktrees/feature/phase-J-j3-first-release
target: integrate/phase-J
depends_on: j.2
---

# Sprint j.3 — First kit-managed release

## Goal

Cut the **first production semver** using the sc-publish state machine on
`integrate/phase-J` → `main`. Uses real `vX.Y.Z` (kit accepts only
`X.Y.Z` / `vX.Y.Z`); burns a production version.

## Hard dependencies

- j.2 closeout: CR-001 and CR-002 **resolved** (upstream-tracking)
- `WINGET_GITHUB_TOKEN` and `SCOOP_BUCKET_TOKEN` provisioned (j.2 AC #8)
- Winget bootstrap complete if first package (j.2 AC #9)
- Scoop bucket repo ready if first manifest (j.2 AC #10)
- GitHub environment `crates-io` exists

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-J/.plan-hardening/first-release-record.md` | Run IDs, tag, per-channel outcomes |

## Acceptance criteria

1. `release-candidate.yml` dispatched; tag `release-candidate-vX.Y.Z` on branch containing kit workflows.
2. `release/vX.Y.Z` merged to **`main`** with version lockstep.
3. `release-preflight.yml` green on exact `main` SHA.
4. `release.yml` `target=production` creates `vX.Y.Z` and GitHub Release (kit asset names).
5. Archives contain `bin/wyvern`, `bin/wyvern-viewer`, `share/wyvern/ui/{message,input,markdown,question,chrome}/index.html`.
6. crates.io publish succeeds or detect-and-skips.
7. `homebrew-publish.yml` succeeds; `randlee/homebrew-tap` formula updated for `vX.Y.Z`.
8. `scoop-publish.yml` succeeds; `randlee/scoop-bucket` manifest updated for `vX.Y.Z`.
9. `winget-publish.yml` opens submission PR (skip-probe allowed **only** when version already submitted); auth failure = fail.
10. `install.json` omits `channels.pypi`: `pypi-publish.yml` is **not** dispatched;
    `channel-dispatch-plan` for tag `vX.Y.Z` lists no `pypi` channel; record PyPI
    **N/A** in `first-release-record.md`.
11. Re-dispatch one post-release leg; detect-and-skip works.
12. `first-release-record.md` documents every channel outcome (incl. PyPI N/A) and go/no-go for j.4.

## Non-closure (explicit)

- Microsoft winget install visibility may lag; submission success counts.
- Packaging bug found during release: fix on release branch, document in record.

## Required validation

```bash
gh release view vX.Y.Z --json assets
curl -fsS -A wyvern-check "https://crates.io/api/v1/crates/wyvern-cli/X.Y.Z"
gh run list --workflow winget-publish.yml --limit 3
gh run list --workflow scoop-publish.yml --limit 3
python3 .github/scripts/release_artifacts.py channel-dispatch-plan \
  --manifest release/publish-artifacts.toml --tag vX.Y.Z \
  | jq -e '([.channels[]?.name] // []) | index("pypi") | not'
```
