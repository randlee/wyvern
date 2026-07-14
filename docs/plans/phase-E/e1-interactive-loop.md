---
id: e.1
title: Interactive stdin loop and lifecycle actions
status: planning
branch: feature/phase-E-e1-interactive-loop
target: integrate/phase-E
---

# Sprint e.1 — `--interactive` stdin loop and lifecycle actions

## Goal

Persistent `HostSession` with lifecycle `show`/`hide`/`exit` — no blocking dialogs yet. Viewer spawn/show/hide owned by **`wyvern` CLI**, not host.

## Hard dependencies

- Phase C **c.16** complete

## Ownership (locked)

| Concern | Owner |
|---------|-------|
| Stdin readline loop, arg parsing | **`wyvern` CLI** |
| `HostSession`, HTTP server | **`wyvern-host`** |
| `wyvern-viewer` subprocess | **`wyvern` CLI** (`embedded_viewer_spawn`) |
| Lifecycle `show`/`hide`/`exit` | **`wyvern` CLI** → viewer subprocess + `HostSession::shutdown` on exit |

## Deliverables

- `HostSession::new` in `wyvern-host` — persistent HTTP listener (no viewer child field)
- `wyvern --interactive` stdin readline loop
- Lifecycle actions: `show`, `hide`, `exit` per [http-interactive-mcp-contract.md](../phase-C/http-interactive-mcp-contract.md)
- CLI spawns optional `wyvern-viewer` once per session (`--viewer embedded` default on desktop)
- `cargo test -p wyvern` — lifecycle action tests

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `wyvern --interactive` starts persistent `HostSession` + CLI-owned optional `wyvern-viewer`
3. `{"action":"hide"}` and `{"action":"show"}` toggle viewer via CLI; host HTTP stays up
4. Lifecycle actions return `{"action":"...","ok":true}`
5. Loop remains alive after lifecycle actions
6. `HostSession` has **no** `wyvern-viewer` child field — spawn/show/hide owned by CLI

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host session_lifecycle
cargo test -p wyvern interactive_lifecycle
# Manual: wyvern --interactive → show/hide/exit sequence
```

## Non-closure

- Blocking dialogs in loop (e.2)
- MCP server (e.3–e.4)

## Authority

- [http-interactive-mcp-contract.md](../phase-C/http-interactive-mcp-contract.md), [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
