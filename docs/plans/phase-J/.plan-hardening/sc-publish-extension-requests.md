# sc-publish extension requests (wyvern → org release)

**Audience:** atm-core / sc-publish maintainers  
**PR under review:** [sc-publish #63](https://github.com/randlee/sc-publish/pull/63) @ `928c8f9` (`chore/reconcile-main-develop`)

## Summary

Wyvern needs **no new `install.json` schema fields** beyond what PR #63 already
includes. All wyvern product slots fit existing consumer extension points.

## B1–B4 status (incorporated in PR #63 @ `928c8f9`)

| ID | Request | Status |
|----|---------|--------|
| B1 | `release/sc-publish-pin.toml.example` + README pin contract | **In PR #63** (ships via `package_files()`) |
| B2 | Isolated-clone bootstrap recipe | **In PR #63** README |
| B3 | Input-path convention (`install.json` vs `sc-publish-consumer-input.json`) | **In PR #63** README |
| B4 | Qualification checklist before pin advance | **In PR #63** README |

C1 (reusable workflows) and C2 (gitignored-only materialization) remain **deferred** by wyvern this cycle; vendor-and-pin stays the model.

## Explicitly not requested

- `org-destinations.toml` install enforcement (use `install.json` slots)
- Per-repo kit forks or wyvern-only workflow edits
- PyPI channel (omitted until Python bindings exist)

## Wyvern pin policy

- **Current:** `42e0fce` (atm-core AT.2 qualified) via `release/sc-publish-pin.toml`
- **Next:** org blessed SHA after PR #63 merge + multi-repo qualification (candidate **`928c8f9`**)
- **Never:** track `sc-publish` `main` unilaterally

## Wyvern consumer-owned (not kit)

- `release/install.json`
- `release/sc-publish-pin.toml`
- `scripts/sync-sc-publish.sh` (isolated `.sc-publish-kit/` cache)
- `release/homebrew/*.j2`, `release/scoop/*.j2`
