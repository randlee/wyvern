---
id: g.8
title: Wizard authoring foundation (dataflow spec + stack registry)
status: complete (integrate)
branch: feature/phase-G-g8-authoring-foundation
worktree: ../wyvern-worktrees/feature/phase-G-g8-authoring-foundation
target: integrate/phase-G
---

# Sprint g.8 — Wizard authoring foundation

## Goal

Land the **normative authoring contracts** Wave 3 consumes: declared page `exports` / `requires` for dataflow lint (g.9), the UI stack registry (`vanilla-chrome` default, `workspace-canvas` supported), per-stack docs, and author gates G1–G6. This sprint ships **spec and skill-reference files only** — no new CLI subcommands, no lint-rule implementation, no `SKILL.md` Layer 0 router.

## Hard dependencies

- Wave 2 merged on `integrate/phase-G` (g.4 workflow/chain, g.5–g.7 example wizards)
- Phase D wizard chrome (`ui/shared/wizard-nav.js`, `ui/shared/wyvern-api.js`) — contracts cited, not re-implemented
- `wyvern wizard lint` nav rules WIZARD-LINT-001–004 exist on `feature/phase-G-wizard-lint` (g.9 extends them; this sprint does not merge that branch)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g8-wizard-authoring-foundation.md` | This sprint doc (sole AC / validation authority) |
| `docs/plans/phase-G/wave-3-wizard-authoring/README.md` | Wave 3 sprint map g.8–g.14 |
| `.claude/skills/creating-wyvern-wizard/references/core/dataflow-contracts.md` | `exports` / `requires` declaration + WIZARD-LINT-005–008 spec |
| `.claude/skills/creating-wyvern-wizard/references/core/author-workflow.md` | Author gates G1–G6 |
| `.claude/skills/creating-wyvern-wizard/references/stacks/registry.yaml` | Stack registry (`vanilla-chrome` default, `workspace-canvas` supported) |
| `.claude/skills/creating-wyvern-wizard/references/stacks/vanilla-chrome.md` | Default dialog-frame stack |
| `.claude/skills/creating-wyvern-wizard/references/stacks/workspace-canvas.md` | Canvas / DAG stack (turbo-flow pattern) |

`SKILL.md`, agents, type-specific refs, templates, and lint implementation are **later sprints** (see Non-closure).

### Contracts

`config.dataflow` is the normative declaration site. Host and schema treat `config` as opaque (ADR-0006); g.9 lint reads the object. No `wizard.json` schema field is added in this sprint.

Template-picker shape (golden `vanilla-chrome` example):

```json
{
  "type": "wizard",
  "page": { "id": "pick", "title": "Templates", "html": "pages/pick.html" },
  "config": {
    "dataflow": {
      "version": 1,
      "pages": {
        "pick": {
          "exports": {
            "template_id": "string",
            "variables": "object",
            "output_path": "string"
          }
        },
        "form": {
          "requires": ["template_id"],
          "exports": {
            "template_id": "string",
            "variables": "object",
            "output_path": "string"
          }
        },
        "review": {
          "requires": ["template_id", "output_path"],
          "terminal": true,
          "post_input": {
            "template_id": "string",
            "variables": "object",
            "output_path": "string"
          }
        }
      }
    }
  },
  "workflow": {
    "post": "{wyvern_share}/scripts/ext/apply-template.py"
  }
}
```

Agent-DAG finish export (`workspace-canvas`) stays the g.7 wire-shape, declared as a single terminal export:

```json
{
  "config": {
    "dataflow": {
      "version": 1,
      "pages": {
        "canvas": {
          "exports": { "dag": "object" }
        },
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

`data.dag` required keys (g.7 AC 2): `layout_id` (string), `nodes` (array of `{ id, name, role }`), `edges` (array of `[from, to]` pairs).

Stack registry (normative excerpt):

```yaml
stacks:
  vanilla-chrome:
    status: default
    lint_profile: nav + dataflow-v1
    agent: wyvern-wizard-js
  workspace-canvas:
    status: supported
    lint_profile: nav-limited + export-contract
    agent: wyvern-dag-wizard-js
```

Author gates (ordered, fail-fast):

| Gate | Check |
|------|--------|
| **G1** | `wizard.json` passes schema validation |
| **G2** | Stack chosen from `registry.yaml` (default `vanilla-chrome`) |
| **G3** | Page graph complete (`wizard.json` entry + `app.js` hops) |
| **G4** | `wyvern wizard lint` (nav now; dataflow when `config.dataflow` is declared — g.9) |
| **G5** | Tests + optional `--workflow-dry-run` |
| **G6** | Sprint doc / REQ updated if shipped behavior changed |

### Agent pairing (page authors only)

| Stack | Agent | Golden example |
|-------|-------|----------------|
| `vanilla-chrome` | `wyvern-wizard-js` | `share/wyvern/examples/template-picker/`, `share/wyvern/examples/askuserquestion-hook/` |
| `workspace-canvas` | `wyvern-dag-wizard-js` | `share/wyvern/examples/agent-dag/` |

Page JS has no disk I/O. Persist via `workflow.post`; pre-fill via `workflow.pre` `config_patch` (REQ-0124, REQ-0125).

## Acceptance criteria

1. Wave 3 map lists sprints g.8–g.14 with dependencies, parallel groups, and worktree branches; PRs target `integrate/phase-G`.
2. `dataflow-contracts.md` defines `config.dataflow` version 1 (`pages.<id>.exports`, `.requires`, `.terminal`, `.post_input`) and reserved codes **WIZARD-LINT-005** through **WIZARD-LINT-008**. Codes 001–004 remain nav-only (already shipped on the lint branch).
3. Registry lists exactly two Wave 3 stacks: `vanilla-chrome` (`status: default`) and `workspace-canvas` (`status: supported`). Future stacks may appear as comments or `status: experimental` only.
4. `vanilla-chrome.md` names required `/shared/wyvern-api.js` + `/shared/wizard-nav.js` includes, `collectCurrentPageData` / `wizardNextDescriptor` / `wizardNextWizard`, and lint profile `nav + dataflow-v1`.
5. `workspace-canvas.md` names `page.layout: "workspace"`, prebuilt `dist/` (no in-repo Svelte build for authors), `data.dag` export contract, custom toolbar vs wizard-nav, and lint profile `nav-limited + export-contract`.
6. `author-workflow.md` lists gates G1–G6 in fail-fast order and routes authors to `wyvern-wizard-js` / `wyvern-dag-wizard-js` only.
7. Authoring docs delegate only to `wyvern-wizard-js` and `wyvern-dag-wizard-js` (page authors). CLI/host work is out of this skill.

## Required validation

```bash
test -f docs/plans/phase-G/g8-wizard-authoring-foundation.md
test -f docs/plans/phase-G/wave-3-wizard-authoring/README.md
test -f .claude/skills/creating-wyvern-wizard/references/core/dataflow-contracts.md
test -f .claude/skills/creating-wyvern-wizard/references/core/author-workflow.md
test -f .claude/skills/creating-wyvern-wizard/references/stacks/registry.yaml
test -f .claude/skills/creating-wyvern-wizard/references/stacks/vanilla-chrome.md
test -f .claude/skills/creating-wyvern-wizard/references/stacks/workspace-canvas.md

rg -n "WIZARD-LINT-005|WIZARD-LINT-006|WIZARD-LINT-007|WIZARD-LINT-008" \
  .claude/skills/creating-wyvern-wizard/references/core/dataflow-contracts.md
rg -n "config.dataflow" \
  .claude/skills/creating-wyvern-wizard/references/core/dataflow-contracts.md
rg -n "status: default" .claude/skills/creating-wyvern-wizard/references/stacks/registry.yaml
rg -n "vanilla-chrome|workspace-canvas" \
  .claude/skills/creating-wyvern-wizard/references/stacks/registry.yaml
rg -n "^## Gate G[1-6]" \
  .claude/skills/creating-wyvern-wizard/references/core/author-workflow.md

# Agent pairing is page-author only
rg -n "wyvern-wizard-js|wyvern-dag-wizard-js" \
  .claude/skills/creating-wyvern-wizard/references/core/author-workflow.md \
  .claude/skills/creating-wyvern-wizard/references/stacks/registry.yaml
```

## Non-closure

- `SKILL.md` Layer 0 router, `platform-contract.md`, `validation-and-lint.md` — **g.10**
- WIZARD-LINT-005–008 implementation in `wyvern wizard lint` — **g.9**
- `.claude/agents/wyvern-wizard-js.md` + `templates/vanilla-chrome/` — **g.11**
- `.claude/agents/wyvern-dag-wizard-js.md` + `wizard-types/dag-wizards.md` — **g.12**
- Type refs (template, hook, welcome-bridge) and sc-compose J2 — **g.13**
- CI lint gate and known nav-lint HTML fixes (template-picker Cancel) — **g.14**
- Declaring `config.dataflow` on shipped g.5–g.7 examples (authors add it when g.9 lands)
- New `wizard.json` schema fields (`pages` map at top level)
- Live Svelte/Vite source build in-repo for canvas wizards
- User stack registry outside this skill (`vite-spa` / React remain comments only)

## Authority

- ADR-0006 (opaque `data` / `config`), ADR-0023, ADR-0024
- REQ-0124, REQ-0125, REQ-0126
- [wizard-workflow-architecture.md](wizard-workflow-architecture.md)
- [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md)
- [g5-askuserquestion-claude-code.md](g5-askuserquestion-claude-code.md)
- [g6-template-wizard.md](g6-template-wizard.md)
- [g7-dag-agent-execution.md](g7-dag-agent-execution.md)
- Phase D `ui/shared/wizard-nav.js` (d.7 chrome opt-in)
- [wave-3-wizard-authoring/README.md](wave-3-wizard-authoring/README.md)
