---
id: j.3
title: Kit release rehearsal
status: planning
branch: feature/phase-J-j3-rehearsal
worktree: ../wyvern-worktrees/plan/phase-J-j3-rehearsal
target: integrate/phase-J
depends_on: j.2
---

# Sprint j.3 — Kit release rehearsal

## Goal

Execute one full sc-publish release drill on a rehearsal version without
surprising production download consumers. Prove candidate → `main` → preflight →
release dispatch → assets → post-release channel legs.

## Hard dependencies

- j.2 closeout: CR-001/CR-002 resolved **or** signed waiver
- `WINGET_GITHUB_TOKEN` provisioned (j.2 AC #7)
- GitHub environment `crates-io` exists

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-J/.plan-hardening/rehearsal-record.md` | Run IDs, tag, channel outcomes, waiver notes |

No product code changes expected unless rehearsal exposes packaging bug (fix in-place, document in record).

## Acceptance criteria

1. `release-candidate.yml` dispatched; tag `release-candidate-vX.Y.Z` on `origin/develop`.
2. Branch `release/vX.Y.Z` merged to **`main`** with version lockstep.
3. `release-preflight.yml` green on exact `main` SHA.
4. `release.yml` `target=production` creates `vX.Y.Z` and GitHub Release with kit asset names.
5. Each archive contains `bin/wyvern`, `bin/wyvern-viewer`, and `share/wyvern/ui/{message,input,markdown,question,chrome}/index.html`.
6. crates.io publish leg succeeds or detect-and-skips already-published crates.
7. `homebrew-publish.yml` succeeds **or** waived per j.2 closeout (failure without waiver = sprint fail).
8. `winget-publish.yml` reaches submission (PR opened or confirmed skip probe); missing token = sprint fail.
9. Re-dispatch one post-release leg; detect-and-skip prevents duplicate publish.
10. `rehearsal-record.md` lists all workflow run URLs and go/no-go for j.4.

## Non-closure (explicit)

- Microsoft winget **install visibility** is not required same-day; submission success counts (per `WINGET_SETUP.md`).
- Rehearsal version may be pre-release bump; need not be the final marketing version.

## Required validation

Evidence in `rehearsal-record.md`:

```bash
gh release view vX.Y.Z --json assets
curl -fsS -A wyvern-check "https://crates.io/api/v1/crates/wyvern-cli/X.Y.Z"
gh run list --workflow winget-publish.yml --limit 3
```
