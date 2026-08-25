---
id: g.12
title: wyvern-dag-wizard-js agent + DAG type recipe
status: complete (integrate)
branch: feature/phase-G-g12-dag-js-agent
worktree: ../wyvern-worktrees/feature/phase-G-g12-dag-js-agent
target: integrate/phase-G
---

# Sprint g.12 — DAG wizard JS agent

## Goal

Ship the **page-author agent** for the `workspace-canvas` stack and the Layer 3 DAG recipe. Authors who need a canvas / graph / agent-DAG wizard load `references/wizard-types/dag-wizards.md` and invoke `wyvern-dag-wizard-js`. Golden reference is `share/wyvern/examples/agent-dag/` (vendored turbo-flow `dist/`). This sprint ships **agent + type-ref + registry entry only**.

## Hard dependencies

- Wave 2 merged on `integrate/phase-G` (g.7 Agent DAG demo + `data.dag` export)
- g.8 stack registry (`workspace-canvas`, lint profile `nav-limited + export-contract`) — merge-reconcile if this PR lands first
- Phase D host serves static wizard assets (`/wizard/…`, `/shared/…`); authors do not add host routes

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g12-wyvern-dag-wizard-js-agent.md` | This sprint doc (sole AC / validation authority) |
| `.claude/agents/wyvern-dag-wizard-js.md` | Execution-layer agent for canvas / DAG wizards |
| `.claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md` | Layer 3 type recipe (workspace layout, `data.dag`, toolbar vs wizard-nav, prebuilt dist) |
| `.claude/agents/registry.yaml` | Register `wyvern-dag-wizard-js` 0.1.0 |

`SKILL.md` Layer 0, `workspace-canvas.md` stack doc, vanilla-chrome agent, and other type refs are **other sprints** (see Non-closure).

### Contracts

Entry `wizard.json` uses `page.layout: "workspace"`. Page JS reads `config.layouts` only. Finish writes `data.dag` flat under `data` (not wrapped again).

```json
{
  "type": "wizard",
  "page": {
    "id": "canvas",
    "title": "Agent DAG",
    "html": "pages/canvas.html",
    "layout": "workspace"
  },
  "config": {
    "layouts": [
      { "id": "solo", "agents": 1 },
      { "id": "pair", "agents": 2 },
      { "id": "trio", "agents": 3 }
    ]
  },
  "workflow": {
    "post": "{wyvern_share}/scripts/ext/export-agent-dag.py"
  }
}
```

Finish `data.dag` wire-shape (g.7 AC 2). `layout_id` is derived from `config.layouts` by node count (`1` → `solo`, `2` → `pair`, `3` → `trio`, else `custom`):

```json
{
  "layout_id": "pair",
  "nodes": [
    { "id": "node-1", "name": "planner", "role": "plan" },
    { "id": "node-2", "name": "reviewer", "role": "review" }
  ],
  "edges": [
    ["node-1", "node-2"]
  ]
}
```

Canvas page loads the **prebuilt** turbo-flow bundle. Authors do not run an in-repo Svelte build.

```html
<link rel="stylesheet" href="/wizard/dist/canvas.css" />
<script type="module" src="/wizard/dist/canvas.js"></script>
<div id="canvas-app" data-testid="turbo-flow-canvas"></div>
```

Configure / extras / review pages stay `dialog--frame` with a **custom toolbar** (`data-testid` Back / Next / Finish). They do not opt in to `[data-wizard-nav]` / `wizard-nav.js`. Lint profile is `nav-limited + export-contract`.

Page JS navigation (g.7 pair → configure → back → solo recipe):

```js
await wyvernWizardNext({
  data: { nodes: [/* node-1, node-2 */], edges: [/* node-1 → node-2 */], details: {}, editing_node_id: "node-1" },
  next: { id: "node-detail", title: "Configure node", html: "pages/detail.html" }
});
await wyvernWizardBack();
await wyvernWizardFinish({
  button: "finish",
  data: {
    dag: {
      layout_id: "solo",
      nodes: [{ id: "node-1", "name": "scout", "role": "explore" }],
      edges: [["node-1", "finish"]]
    }
  },
  stack: [ /* full visited stack */ ]
});
```

Agent output is **fenced JSON** (guidelines v0.7 basic envelope):

```json
{
  "success": true,
  "data": {
    "status": "implemented",
    "stack": "workspace-canvas",
    "wizard_type": "dag",
    "summary": "…",
    "files_changed": [],
    "export_contract": {
      "finish_data_key": "dag",
      "required_keys": ["layout_id", "nodes", "edges"]
    },
    "nav": "custom-toolbar",
    "dist": "prebuilt",
    "verification": [],
    "follow_up": []
  },
  "error": null
}
```

Page JS has no disk I/O. Persist via `workflow.post` (g.7 `export-agent-dag.py` reads finish stdin). `--workflow-dry-run` writes nothing.

## Acceptance criteria

1. `.claude/agents/wyvern-dag-wizard-js.md` has YAML frontmatter `name: wyvern-dag-wizard-js`, `version: 0.1.0`, and a fenced-JSON output envelope with `success` / `data` / `error`.
2. The agent names `page.layout: "workspace"`, the g.7 `data.dag` wire-shape (`layout_id`, `nodes[]` of `{ id, name, role }`, `edges[]` of `[from, to]`), custom toolbar vs `wizard-nav.js`, and the prebuilt `dist/canvas.js` + `dist/canvas.css` pattern (no in-repo Svelte build).
3. `dag-wizards.md` is the Layer 3 recipe: stack `workspace-canvas`, golden path `share/wyvern/examples/agent-dag/`, pair → configure → back → solo, finish `data.dag` flat under `data`, lint profile `nav-limited + export-contract`.
4. `.claude/agents/registry.yaml` registers `wyvern-dag-wizard-js` at version `0.1.0` with path `.claude/agents/wyvern-dag-wizard-js.md`.
5. Agent and type-ref tell authors to copy/vendor `dist/` from a built turbo-flow (or the shipped example); they do not add a Svelte/npm build step to the Wyvern repo.

## Required validation

```bash
test -f docs/plans/phase-G/g12-wyvern-dag-wizard-js-agent.md
test -f .claude/agents/wyvern-dag-wizard-js.md
test -f .claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md

