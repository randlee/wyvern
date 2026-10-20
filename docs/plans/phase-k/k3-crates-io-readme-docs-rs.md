---
id: k.3
title: crates.io README and documentation links (#62, #63)
status: planned
branch: feature/phase-K-k3-crates-io-metadata
worktree: ../wyvern-worktrees/feature/phase-K-k3-crates-io-metadata
target: integrate/phase-K
depends_on: k.2
github_issues: [62, 63]
---

# Sprint k.3 — crates.io packaging metadata (#62, #63)

## Goal

Fix **[#62](https://github.com/randlee/wyvern/issues/62)** and
**[#63](https://github.com/randlee/wyvern/issues/63)** for every crate with
`publish = true` in **`release/publish-artifacts.toml`**, then publish a patch
so crates.io reflects the fix.

## Hard dependencies

- k.2 merged to `integrate/phase-K`

## Published crate list (authoritative)

From `release/publish-artifacts.toml` (`publish = true`):

- `wyvern-schema`, `wyvern-wizard`, `wyvern-host`, `wyvern-viewer`, `wyvern-cli`
  (`wyvern-mcp` is **not** published)

## Deliverables

| Path | Purpose |
|------|---------|
| `Cargo.toml` / `crates/*/Cargo.toml` | README in package; `documentation` URLs |
| `scripts/check-crates-publish-manifest.sh` | CI: `cargo package` includes README |
| `docs/release/version-policy.md` | Record **0.7.1** vs defer-to-**0.8.0** decision |
| `docs/plans/phase-k/k3-crates-io-readme-docs-rs.md` | This sprint doc |

## Explicit code samples

```toml
[workspace.package]
documentation = "https://docs.rs/wyvern-cli"
```

```toml
[package]
name = "wyvern-schema"
documentation = "https://docs.rs/wyvern-schema"
readme = "README.md"
include = ["src/**", "Cargo.toml", "README.md"]
```

## Required work

1. Audit **only** crates with `publish = true` in `release/publish-artifacts.toml`.
2. Fix `include` / crate-local README so `cargo package` ships non-empty README.
3. Bump workspace to **0.7.1** (patch) by default for crates.io metadata-only fix (record in
   `docs/release/version-policy.md`).
4. Post-merge: dispatch **`crates-publish.yml`** for tag **v0.7.1** (document in PR).

## Non-closure

- Closing #62/#63 before crates.io shows README + Documentation link post-publish.

## Acceptance criteria

1. `cargo package -p <each published crate> --allow-dirty` — README present in tarball.
2. Every published crate has valid `documentation = "https://docs.rs/<name>"`.
3. `scripts/check-crates-publish-manifest.sh` in CI (or equivalent gate).
4. **`docs/release/version-policy.md`** records version bump (**0.7.1** default) and
   publish dispatch requirement.
5. **#62 and #63 remain open** until post-merge crates-publish succeeds; close with
   crates.io verification note in same PR as publish or follow-up comment.

## Validation

```bash
scripts/check-crates-publish-manifest.sh
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Post-merge: `gh workflow run crates-publish.yml` after **v0.7.1** tag.
