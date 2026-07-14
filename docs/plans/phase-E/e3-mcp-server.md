---
id: e.3
title: MCP server wrapper and tool mapping
status: planning
branch: feature/phase-E-e3-mcp-server
target: integrate/phase-E
---

# Phase E / e.3 — MCP server wrapper and tool mapping

## Status
pending

## Hard dependency

Phase C **c.16** complete; d.1–d.2 for wizard tools.

## Ownership (locked)

| Concern | Owner |
|---------|-------|
| MCP stdio transport | **`wyvern-mcp`** |
| Persistent HTTP + `HostSession` | **`wyvern-host`** |
| Optional embedded viewer | **`wyvern` CLI** (same spawn policy as one-shot) |

## Deliverables

- MCP stdio server in `wyvern-mcp`
- Tool map: `message`, `input`, `markdown`, `question`, `wizard`
- Persistent `HostSession` at startup (via `wyvern --mcp` entry)
- Each tool: `validate` → `run_dialog` → `DialogHandle::await_result`

## Acceptance criteria

- `wyvern --mcp` starts MCP server with persistent `HostSession`
- Each dialog type registered; schemas match CLI JSON
- Tool call → `HostSession::run_dialog` → `DialogHandle::await_result` → tool response JSON
- MCP CI uses `--viewer none` + HTTP client to complete dialogs
- `HostSession` has no viewer child; no `show`/`hide` MCP tools in MVP

## Required validation

```bash
cargo test -p wyvern-mcp
# Integration: MCP stdio harness with --viewer none + HTTP client
```

## Non-closure

- MCP persistent multi-call polish (e.4)
- `show`/`hide`/`exit` as MCP tools

## Authority

[http-interactive-mcp-contract.md](../phase-C/http-interactive-mcp-contract.md), [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
