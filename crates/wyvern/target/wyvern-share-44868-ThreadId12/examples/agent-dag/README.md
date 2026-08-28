---
name: Agent DAG
description: Configure agents on a canvas and export data.dag; execution is deferred.
---

# Agent DAG

Three-page workspace wizard: pick a layout, configure agents on the turbo-flow
canvas, then review and finish. `workflow.post` exports `data.dag` via
`export-agent-dag.py`. Wyvern does not spawn agents or interpret the graph.

## Run

```bash
wyvern {wyvern_share}/examples/agent-dag/wizard.json \
  --ui-root {wyvern_share}/examples/agent-dag
```

Repo checkout:

```bash
wyvern share/wyvern/examples/agent-dag/wizard.json \
  --ui-root share/wyvern/examples/agent-dag
```

Lint:

```bash
wyvern wizard lint {wyvern_share}/examples/agent-dag
```
