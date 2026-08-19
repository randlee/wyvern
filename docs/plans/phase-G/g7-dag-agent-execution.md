---
id: g.7
title: Agent DAG demo + export (execution deferred)
status: complete
branch: feature/phase-G-g7-dag-agent-execution
target: integrate/phase-G
---

# Sprint g.7 — Agent DAG flow demo

## Goal

Example (c): layout → configure agents → review, then export DAG JSON via the g.4 post runner (REQ-0125). The welcome Agent DAG page finishes with `next_wizard` into this demo (REQ-0126). Page JS branching is specified in this sprint's Contracts. **DAG execution is deferred** ([agent-dag-execution-deferral.md](agent-dag-execution-deferral.md)).

## Hard dependencies

- g.4 merged (`WorkflowRunner`, chain loop)
- Phase D wizard stack/back HTTP routes (behavior specified by AC 1 below)

## Deliverables

| Path | Purpose |
|------|---------|
| `share/wyvern/examples/agent-dag/wizard.json` | Demo + `workflow.post` |
| `share/wyvern/examples/agent-dag/pages/*.html` | Layout, configure, review (HTML graph) |
| `share/wyvern/examples/agent-dag/app.js` | `wyvernWizardNext` / back / finish `data.dag` assembly |
| `scripts/ext/export-agent-dag.py` | Default write `$WYVERN_REPO_ROOT/wyvern-dag-export.json` or `./wyvern-dag-export.json`; `-o` is script/test-only |
| `share/wyvern/welcome/pages/agent-dag.html` | Deferral notice + required `next_wizard` |
| `crates/wyvern/tests/workflow_export_agent_dag.rs` | Post export contract; assert no execute/spawn API |
| `crates/wyvern/tests/workflow_welcome_chain_agent_dag.rs` | Welcome Agent DAG finish JSON → CLI resolves next hop |
| `crates/wyvern-host/tests/wizard_agent_dag.rs` | HTTP finish asserts AC 2 `data.dag` wire-shape (layout_id, nodes, edges) |
| `crates/wyvern-host/tests/wizard_agent_dag_nav.rs` | HTTP drive of AC 1: pair → agent-1 → back → solo → finish |

No **Run DAG** control. Host and schema treat `data` as opaque (ADR-0006).

### Contracts

Page JS reads `config.layouts` only.

```json
{
  "type": "wizard",
  "page": { "id": "layout", "title": "Agent DAG", "html": "pages/layout.html" },
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

Page JS contract (`app.js`):

```js
// layout page — pair selected
await wyvernWizardNext({
  data: { layout_id: "pair" },
  next: { id: "agent-1", title: "Agent 1", html: "pages/agent.html" }
});
// after configure agent-1, back restores page_data on agent-1
await wyvernWizardBack();
// switch to solo — forward history truncated; finish assembles data.dag
await wyvernWizardFinish({
  button: "finish",
  data: {
    dag: {
      layout_id: "solo",
      nodes: [{ id: "agent-1", name: "planner", role: "plan" }],
      edges: [["layout-picker", "agent-1"], ["agent-1", "finish"]]
    }
  },
  stack: [ /* full visited stack */ ]
});
```

Finish `data.dag` (flat under `data`, not wrapped again) for a **pair** path (review-page example):

```json
{
  "layout_id": "pair",
  "nodes": [
    { "id": "agent-1", "name": "planner", "role": "plan" },
    { "id": "agent-2", "name": "reviewer", "role": "review" }
  ],
  "edges": [
    ["layout-picker", "agent-1"],
    ["agent-1", "agent-2"],
    ["agent-2", "finish"]
  ]
}
```

Welcome Agent DAG finish (required):

```json
{
  "button": "finish",
  "data": {},
  "stack": [],
  "next_wizard": {
    "path": "{wyvern_share}/examples/agent-dag/wizard.json",
    "input": { "from": "welcome" },
    "ui_root": "{wyvern_share}/examples/agent-dag"
  }
}
```

Shipped post writes the default path only (`$WYVERN_REPO_ROOT/wyvern-dag-export.json` if set, else `./wyvern-dag-export.json`). `-o` is script/test-only (`WorkflowRunner` does not pass it).

## Acceptance criteria

1. `wizard_agent_dag_nav` drives: select **pair** → configure agent-1 → **back** → change to **solo** → review/finish. Finish `data.dag.layout_id` is `solo` with one node; agent-1 fields entered before back are restored when revisiting pair, then discarded after switching to solo.
2. Finish JSON includes `data.dag` with required wire-shape: `layout_id` (string), `nodes` (array of `{ id, name, role }`), `edges` (array of `[from, to]` pairs). `wizard_agent_dag` asserts this shape on finish; review-page HTML graph is illustrative only (no DOM render gate).
3. Welcome Agent DAG page states execution is deferred and emits the `next_wizard` object above; `workflow_welcome_chain_agent_dag` asserts the CLI resolves that hop (REQ-0126).
4. `export-agent-dag.py` runs as `workflow.post` and writes the default export path from finish stdin (REQ-0125). `--workflow-dry-run` writes nothing.
5. `workflow_export_agent_dag` asserts the export shape and that the wizard command has no execute hook (no agent spawn, Task delegation, or Rust DAG engine).

## Required validation

```bash
cargo test -p wyvern-cli --test workflow_export_agent_dag
cargo test -p wyvern-cli --test workflow_welcome_chain_agent_dag
cargo test -p wyvern-host --test wizard_agent_dag
cargo test -p wyvern-host --test wizard_agent_dag_nav
```

```bash
# manual (not a QA gate):
# wyvern share/wyvern/examples/agent-dag/wizard.json \
#   --ui-root share/wyvern/examples/agent-dag --viewer none
```

## Non-closure

- Wiring to a separate DAG execution runtime (post-publish, other repo)
- Validating DAG acyclicity in Rust
- turbo-flow / Svelte canvas
- Live execution status in the wizard
- Spawning Cursor / Claude / ATM agents from finish JSON
- `--emit-all`

## Authority

- REQ-0125, REQ-0126
- ADR-0023, ADR-0024, ADR-0006
- [agent-dag-execution-deferral.md](agent-dag-execution-deferral.md)
- [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md)
