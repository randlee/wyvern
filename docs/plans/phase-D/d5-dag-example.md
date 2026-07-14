---
id: d.5
title: Example DAG layout-picker wizard
status: planning
branch: feature/phase-D-d5-dag-example
target: integrate/phase-D
---

# Sprint d.5 — Example DAG layout-picker wizard

## Goal

Ship the phase smoke example: layout-picker DAG with branching agent pages.

## Hard dependencies

- **d.4** merged

## Deliverables

Exact paths (authoritative):

| Path | Purpose |
|------|---------|
| `examples/wizards/layout-picker/wizard.json` | CLI command fixture (`type: wizard`, `config.layouts`, `page.html`) |
| `examples/wizards/layout-picker/pages/layout-picker.html` | Step 1 — layout card selection |
| `examples/wizards/layout-picker/pages/agent.html` | Agent name/description form (reused per agent count) |
| `examples/wizards/layout-picker/pages/finish.html` | Optional summary page before finish |
| `examples/wizards/layout-picker/app.js` | DAG branching via `POST /api/wizard/navigate` `next` descriptors |

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. Step 1: layout cards rendered from `config.layouts` array in `layout-picker.html`
3. Each layout card shows label + agent count
4. Selecting a layout navigates to the first of N agent pages (`POST /api/wizard/navigate`)
5. Each agent page collects a name and description
6. `POST /api/wizard/finish` returns full stack with layout selection + all agent configs
7. Phase D smoke: full flow with back-navigation and data restoration

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_layout_picker
# L2: examples/wizards/layout-picker end-to-end --viewer none
wyvern "$(cat examples/wizards/layout-picker/wizard.json)" --viewer none --ui-root examples/wizards/layout-picker
```

## Non-closure

- Wizard polish and edge cases (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- [project-plan.md](../../project-plan.md) — Phase D acceptance criteria
