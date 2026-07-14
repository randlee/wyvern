# Phase E — Interactive & MCP (`integrate/phase-E`)

Phase E implementation PRs target **`integrate/phase-E`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Sprints are **sequentially numbered** `e.1` → `e.4` (strict dependency order — not parallel sub-sprints).

Each individual sprint doc (`e1`–`e4`) is the **sole authority** for that sprint's deliverables, acceptance criteria, and required validation.

## Phase goal

Wyvern runs as a persistent process, driveable by agents over stdin or as an MCP server.

## Phase acceptance criteria (smoke)

A Claude Code agent can open Wyvern in `--interactive` mode from a background shell, issue multiple blocking dialog commands against one persistent process, receive the JSON results, and exit — with no MCP required.

## What Phase E closes

- `--interactive` stdin loop and lifecycle actions — [http-interactive-mcp-contract.md](../phase-C/http-interactive-mcp-contract.md)
- Persistent **`wyvern-host`** `HostSession`; blocking dialogs via HTTP (same as Phase C)
- MCP server wrapper → `HostSession::run_dialog` for each tool call
- MCP stdio integration harness (headless default)

**Hard dependency:** Phase C **c.16** complete. Phase D required only for wizard MCP tools.

## What Phase E does not close

- Wizard MCP tools when Phase D incomplete — deferred to post-d.2 (see e.3 non-closure)

## Sprint index (sequential: e.1–e.4)

| Sprint | Doc | Branch (pattern) |
|--------|-----|------------------|
| e.1 | [e1-interactive-loop.md](e1-interactive-loop.md) | `feature/phase-E-e1-interactive-loop` |
| e.2 | [e2-blocking-question.md](e2-blocking-question.md) | `feature/phase-E-e2-blocking-question` |
| e.3 | [e3-mcp-server.md](e3-mcp-server.md) | `feature/phase-E-e3-mcp-server` |
| e.4 | [e4-mcp-persistent.md](e4-mcp-persistent.md) | `feature/phase-E-e4-mcp-persistent` |
