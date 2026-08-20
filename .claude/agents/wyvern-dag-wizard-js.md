---
name: wyvern-dag-wizard-js
version: 0.1.0
description: >-
  Author workspace-canvas DAG wizards (page.layout workspace, vendored
  turbo-flow dist, custom toolbar, data.dag finish export). Use when the
  task is a canvas / graph / agent-DAG wizard. Golden example:
  share/wyvern/examples/agent-dag/.
---

# wyvern-dag-wizard-js

## Purpose

Implement or revise a Wyvern **workspace-canvas** wizard: canvas page + node
configure/review hops + finish `data.dag`. This is the execution-layer agent
for stack `workspace-canvas`. Dialog-frame / form / hook / welcome wizards
belong to `wyvern-wizard-js`.

Load first (progressive disclosure — do not load the whole skill tree):

- `.claude/skills/creating-wyvern-wizard/references/wizard-types/dag-wizards.md`
- `.claude/skills/creating-wyvern-wizard/references/stacks/workspace-canvas.md` (when present; g.8)

Golden example: `share/wyvern/examples/agent-dag/` (`wizard.json`, `app.js`,
`pages/*.html`, `dist/canvas.js`, `dist/canvas.css`, `stack-merge.js`).

## Inputs

Fenced JSON or equivalent fields:

```json
{
  "task": "create | revise | review",
  "wizard_root": "share/wyvern/examples/<name>",
  "title": "optional canvas title",
  "layouts": [
    { "id": "solo", "agents": 1 },
    { "id": "pair", "agents": 2 }
  ],
  "post_script": "{wyvern_share}/scripts/ext/export-agent-dag.py",
  "notes": "optional"
}
```

- `task` required: `create` scaffolds from the golden example; `revise` edits
  an existing canvas wizard; `review` checks contracts only (no writes).
- `wizard_root` required; must stay under the repo (no `..` escape).
- `layouts` optional; default is the g.7 trio (`solo` / `pair` / `trio`).

## Execution Steps

1. **Confirm stack.** Entry page MUST set `page.layout: "workspace"`. If the
   wizard is dialog-frame only, stop and return `VALIDATION.WRONG_STACK`.
2. **Read the golden example.** Copy contracts from
   `share/wyvern/examples/agent-dag/` — do not invent a second canvas API.
3. **Vendor `dist/`, do not build Svelte in this repo.** Serve
   `/wizard/dist/canvas.js` and `/wizard/dist/canvas.css` as static files.
   Authors copy a prebuilt turbo-flow bundle (or the shipped example `dist/`).
   Do not add `package.json`, `npm run build`, or Svelte source under the
   wizard root.
4. **Split chrome.**
   - Canvas page: `#canvas-app` + module script; **custom toolbar** lives
     inside the vendored canvas (not `[data-wizard-nav]`).
   - Configure / extras / review: `dialog--frame` with explicit
     `data-testid` Back / Next / Finish buttons. Do **not** include
     `/shared/wizard-nav.js`.
5. **Wire hops in `app.js`.** Entry is only in `wizard.json`. Further pages
   use `wyvernWizardNext({ data, next })` / `wyvernWizardBack()` /
   `wyvernWizardFinish`. Restore configure fields from stack / cached graph
   after back. Switching node count (pair → solo) truncates forward history
   and rebuilds `data.dag`.
6. **Assemble finish `data.dag`.** Flat under `data` (not `data.data.dag`).
   Shape:
   - `layout_id` string — from `config.layouts` by node count (`1` → that
     layout's `id`, else `custom`)
   - `nodes` array of `{ id, name, role }` (all non-empty strings)
   - `edges` array of `[from, to]` string pairs
   Solo with no authored edges uses `[node-id, "finish"]`.
7. **No disk I/O in page JS.** Persist through `workflow.post` (finish JSON
   on stdin). `--workflow-dry-run` / `--dry-run` writes nothing. Do not call
   `fs`, `fetch` to `file://`, or spawn agents from the canvas.
8. **`data-testid` on every control** the g.7 tests already use
   (`turbo-flow-canvas`, `node-detail-*`, `review-finish`, `wizard-error`).
9. **Return fenced JSON only** (envelope below).

## Platform boundaries

| Need | Where | Mechanism |
|------|--------|-----------|
| Read session | Page JS | `window.wyvern` via `/shared/wyvern-api.js` (`config`, `page`, `page_data`, `stack`) |
| Navigate | Page JS | `wyvernWizardNext` / `wyvernWizardBack` / `wyvernWizardFinish` |
| Size workspace | Page JS | `WyvernApi.applyWizardLayout` when present |
| Persist DAG | `workflow.post` | Finish stdin → `export-agent-dag.py` → `wyvern-dag-export.json` |
| Pre-fill | `workflow.pre` | stdout `{ "config_patch": { … } }` merged into `config` |

Page JS must not spawn Cursor / Claude / ATM agents or run the DAG.
Execution stays deferred.

## Output Format

Always return **one** fenced JSON block (guidelines v0.7 basic envelope).
Skills format this for humans.

```json
{
  "success": true,
  "data": {
    "status": "implemented",
    "stack": "workspace-canvas",
    "wizard_type": "dag",
    "summary": "What changed, in one or two sentences.",
    "files_changed": [
      "share/wyvern/examples/<name>/wizard.json"
    ],
    "export_contract": {
      "finish_data_key": "dag",
      "required_keys": ["layout_id", "nodes", "edges"]
    },
    "nav": "custom-toolbar",
    "dist": "prebuilt",
    "assumptions": [],
    "verification": [
      "wyvern share/wyvern/examples/<name>/wizard.json --ui-root share/wyvern/examples/<name> --viewer none"
    ],
    "follow_up": []
  },
  "error": null
}
```

`status` is `implemented` | `reviewed` | `blocked`.
`nav` is `custom-toolbar` (required for this stack).
`dist` is `prebuilt`.

On failure:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION.WRONG_STACK",
    "message": "Entry page is dialog-frame; use wyvern-wizard-js.",
    "recoverable": false,
    "suggested_action": "Re-route to wyvern-wizard-js or set page.layout workspace."
  }
}
```

Error codes: `VALIDATION.INPUT`, `VALIDATION.WRONG_STACK`,
`VALIDATION.EXPORT_CONTRACT`, `EXECUTION.BLOCKED`.

## Constraints

- One stack only: `workspace-canvas`.
- Do not add an in-repo Svelte / Vite / npm build for authors.
- Do not opt canvas or node pages into `wizard-nav.js`.
- Do not wrap finish as `{ data: { data: { dag } } }` — `data.dag` is flat.
- Do not invent a Rust DAG engine, execute hook, or "Run DAG" control.
- Do not edit `crates/wyvern-host`, `wyvern-schema`, or CLI subcommands.
- Do not load `references/wizard-types/template-wizards.md` or vanilla-chrome
  templates unless the user asked to convert stacks.
