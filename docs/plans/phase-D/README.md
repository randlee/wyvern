# Phase D — Wizard (`integrate/phase-D`)

Phase D implementation PRs target **`integrate/phase-D`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Sprints are **sequentially numbered** `d.1` → `d.6` (strict dependency order — not parallel sub-sprints).

Each individual sprint doc (`d1`–`d6`) is the **sole authority** for that sprint's deliverables, acceptance criteria, and required validation.

## Code baseline (hard prerequisite)

Phase D sprints assume the **post-c.16** codebase on `main` / `integrate/phase-C`:

- `wyvern-host` exists; `wyvern-window` is deleted
- Packaged `ui/` + optional `wyvern-viewer`
- Blocking dialog types (`message`, `input`, `markdown`, `question`) pass CI with `--viewer none`

**`integrate/phase-D` must be created or rebased from that baseline** before d.1 lands. `develop` may lag until Phase C merges back; do not implement wizard routes against `wyvern-window`.

## Phase goal

Multi-page wizards with branching navigation and data persistence across pages — including optional **workspace-layout** wizard pages (e.g. HTML/JS graph canvases) within the same `type: wizard` session.

> **Examples only:** Names like DAG, graph, or Flowise describe **sample HTML/JS page content**, not Rust integrations or required dependencies. Wyvern passes opaque `config` and `page.layout`; all domain UI lives in page HTML (ADR-0006).

## Phase acceptance criteria (smoke)

The example DAG layout-picker wizard completes a full flow with branching, back-navigation, data restoration, and returns the correct stack JSON. At least one wizard page demonstrates **workspace layout** (graph/DAG surface) within the same `type: wizard` session.

## Architecture principle — traits hide implementation

| Crate | Owns | Must not leak |
|-------|------|----------------|
| `wyvern-wizard` | Pure navigation logic behind **`WizardEngine`** trait | History array layout, cursor internals, concrete `BrowserHistory` type |
| `wyvern-host` | HTTP routes, session lifecycle, `Box<dyn WizardEngine>` holder | History cursor math, stack truncation rules, page-domain interpretation |
| `wyvern-schema` | `WizardCommand` / `WizardResult` validation | Navigation behaviour |
| Page JS | Domain branching, opaque `data` blobs, **any embedded HTML UI** (graphs, forms, etc.) | Host/wizard internals |

**Workspace pages:** Optional `page.layout: "workspace"` for HTML that needs viewport-sized canvas (e.g. a graph editor page). Still wizard HTTP routes — not a separate dialog type. Rust does not implement graph/DAG/Flowise logic.

Host calls **only** the public `WizardEngine` API from `wyvern-wizard`. Integration tests may use `WizardEngine::new_for_test(...)`; production code must not import wizard private modules.

See [docs/wyvern-wizard/architecture.md](../../wyvern-wizard/architecture.md) ADR-0007.

## What Phase D closes

- Wizard on **`wyvern-host`** HTTP (not wry IPC) — [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- Browser-history navigation model (ADR-0005) in `wyvern-wizard` behind `WizardEngine`
- Stack injection and data restoration across pages (REQ-0024)
- Example DAG layout-picker wizard (form branching — HTML/JS example)
- Optional workspace-layout wizard page in examples (e.g. graph canvas — HTML only)
- Wizard polish, edge cases, and **viewport sizing policy** — [viewport-sizing.md](viewport-sizing.md)

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
| d.6 | [d6-wizard-polish.md](d6-wizard-polish.md) — polish, viewport sizing, workspace layout | `feature/phase-D-d6-wizard-polish` |

## Viewport sizing (cross-cutting)

High-churn agent dialogs must **fit on screen without manual resize iteration**. Policy: [viewport-sizing.md](viewport-sizing.md).

- **Dialog steps** — intrinsic measure + ~25% slack, viewport clamp, scroll overflow (d.6).
- **Workspace-layout wizard pages** — generic `page.layout` + optional `config.estimated_size` passthrough (examples may show graph HTML; no Rust graph code) (d.5–d.6).

## Boundary files (tightened in plan hardening)

- `boundaries/wyvern-wizard/wizard.toml` — pure logic, public trait surface
- `boundaries/wyvern-host/host.toml` — HTTP + session; wizard routes delegate to `WizardEngine` only
