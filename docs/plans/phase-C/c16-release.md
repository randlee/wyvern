---
id: c.16
title: Release bundle + v0.1.0 tag
status: ready-for-tag
branch: feature/phase-C-c16-release
target: integrate/phase-c-web-server
---

# Sprint c.16 — Release + v0.1.0 (final Phase C delivery sprint)

## Goal

Ship release tarball with full `share/wyvern/ui/` + `wyvern-viewer`; valid **v0.1.0** on HTTP stack.

> **Sole v0.1 authority:** This sprint is the **only** authoritative v0.1.0 delivery gate. [c5-release.md](c5-release.md) is **historical** workflow baseline only — its tag/AC do not apply until c.16.

## Hard dependencies

- c.10–c.15 complete

## Deliverables

- Release workflow packages `share/wyvern/ui/**` + **`wyvern-viewer` binary** beside `wyvern` (per [http-viewer-contract.md](http-viewer-contract.md) binary discovery)
- README quickstart: HTTP host, embedded default, `WYVERN_VIEWER=none` for CI
- `CHANGELOG.md` — delivery rewrite entry
- Tag `v0.1.0` from release artifact smoke (**post-merge on `integrate/phase-c-web-server`**)

## Acceptance criteria

1. Release tarball: `wyvern` + `wyvern-viewer` + full `ui/` tree (all five types)
2. Tag push triggers macOS/Windows/Linux matrix build
3. `integrate/phase-c-web-server` head: full CI green (build, clippy, sc-lint, Playwright, cargo-audit)
4. Manual macOS smoke: embedded viewer on release binary (checklist: [macos-embedded-viewer-smoke.md](macos-embedded-viewer-smoke.md))

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
sc-lint check native --config .sc-lint.toml
# CI Playwright matrix --viewer none
# Release: gh workflow after tag v0.1.0
```

## Non-closure

- Wizard (Phase D), `--interactive` / MCP (Phase E)

## Authority

- [c5-release.md](c5-release.md) — **historical** workflow baseline only
- [README.md](README.md#phase-acceptance-criteria-smoke--delivery-rewrite-c16)

**Final Phase C sprint.** Phase D/E blocked until c.16 merges. Status `ready-for-tag` = release prep validated on `release/v0.1.0`; bump to `complete` after `v0.1.0` tag + release smoke.
