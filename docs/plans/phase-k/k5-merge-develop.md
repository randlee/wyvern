---
id: k.5
title: Merge integrate/phase-K to develop
status: planned
branch: feature/phase-K-k5-merge-develop
worktree: ../wyvern-worktrees/feature/phase-K-k5-merge-develop
target: develop
depends_on: k.4
---

# Sprint k.5 — Phase K integration merge

## Goal

Merge **`integrate/phase-K`** → **`develop`** after k.1–k.4 PASS QA + CI.

## Hard dependencies

- k.4 merged; phase-ending QA PASS
- k.3 crates-publish completed if **0.7.1** path chosen (`docs/release/version-policy.md`)

## Deliverables

| Path | Purpose |
|------|---------|
| PR `integrate/phase-K` → `develop` | Integration merge |
| `CHANGELOG.md` | **0.8.0** section (product features from Phase K) + **0.7.1** if shipped in k.3 |
| `docs/plans/phase-k/.plan-hardening/integrate-record.md` | Merge SHA, PR URL |
| `site/announcements/wyvern-v0.8.0.md` | Phase K announcement |

## Non-closure

- `site/announcements/wyvern-v0.8.0.md` omitted only if phase scope shrinks to patch-only
  with no minor bump — must match `docs/release/version-policy.md`.

## Acceptance criteria

1. Merge PR green; `develop` contains `site/cli/`.
2. #141 closed; #62/#63 closed post-crates.io verify (from k.3).
3. Version policy in repo (`docs/release/version-policy.md`), not PR-body-only.
4. Phase K acceptance criteria in [plan-phase-k.md](plan-phase-k.md) satisfied.

## Validation

```bash
gh pr checks <merge-pr> --watch
gh issue list --repo randlee/wyvern --state open --label release/0.8.0
```

## Cursor orchestration

| Field | Value |
|-------|--------|
| `pr_target` | `develop` |
| `sprint_doc` | `docs/plans/phase-k/k5-merge-develop.md` |
| Dev agent | none (merge-only; coordinator + `gh`) |
| QA-1 reviewers | req-qa, arch-qa, rust-qa-agent (Terra if available), flaky-test-qa — phase-ending gate |
