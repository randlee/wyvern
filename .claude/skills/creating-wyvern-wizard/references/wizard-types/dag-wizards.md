# DAG wizards (`workspace-canvas`)

Layer 3 type recipe. Load this **and** `references/stacks/workspace-canvas.md`
(when present). Do not load template / hook / welcome type docs.

**Agent:** `wyvern-dag-wizard-js`  
**Stack:** `workspace-canvas`  
**Lint profile:** `nav-limited + export-contract`  
**Golden example:** `share/wyvern/examples/agent-dag/`

Use this type when the author needs a **node/edge canvas** (agent graph,
pipeline sketch, turbo-flow). Use `vanilla-chrome` + `wyvern-wizard-js` for
forms, pickers, hooks, and welcome cards.

## When to use

| Intent | This type? |
|--------|------------|
| Draw / edit a DAG, then export JSON | Yes |
| Pick a template and fill fields | No — template-wizards |
| Toggle hook / settings files | No — hook-and-settings-wizards |
| Hub card that hops to another wizard | No — welcome-bridge-wizards |

## Layout

Entry page is a **workspace**, not a dialog frame:

```json
{
  "page": {
    "id": "canvas",
    "title": "Agent DAG",
    "html": "pages/canvas.html",
    "layout": "workspace"
  }
}
```

Host sizes the window from `config.estimated_size` when present (agent-dag
uses `1200×800`). Configure / extras / review pages stay
`class="dialog dialog--frame"` and do **not** set `layout: "workspace"`.

## Prebuilt `dist/` (no in-repo Svelte build)

The canvas is a **vendored** turbo-flow bundle. Authors copy `dist/canvas.js`
and `dist/canvas.css` from a built turbo-flow (or from the shipped example).
The Wyvern repo does not require `npm install` or a Svelte compiler for
authors.

```html
<link rel="stylesheet" href="/wizard/dist/canvas.css" />
<script src="/shared/wyvern-api.js"></script>
<script src="/wizard/stack-merge.js"></script>
<script src="/wizard/app.js"></script>
<div id="canvas-app" data-testid="turbo-flow-canvas"></div>
<script type="module" src="/wizard/dist/canvas.js"></script>
```

Do not add wizard-root `package.json` / `src/*.svelte` unless a later stack
(`vite-spa`) is registered.

## Custom toolbar vs wizard-nav

| Surface | Navigation |
|---------|------------|
| Canvas | Toolbar **inside** the vendored canvas (add node, layout chips, Review) |
| node-detail / node-extras / review | Explicit buttons with `data-testid` (`*-back`, `*-next`, `review-finish`) |

Do **not** include `/shared/wizard-nav.js` or `[data-wizard-nav]`. Nav lint
is **limited**; the export contract is the gate (`data.dag` on finish).

## `data.dag` export contract

Finish payload is `{ "dag": { … } }` **flat under `data`**. Host treats
`data` as opaque (ADR-0006). Post script `export-agent-dag.py` validates:

| Field | Type | Rule |
|-------|------|------|
| `layout_id` | string | Non-empty. From `config.layouts` by node count (`1`/`2`/`3` → that `id`, else `custom`) |
| `nodes` | array | Each `{ id, name, role }` — all non-empty strings |
| `edges` | array | Each `[from, to]` two non-empty strings |

Solo with no authored edges: `edges: [[nodeId, "finish"]]`.

```json
{
  "button": "finish",
  "data": {
    "dag": {
      "layout_id": "pair",
      "nodes": [
        { "id": "node-1", "name": "planner", "role": "plan" },
        { "id": "node-2", "name": "reviewer", "role": "review" }
      ],
      "edges": [["node-1", "node-2"]]
    }
  },
  "stack": []
}
```

`workflow.post` reads that finish JSON on stdin and writes
`$WYVERN_REPO_ROOT/wyvern-dag-export.json` (else `./wyvern-dag-export.json`).
`--dry-run` writes nothing. Page JS never writes the file.

## Recipe: pair → configure → back → solo

Normative hop sequence (g.7 `wizard_agent_dag_nav`):

1. Canvas **pair** graph (two nodes, one edge).
2. Open configure for `node-1` via `wyvernWizardNext` + `editing_node_id`.
3. **Back** — canvas restores graph; revisiting configure restores fields.
4. Switch canvas to **solo** (one node) — forward history truncated.
5. Review / finish — `data.dag.layout_id` is `solo` with one node.

`app.js` keeps graph state in `stack-merge.js` (`cacheGraph` /
`mergeStack`). `assembleDag()` maps Svelte nodes/edges + detail forms onto
the wire-shape above.

## Page graph

| Page id | HTML | Role |
|---------|------|------|
| `canvas` | `pages/canvas.html` | Workspace entry (only page named in `wizard.json`) |
| `node-detail` | `pages/detail.html` | Core fields (`name`, `role`, `description`) |
| `node-extras` | `pages/extras.html` | Optional prompt / tool |
| `review` | `pages/review.html` | Terminal (`data-wizard-terminal="true"`); Finish |

Further pages exist only as `wizardNextDescriptor` objects in `app.js`.

## Dataflow declaration (optional until g.9)

When `config.dataflow` is added (g.8 spec / g.9 lint):

```json
{
  "config": {
    "dataflow": {
      "version": 1,
      "pages": {
        "canvas": { "exports": { "dag": "object" } },
        "review": {
          "requires": ["dag"],
          "terminal": true,
          "post_input": { "dag": "object" }
        }
      }
    }
  }
}
```

## Tests and smoke (author checklist)

- `data-testid` on canvas root, detail fields, review finish, `wizard-error`.
- Manual: `wyvern share/wyvern/examples/agent-dag/wizard.json --ui-root share/wyvern/examples/agent-dag --viewer none`
- Shipped host tests (maintainers): `wizard_agent_dag`, `wizard_agent_dag_nav`.
- Mirror `share/wyvern/…` into `crates/wyvern/embedded/…` when the example
  is the bundled asset.

## Out of scope for this type

- Live DAG execution or "Run DAG"
- Validating acyclicity in the host
- In-repo Svelte rebuild of turbo-flow
- `wizard-nav.js` chrome on the canvas
