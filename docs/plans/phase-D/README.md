# Phase D — Wizard (`integrate/phase-D`)

Phase D implementation PRs target **`integrate/phase-D`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Sprints are **sequentially numbered** `d.1` → `d.6` (strict dependency order — not parallel sub-sprints).

Each individual sprint doc (`d1`–`d6`) is the **sole authority** for that sprint's deliverables, acceptance criteria, and required validation.

## Phase goal

Multi-page wizards with branching navigation and data persistence across pages.

## Phase acceptance criteria (smoke)

The example DAG layout-picker wizard completes a full flow with branching, back-navigation, data restoration, and returns the correct stack JSON.

## What Phase D closes

- Wizard on **`wyvern-host`** HTTP (not wry IPC) — [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- Browser-history navigation model (ADR-0005) in `wyvern-wizard`
- Stack injection and data restoration across pages
- Example DAG layout-picker wizard
- Wizard polish and edge cases

**Hard dependency:** Phase C **c.16** complete (`wyvern-host`, packaged `ui/`, `wyvern-viewer` optional).

## What Phase D does not close

- `--interactive` / lifecycle actions — **Phase E** (`integrate/phase-E`)
- MCP server — **Phase E**

## Sprint index (sequential: d.1–d.6)

| Sprint | Doc | Branch (pattern) |
|--------|-----|------------------|
| d.1 | [d1-wizard-host.md](d1-wizard-host.md) | `feature/phase-D-d1-wizard-host` |
| d.2 | [d2-wizard-ipc.md](d2-wizard-ipc.md) — Wizard HTTP navigation | `feature/phase-D-d2-wizard-ipc` |
| d.3 | [d3-history-nav.md](d3-history-nav.md) | `feature/phase-D-d3-history-nav` |
| d.4 | [d4-stack-inject.md](d4-stack-inject.md) | `feature/phase-D-d4-stack-inject` |
| d.5 | [d5-dag-example.md](d5-dag-example.md) | `feature/phase-D-d5-dag-example` |
| d.6 | [d6-wizard-polish.md](d6-wizard-polish.md) | `feature/phase-D-d6-wizard-polish` |
