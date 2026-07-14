---
id: c.15
title: wyvern-viewer + browser registry
status: in_progress
branch: feature/phase-C-c15-wyvern-viewer
target: integrate/phase-c-web-server
---

# Sprint c.15 — `wyvern-viewer` + browser registry

## Goal

Presentation layer: **embedded default**, named system browsers via local registry.

## Hard dependencies

- **c.14** merged (full dialog matrix)

## Ownership (locked)

| Concern | Owner |
|---------|-------|
| `wyvern-viewer` crate (wry URL-only) | **`wyvern-viewer`** |
| Spawn embedded child process | **`wyvern` CLI** — `boundaries/wyvern/cli.toml` (`embedded_viewer_spawn`); host forbids `embedded_viewer_spawn` |
| System + named browser launch | **`wyvern-host`** `browser_launch.rs` + registry |
| Show / hide embedded window | **`wyvern` CLI** → **`wyvern-viewer`** (not `HostSession`) |
| Viewer OS-close → dismissed | **`wyvern-viewer`** POST + **`wyvern` CLI** child-exit watch — [http-viewer-contract.md](http-viewer-contract.md) |
| Browser cache file | **`wyvern-host`** `browser_registry.rs` |

## Deliverables

- `crates/wyvern-viewer/` — navigate to `WYVERN_DIALOG_URL`; transparent chrome attrs; viewer-close dismiss POST per [http-viewer-contract.md](http-viewer-contract.md)
- `boundaries/wyvern-viewer/viewer.toml` enforced in CI
- **Embedded one-shot pipeline:** CLI uses `wyvern_host::begin` → `embedded_viewer_spawn` → `await_result` — **not** `wyvern-host::run` (reserved for `none`/`system`/`named`)
- `wyvern` spawns viewer for `--viewer embedded` — **binary discovery** (sibling → `CARGO_BIN_EXE` → `WYVERN_VIEWER_BIN` → `PATH`)
- `boundaries/wyvern/cli.toml` — `embedded_viewer_spawn`, `viewer_show_hide` in `io_owns`
- `wyvern-host`: `browser_catalog.rs`, `browser_registry.rs`, `browser_launch.rs` — `system` + named launch only (inside `run` / `run_dialog`)
- `wyvern browsers list` / `wyvern browsers refresh` subcommands
- Product default: `--viewer embedded` when flag omitted

## Acceptance criteria

1. **Embedded:** `wyvern '{"type":"message",...}'` (no flag) — embedded viewer; OK → stdout JSON
2. **Headless:** `--viewer none` + full Playwright matrix — unchanged CI path passes
3. **Named browser:** `--viewer chrome` when installed; `HostError::ViewerNotFound` when not
4. All three surfaces above required in one sprint — partial viewer modes do not close c.15
5. `wyvern browsers list` prints cache entries
6. Missing `wyvern-viewer` binary → `HOST_VIEWER_ERROR` with install hint (not silent `none` fallback)
7. OS-close embedded viewer without button click → `{ "button": "dismissed" }` on stdout (REQ-0097)

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
sc-lint check native --config .sc-lint.toml
# L2 full matrix with --viewer none (unchanged)
# Manual macOS: wyvern '{"type":"message",...}'  # default embedded
wyvern browsers list
wyvern browsers refresh
```

## Non-closure

- Release tarball + v0.1.0 tag (c.16)

## Authority

- [http-viewer-contract.md](http-viewer-contract.md), [HTTP-TYPES.md](HTTP-TYPES.md)
- ADR-0019
- `boundaries/wyvern-host/host.toml` — no `webview_creation`, no `embedded_viewer_spawn`
- `boundaries/wyvern/cli.toml` — `embedded_viewer_spawn`
