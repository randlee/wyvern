---
id: d.5
title: Example DAG layout-picker wizard
status: planning
branch: feature/phase-D-d5-dag-example
target: integrate/phase-D
---

# Sprint d.5 — Example DAG layout-picker wizard

## Goal

Ship the phase smoke example: layout-picker DAG with branching agent pages, plus a **wizard graph page** demonstrating workspace layout (Flowise-style hints).

DAG branching and graph editing are **wizard pages** in one `type: wizard` flow — not a separate Wyvern dialog type.

## Hard dependencies

- **d.4** merged

## Deliverables

Exact paths (authoritative):

| Path | Purpose |
|------|---------|
| `examples/wizards/layout-picker/wizard.json` | CLI command fixture (`type: wizard`) |
| `examples/wizards/layout-picker/pages/layout-picker.html` | Step 1 — layout card selection |
| `examples/wizards/layout-picker/pages/agent.html` | Agent name/description form (reused per agent count) |
| `examples/wizards/layout-picker/pages/finish.html` | Optional summary page before finish |
| `examples/wizards/layout-picker/app.js` | DAG branching via `wyvernWizardNext` + explicit `next` descriptors |
| `examples/wizards/layout-picker/styles.css` | Layout card grid |
| `examples/wizards/workspace-hint/` | **new** — minimal workspace page proving `config.layout` + Flowise-style hints |
| `crates/wyvern-host/tests/wizard_layout_picker.rs` | **new** — HTTP integration against fixture |

### Workspace size hints (Flowise — wizard graph page)

DAG/graph/Flowise surfaces are **wizard pages**: served under `/wizard/**`, sized via [viewport-sizing.md](viewport-sizing.md) `layout: workspace`, navigated with `wyvernWizardNext` / `wyvernWizardFinish`.

Small graphs may not be full-screen. Flowise (or the agent) supplies bounds in wizard `config`; the graph HTML page embeds the editor.

**d.5 proves the wire shape** in `examples/wizards/workspace-hint/wizard.json`:

```json
{
  "type": "wizard",
  "page": { "id": "graph", "title": "Flow", "html": "pages/graph.html", "layout": "workspace" },
  "config": {
    "layout": "workspace",
    "estimated_size": { "width": 960, "height": 640 },
    "flowise": { "estimated_width": 960, "estimated_height": 640 }
  }
}
```

- `pages/graph.html` uses `dialog--workspace` (CSS) — internal pan/scroll for canvas.
- Page reads `window.wyvern.config.estimated_size` (normalized from `flowise.*` in bootstrap).
- **d.5** documents hint passthrough only; **d.6** implements viewer sizing policy ([viewport-sizing.md](viewport-sizing.md)).

Authority: [viewport-sizing.md](viewport-sizing.md) — workspace mode.

### `wizard.json` (authoritative fixture)

```json
{
  "type": "wizard",
  "page": {
    "id": "layout-picker",
    "title": "Choose layout",
    "html": "pages/layout-picker.html"
  },
  "config": {
    "layouts": [
      { "id": "solo", "label": "Solo", "agents": 1 },
      { "id": "pair", "label": "Pair", "agents": 2 },
      { "id": "trio", "label": "Trio", "agents": 3 }
    ]
  },
  "width": 640,
  "height": 480
}
```

### DAG branching (`app.js`)

1. **layout-picker.html** — render cards from `window.wyvern.config.layouts`; on select, `wyvernWizardNext` with:
   - `data: { layout_id, label, agent_count }`
   - `next: { id: "agent-1", title: "Agent 1", html: "pages/agent.html" }`
2. **agent.html** — read `window.wyvern.stack` to determine current agent index (`agent-1` … `agent-N`); on submit:
   - If more agents remain: `next` → next `agent-{k}` page (same `agent.html`, new id/title)
   - If last agent: `next` → `{ id: "finish", title: "Review", html: "pages/finish.html" }`
3. **finish.html** — display summary from `stack`; `wyvernWizardFinish({ button: "finish", data: {}, stack })` where `stack` is built from `window.wyvern.stack` + current page data

Domain logic stays in JS — host/wizard only store opaque `data` blobs (ADR-0006).

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. Step 1: layout cards rendered from `config.layouts` in `layout-picker.html`
3. Each layout card shows label + agent count
4. Selecting a layout navigates to the first of N agent pages (`POST /api/wizard/navigate`)
5. Each agent page collects a name and description (`data: { name, description }`)
6. `POST /api/wizard/finish` returns full stack with layout selection + all agent configs
7. Phase D smoke: full flow with back-navigation and data restoration (select pair → agent-1 → back → change to solo → complete)
8. `cargo test -p wyvern-host wizard_layout_picker` passes without a GUI
9. `workspace-hint` example: `config.layout: "workspace"` + `estimated_size` / `flowise` hints render without manual resize

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_layout_picker
cargo test -p wyvern-host wizard_workspace_hint
# L2: examples/wizards/layout-picker end-to-end --viewer none
wyvern "$(cat examples/wizards/layout-picker/wizard.json)" --viewer none --ui-root examples/wizards/layout-picker
wyvern "$(cat examples/wizards/workspace-hint/wizard.json)" --viewer embedded --ui-root examples/wizards/workspace-hint
npx playwright test tests/l2/wizard-layout-picker.spec.ts
```

## Non-closure

- Viewport slack sizing implementation in `wyvern-api.js` / viewer (d.6)
- Wizard polish and edge cases (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- [project-plan.md](../../project-plan.md) — Phase D acceptance criteria
- [viewport-sizing.md](viewport-sizing.md)
- ADR-0006
