# sc-publish extension requests (wyvern → org release)

**Audience:** atm-core / sc-publish maintainers  
**PR under review:** [sc-publish #63](https://github.com/randlee/sc-publish/pull/63) (`chore/reconcile-main-develop`)

## Summary

Wyvern needs **no new `install.json` schema fields** beyond what PR #63 already
includes. All wyvern product slots fit existing consumer extension points.

## Not in PR #63 — recommended before org-wide pin bump

| ID | Request | Type |
|----|---------|------|
| B1 | `release/sc-publish-pin.toml.example` + README pin contract | Docs |
| B2 | Bootstrap recipe: clone kit @ pin to local cache; never mutate shared sibling | Docs |
| B3 | Document that `--input` path is consumer choice (`install.json` vs `sc-publish-consumer-input.json`) | Docs |
| B4 | AT-style qualification checklist before pin advance | Docs / optional script |

## Explicitly not requested

- `org-destinations.toml` install enforcement (use `install.json` slots)
- Per-repo kit forks or wyvern-only workflow edits
- PyPI channel (omitted until Python bindings exist)

## Wyvern pin policy

- **Current:** `42e0fce` (atm-core AT.2 qualified) via `release/sc-publish-pin.toml`
- **Next:** org blessed SHA after PR #63 merge + multi-repo qualification (target `4ce6aac`)
- **Never:** track `sc-publish` `main` unilaterally

## Wyvern consumer-owned (not kit)

- `release/install.json`
- `release/sc-publish-pin.toml`
- `scripts/sync-sc-publish.sh` (isolated `.sc-publish-kit/` cache)
- `release/homebrew/*.j2`, `release/scoop/*.j2`