rg -n "^name: wyvern-dag-wizard-js" .claude/agents/wyvern-dag-wizard-js.md
rg -n "^version: 0.1.0" .claude/agents/wyvern-dag-wizard-js.md
rg -n 'wyvern-dag-wizard-js:' .claude/agents/registry.yaml
rg -n 'version: 0.1.0' .claude/agents/registry.yaml
rg -n 'path: .claude/agents/wyvern-dag-wizard-js.md' .claude/agents/registry.yaml

rg -n 'layout: "workspace"|page.layout' .claude/agents/wyvern-dag-wizard-js.md
rg -n 'data.dag' .claude/agents/wyvern-dag-wizard-js.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md
rg -n 'wizard-nav' .claude/agents/wyvern-dag-wizard-js.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md
rg -n 'dist/canvas.js|prebuilt' .claude/agents/wyvern-dag-wizard-js.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md
rg -n 'nav-limited' .claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md
rg -n 'share/wyvern/examples/agent-dag' \
  .claude/agents/wyvern-dag-wizard-js.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md

# Fenced JSON envelope present
rg -n '"success"' .claude/agents/wyvern-dag-wizard-js.md
rg -n '"error"' .claude/agents/wyvern-dag-wizard-js.md
```

## Non-closure

- `SKILL.md` Layer 0 router and core refs — **g.10**
- `.claude/agents/wyvern-wizard-js.md` + `templates/vanilla-chrome/` — **g.11**
- `references/stacks/workspace-canvas.md` stack doc — **g.8**
- Type refs for template / hook / welcome-bridge and sc-compose J2 — **g.13**
- WIZARD-LINT-005–008 implementation — **g.9**
- CI lint gate — **g.14**
- Replacing vendored turbo-flow with live Svelte source in-repo
- DAG execution / agent spawn (still deferred; [agent-dag-execution-deferral.md](agent-dag-execution-deferral.md))
- New `wizard.json` schema fields
- Declaring `config.dataflow` on the shipped agent-dag example (authors add it when g.9 lands)

## Authority

- REQ-0125, REQ-0126
- ADR-0006, ADR-0023, ADR-0024
- [g7-dag-agent-execution.md](g7-dag-agent-execution.md)
- [g8-wizard-authoring-foundation.md](g8-wizard-authoring-foundation.md) (stack registry; may land in parallel)
- [agent-dag-execution-deferral.md](agent-dag-execution-deferral.md)
- [wizard-workflow-architecture.md](wizard-workflow-architecture.md)
- synaptic-canvas `docs/claude-code-skills-agents-guidelines.md` v0.7 (fenced JSON, registry)
