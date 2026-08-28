# Phase-J integrate record (j.4)

**Status:** complete  
**Phase integrate PR:** [#147](https://github.com/randlee/wyvern/pull/147), [#151](https://github.com/randlee/wyvern/pull/151) (`integrate/phase-J` → `develop`)  
**First release record:** [first-release-record.md](first-release-record.md) go/no-go = **go**

## j.4 acceptance criteria

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | `integrate/phase-J` merged to `develop` | **done** | PR #147 merged 2026-08-28 |
| 2 | No tag-push release triggers | **pass** | `release.yml` is `workflow_dispatch` only; grep below |
| 3 | j.3 production tag/channels verified | **pass** | `v0.6.0`; see first-release-record |
| 4 | Release notes document archive rename | **pass** | `release/release-notes.md` on tag assets |
| 5 | `main` → `develop` back-merge PR opened | **done** | [#150](https://github.com/randlee/wyvern/pull/150) (open per j.4 non-closure) |

## Workflow grep (no legacy tag-push release)

```bash
# Executed on integrate/phase-J @ 21e9ee5
rg 'tags:' .github/workflows/   # no matches
rg 'push:' .github/workflows/*.yml -A3 | rg 'tags:'   # no matches
```

`ci.yml` and `pages.yml` use branch `push:` only (`develop`, `main`) — not tag triggers.

## Back-merge

Publisher policy: open `main` → `develop` after production cut to land release-branch fixes and tag metadata on develop.
