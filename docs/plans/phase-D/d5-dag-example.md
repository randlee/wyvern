---
id: d.5
title: Example DAG layout-picker wizard
status: planning
branch: feature/phase-D-d5-dag-example
target: integrate/phase-D
---

# Sprint d.5 — Example DAG layout-picker wizard

## Goal

HTML **examples** that exercise `WizardSession` branching via page JS — not new Rust stack code.

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
| `examples/wizards/workspace-hint/` | **new** — example HTML page with `layout: workspace` + sample `estimated_size` in config |
| `crates/wyvern-host/tests/wizard_layout_picker.rs` | HTTP integration against layout-picker fixture |
| `crates/wyvern-host/tests/wizard_workspace_hint.rs` | HTTP integration against workspace-hint fixture |

### Workspace example (HTML only)

Example wizard page that needs a larger viewport (e.g. a graph canvas in HTML). Not a Rust integration.

- Served like any wizard page under `/wizard/**`
- `page.layout: "workspace"` — Wyvern passes the string through; sizing in d.6
- `config.estimated_size` — **opaque** example shape; page JS reads what it needs

```json
{
  "type": "wizard",
  "page": { "id": "editor", "title": "Canvas", "html": "pages/editor.html", "layout": "workspace" },
  "config": {
    "estimated_size": { "width": 960, "height": 640 }
  }
}
```

- `pages/editor.html` is placeholder/minimal canvas — authors replace with any HTML (Flowise embed, custom DAG, etc.)
- **d.5** proves wire shape in fixtures; **d.6** implements generic workspace sizing in `wyvern-api.js`

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
9. `workspace-hint` example: `page.layout: "workspace"` + opaque `estimated_size` in config renders without manual resize

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
