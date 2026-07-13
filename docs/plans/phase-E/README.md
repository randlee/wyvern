# Phase E — Interactive & MCP (`integrate/phase-E`)

Phase E implementation PRs target **`integrate/phase-E`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Sprints are **sequentially numbered** `e.1` → `e.4` (strict dependency order — not parallel sub-sprints).

> **Note:** Sprint docs in this directory are authority shells pending full hardening. Acceptance criteria in each sprint stub are authoritative until hardened sprint docs land.

## Phase goal

Wyvern runs as a persistent process, driveable by agents over stdin or as an MCP server.

## Phase acceptance criteria (smoke)

A Claude Code agent can open Wyvern in `--interactive` mode from a background shell, issue multiple blocking dialog commands against one persistent process, receive the JSON results, and exit — with no MCP required.

## What Phase E closes

- `--interactive` stdin loop and lifecycle actions (`show`/`hide`/`exit`)
- Blocking dialogs and clean process termination in interactive mode
- MCP server wrapper with tool mapping for all dialog types
- MCP persistent window lifecycle and integration testing

## Sprint index (sequential: e.1–e.4)

| Sprint | Doc | Branch (pattern) |
|--------|-----|------------------|
| e.1 | [e1-interactive-loop.md](e1-interactive-loop.md) | `feature/phase-E-e1-interactive-loop` |
| e.2 | [e2-blocking-question.md](e2-blocking-question.md) | `feature/phase-E-e2-blocking-question` |
| e.3 | [e3-mcp-server.md](e3-mcp-server.md) | `feature/phase-E-e3-mcp-server` |
| e.4 | [e4-mcp-persistent.md](e4-mcp-persistent.md) | `feature/phase-E-e4-mcp-persistent` |
