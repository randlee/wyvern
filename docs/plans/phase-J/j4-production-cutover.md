---
id: j.4
title: Integrate phase-J to develop
status: planning
branch: feature/phase-J-j4-integrate
worktree: ../wyvern-worktrees/feature/phase-J-j4-integrate
target: develop
depends_on: j.3
---

# Sprint j.4 — Integrate phase-J to develop

## Goal

Merge `integrate/phase-J` → `develop` after first kit release (j.3). Verify no
legacy tag-push release triggers remain. Open `main` → `develop` back-merge.

## Hard dependencies

- j.3 `first-release-record.md` go/no-go = **go**
- CR-001/CR-002 **resolved** (never waived)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-J/.plan-hardening/integrate-record.md` | Merge PR, workflow grep evidence |

## Paths to delete

None — j.1 removed legacy publish files; kit `release.yml` has no tag-push trigger.

## Acceptance criteria

1. `integrate/phase-J` merged to `develop` after phase-end QA PASS.
2. **No** file in `.github/workflows/` on `develop` contains `push:` + `tags:` release trigger (grep all workflows).
3. j.3 production tag and channels remain verified (no regression).
4. Release notes on j.3 tag document archive rename (if not already).
5. `main` → `develop` back-merge PR opened (publisher policy).

## Non-closure (explicit)

- **j.4 does not** merge back-merge PR.

## Required validation

```bash
! rg -l 'tags:\s*\n\s*-\s*"v\*"' .github/workflows/ 2>/dev/null
! rg 'push:' .github/workflows/*.yml -A2 | rg 'tags:'
```

If Homebrew channel active: `brew info wyvern` shows j.3 version.
If Scoop channel active: `curl -fsS "https://raw.githubusercontent.com/randlee/scoop-bucket/main/bucket/wyvern.json" | rg '"version"'` matches j.3 version.
