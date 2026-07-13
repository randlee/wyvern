# Phase D — Wizard (`integrate/phase-D`)

Phase D implementation PRs target **`integrate/phase-D`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Sprints are **sequentially numbered** `d.1` → `d.6` (strict dependency order — not parallel sub-sprints).

> **Note:** Sprint docs in this directory are authority shells pending full hardening. Acceptance criteria in each sprint stub are authoritative until hardened sprint docs land.

## Phase goal

Multi-page wizards with branching navigation and data persistence across pages.

## Phase acceptance criteria (smoke)

The example DAG layout-picker wizard completes a full flow with branching, back-navigation, data restoration, and returns the correct stack JSON.

## What Phase D closes

- Wizard host: HTML load, config injection, IPC contract
- Browser-history navigation model (ADR-0005)
- Stack injection and data restoration across pages
- Example DAG layout-picker wizard
- Wizard polish and edge cases

## What Phase D does not close

- `--interactive` / lifecycle actions — **Phase E** (`integrate/phase-E`)
- MCP server — **Phase E**

## Sprint index (sequential: d.1–d.6)

| Sprint | Doc | Branch (pattern) |
|--------|-----|------------------|
| d.1 | [d1-wizard-host.md](d1-wizard-host.md) | `feature/phase-D-d1-wizard-host` |
| d.2 | [d2-wizard-ipc.md](d2-wizard-ipc.md) | `feature/phase-D-d2-wizard-ipc` |
| d.3 | [d3-history-nav.md](d3-history-nav.md) | `feature/phase-D-d3-history-nav` |
| d.4 | [d4-stack-inject.md](d4-stack-inject.md) | `feature/phase-D-d4-stack-inject` |
| d.5 | [d5-dag-example.md](d5-dag-example.md) | `feature/phase-D-d5-dag-example` |
| d.6 | [d6-wizard-polish.md](d6-wizard-polish.md) | `feature/phase-D-d6-wizard-polish` |
