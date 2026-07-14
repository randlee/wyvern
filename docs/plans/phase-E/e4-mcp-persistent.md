---
id: e.4
title: MCP persistent host and integration testing
status: planning
branch: feature/phase-E-e4-mcp-persistent
target: integrate/phase-E
---

# Phase E / e.4 — MCP persistent host and integration testing

## Status
pending

## Hard dependency

**e.3** merged.

## Ownership (locked)

| Concern | Owner |
|---------|-------|
| Persistent `HostSession` across tool calls | **`wyvern-host`** |
| Viewer reuse (embedded desktop) | **`wyvern` CLI** — one subprocess, navigate per dialog |
| Lifecycle show/hide | **`wyvern` CLI** only — not MCP MVP tools |

## Deliverables

- Multi-tool-call integration test: host survives between MCP invocations
- Headless CI path: `--viewer none` + HTTP client
- `docs/mcp-setup.md` — register Wyvern as MCP server

## Acceptance criteria

- `HostSession` persists across MCP tool calls; CLI-owned viewer optional on desktop
- Blocking dialog tools keep normal CLI semantics; tool response JSON matches stdout shape
- Repo-owned MCP stdio harness passes multi-tool-call flow with `--viewer none` (required CI gate)
- `docs/mcp-setup.md` documents registration steps (Claude Code registration = optional manual smoke only)

## Required validation

```bash
cargo test -p wyvern-mcp persistent_session
# CI: MCP stdio harness — multiple tool calls, --viewer none
```

## Non-closure

- MCP lifecycle tools (`show`/`hide`/`exit`) — remain `--interactive` only

## Authority

[http-interactive-mcp-contract.md](../phase-C/http-interactive-mcp-contract.md)
