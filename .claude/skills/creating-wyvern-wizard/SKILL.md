---
name: creating-wyvern-wizard
version: 0.1.0
description: >-
  Author Wyvern wizard packages (wizard.json, HTML/JS pages, workflow.pre /
  workflow.post). Use when creating or changing a wizard, picking
  vanilla-chrome vs workspace-canvas, running wyvern wizard lint or schema
  validate, or when the user mentions wizard pages, next_wizard, or
  creating-wyvern-wizard. Delegates page authoring to wyvern-wizard-js or
  wyvern-dag-wizard-js only. Do not use for Rust crates or rust-developer work.
depends_on:
  wyvern-wizard-js: 0.x
  wyvern-dag-wizard-js: 0.x
---

# Creating Wyvern Wizards

Layer 0 router (skills/agents guidelines v0.7). This file is the table of
contents. Load **one** stack doc and **one** type doc after G2. Do not load
every reference up front. Do not send work to `rust-developer`.

## Progressive disclosure load map

```
Layer 0  this SKILL.md
Layer 1  references/core/author-workflow.md     gates G1–G6 (fail-fast)
         references/core/platform-contract.md   host / CLI / page-JS seams
         references/core/validation-and-lint.md schema validate + wizard lint
         references/core/dataflow-contracts.md  exports / requires (G3–G4)
         references/installation-and-troubleshooting.md  G1 / verify wyvern (Step 1)
Layer 2  references/stacks/<stack>.md           after G2 — one stack only
Layer 3  references/wizard-types/<type>.md      after type pick (g.12–g.13)
Layer 4  templates/<stack>/                     skeleton copy (g.11+)
```

Read Layer 1 only as the current gate needs it. After G2, open **one** Layer 2
file from `references/stacks/registry.yaml`. After the type pick, open **one**
Layer 3 file if it exists; otherwise stay on Layer 1–2 and the golden example.

## Workflow gates (G1–G6)

Stop at the first failure. Full checklist:
`references/core/author-workflow.md`.

| Gate | Check | Load |
|------|-------|------|
| **G1** | `wizard.json` schema + `wyvern` on PATH | `installation-and-troubleshooting.md`, `platform-contract.md`, `validation-and-lint.md` |
| **G2** | Exactly one stack | `references/stacks/registry.yaml` |
| **G3** | Walkable page graph | `platform-contract.md`, `dataflow-contracts.md` |
| **G4** | `wyvern wizard lint` | `validation-and-lint.md` |
| **G5** | Tests + `--workflow-dry-run` | `author-workflow.md` |
| **G6** | Sprint / REQ hygiene | `author-workflow.md` |

## Stack picker (G2)

Default **`vanilla-chrome`**. Choose **`workspace-canvas`** only for a node-edge
workspace. Never both on one page without a stack-doc exception.

| Stack | When | Doc | Agent |
|-------|------|-----|-------|
| `vanilla-chrome` | Dialog form, picker, hook, welcome | `references/stacks/vanilla-chrome.md` | `wyvern-wizard-js` |
| `workspace-canvas` | Canvas / Agent DAG | `references/stacks/workspace-canvas.md` | `wyvern-dag-wizard-js` |

`vite-spa` is a registry comment only — not a Wave 3 target.

## Wizard type picker

Pick one type after the stack. Type docs ship in g.12–g.13; missing file is not
a g.10 failure — use the golden and stay on Layer 1–2.

| Type | Stack | Type doc | Golden |
|------|-------|----------|--------|
| template | vanilla-chrome | `references/wizard-types/template.md` | `share/wyvern/examples/template-picker/` |
| hook | vanilla-chrome | `references/wizard-types/hook.md` | `share/wyvern/examples/askuserquestion-hook/` |
| welcome-bridge | vanilla-chrome | `references/wizard-types/welcome-bridge.md` | `share/wyvern/welcome/` |
| dag | workspace-canvas | `references/wizard-types/dag-wizards.md` | `share/wyvern/examples/agent-dag/` |

## Agent delegation

Invoke via Agent Runner / Task using `.claude/agents/registry.yaml`. Receive
the agent's fenced JSON (`success`, `data`, `error`). Present a short summary;
do not re-wrap the envelope.

| Operation | Agent | Returns |
|-----------|-------|---------|
| Dialog pages, forms, hooks, welcome bridges | `wyvern-wizard-js` | Fenced JSON |
| Canvas DAG / workspace | `wyvern-dag-wizard-js` | Fenced JSON |
| Lint findings in the package | Same agent as the stack | Fenced JSON |

**Allowed agents:** `wyvern-wizard-js`, `wyvern-dag-wizard-js`.  
**Forbidden:** `rust-developer` and every other rust-* / CLI / host agent.

Agents land in g.11 / g.12. Until then, follow Layer 1–2 and the goldens; do
not invent a third authoring agent.

## Step 1 — Verify wyvern

See [installation-and-troubleshooting.md](references/installation-and-troubleshooting.md) if the binary is missing or the wrong build.

```bash
which wyvern && wyvern --version
```

G1 schema (same validator the CLI uses; exit 4 = schema fail):

```bash
wyvern path/to/wizard.json --viewer none
```

G4 lint (directory or `wizard.json`; exit 0 clean, 1 findings, 2 usage):

```bash
wyvern wizard lint path/to/wizard-dir
```

Exact flags and codes: `references/core/validation-and-lint.md`.

## Usage

1. Verify `wyvern` (Step 1).
2. G1 — schema-validate `wizard.json` before HTML/JS work.
3. G2 — pick one stack; load that stack doc only.
4. Pick one wizard type; load its type doc if present.
5. Delegate page authoring / lint to the stack's JS agent (never `rust-developer`).
6. G3–G6 — graph, lint, tests, doc hygiene.

Page JS has **no disk I/O**. Persist with `workflow.post`; pre-fill with
`workflow.pre` `config_patch`. Paths are strings in finish `data`.
