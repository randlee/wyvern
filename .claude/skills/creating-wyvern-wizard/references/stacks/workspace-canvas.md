# Stack: `workspace-canvas`

Canvas / graph wizards. **Not the default.** Choose this stack when the primary surface is a node-edge workspace (Agent DAG, turbo-flow), not a dialog form.

**Registry:** `status: supported` · `lint_profile: nav-limited + export-contract` · agent `wyvern-dag-wizard-js`  
**Golden:** `share/wyvern/examples/agent-dag/` (vendored turbo-flow `dist/`)

## When to use

- Users edit nodes and edges on a canvas, then finish
- `page.layout` is `"workspace"` (viewport uses workspace sizing in `wyvern-api.js`)
- A prebuilt SPA `dist/` is served as static files next to the wizard pages

Use [vanilla-chrome.md](vanilla-chrome.md) for pickers, hooks, and welcome bridges.

## Layout and packaging

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

| Path | Role |
|------|------|
| `pages/canvas.html` | Hosts the canvas mount + custom toolbar |
| `dist/canvas.js` / `dist/canvas.css` | **Prebuilt** bundle (turbo-flow). Authors do **not** run an in-repo Svelte/Vite build |
| `app.js` | Workspace glue: graph state → next/back → finish `data.dag` |
| `pages/detail.html` / `pages/review.html` | Optional configure / review steps |

Build the SPA **elsewhere**, copy artifacts into `dist/`, commit the dist. Wave 3 does not replace turbo-flow with live source in this repo.

## Navigation: custom toolbar vs wizard-nav

Workspace pages often ship their own toolbar (add node, layout solo/pair/trio, Finish). Full `wizard-nav.js` chrome is **optional**.

| Control | vanilla-chrome | workspace-canvas |
|---------|----------------|------------------|
| Back | `data-wizard-back` | Custom control that calls `wyvernWizardBack()` |
| Next / configure | `data-wizard-next` | Canvas action → `wyvernWizardNext(data, nextDescriptor)` |
| Finish | chrome flips Next → Finish | Toolbar Finish → `wyvernWizardFinish` |
| Cancel | required on terminal | still required on the terminal page (WIZARD-LINT-002) |

If a page **does** include `<script src="/shared/wizard-nav.js" data-wizard-chrome>`, nav rules 003–004 apply to that page. Canvas-only pages use `nav-limited`: 001/004 may not apply; 002 still applies on terminal.

`wyvern-api.js` is still required (`window.wyvern`, HTTP next/back/finish, workspace viewport).

## Export contract (`data.dag`)

Finish `data` carries a **flat** `dag` object (g.7 AC 2):

```json
{
  "dag": {
    "layout_id": "solo",
    "nodes": [{ "id": "node-1", "name": "scout", "role": "explore" }],
    "edges": [["node-1", "finish"]]
  }
}
```

| Field | Rule |
|-------|------|
| `layout_id` | From `config.layouts` by node count (`1` → `solo`, `2` → `pair`, `3` → `trio`, else `custom`) |
| `nodes` | `{ id, name, role }` |
| `edges` | `[from, to]` pairs |
| Host | Treats `data` as opaque (ADR-0006) |

`workflow.post` (e.g. `export-agent-dag.py`) writes `$WYVERN_REPO_ROOT/wyvern-dag-export.json` or `./wyvern-dag-export.json`. `--workflow-dry-run` writes nothing. No execute / spawn / “Run DAG” control (execution deferred).

Declare in `config.dataflow`:

```json
{
  "pages": {
    "canvas": { "exports": { "dag": "object" } },
    "review": {
      "requires": ["dag"],
      "terminal": true,
      "post_input": { "dag": "object" }
    }
  }
}
```

See [dataflow-contracts.md](../core/dataflow-contracts.md) §6 for the export-contract extra check (`layout_id` / `nodes` / `edges` literals in local JS).

## Page JS pattern (g.7)

```js
await wyvernWizardNext({
  data: { nodes: [/* … */], edges: [/* … */], details: {}, editing_node_id: "node-1" },
  next: { id: "node-detail", title: "Configure node", html: "pages/detail.html" }
});
await wyvernWizardBack();
await wyvernWizardFinish({
  button: "finish",
  data: { dag: { layout_id: "solo", nodes: [/* … */], edges: [/* … */] } },
  stack: [/* full visited stack */]
});
```

Pair → configure → back → restore fields → switch to solo (forward history truncated) → finish is the golden nav recipe (`wizard_agent_dag_nav`).

Page JS still has **no disk I/O**. Export is post-script only.

## Lint profile (`nav-limited + export-contract`)

| Code | workspace-canvas |
|------|------------------|
| WIZARD-LINT-001 | Only if the page is a non-entry **dialog** step (detail/review with chrome) |
| WIZARD-LINT-002 | Terminal page must offer Cancel |
| WIZARD-LINT-003–004 | Only if `data-wizard-chrome` is present |
| WIZARD-LINT-005–008 | When `config.dataflow` is declared (g.9) |
| export-contract | `dag` export must mention `layout_id`, `nodes`, `edges` in local JS |

## Tests

- Host HTTP: finish wire-shape (`wizard_agent_dag`) and pair → configure → back → solo (`wizard_agent_dag_nav`)
- CLI: post export + `--workflow-dry-run` (`workflow_export_agent_dag`)
- Do not assert a live Svelte rebuild

## Non-goals

- In-repo `npm run build` as an author step
- Validating DAG acyclicity in the host
- Spawning agents from finish JSON
- Making this the default stack
