# Phase G Wave 3 — Wizard authoring platform

**Goal:** End-user wizard creation via a progressive-disclosure skill, JS-focused page-author agents, extended `wyvern wizard lint` (declared dataflow), and a documented default UI stack (`vanilla-chrome`) with an extension path (`workspace-canvas`).

**Base branch:** `integrate/phase-G`  
**PR target:** `integrate/phase-G`  
**Sprint docs are sole authority** for deliverables, acceptance criteria, and required validation.

Nav lint WIZARD-LINT-001–004 already landed on `feature/phase-G-wizard-lint` (`0578ebe`). Dataflow rules ship on a **new** g.9 branch so sprint ownership stays clean.

## Sprint map

| Sprint | Doc | Depends | Parallel group |
|--------|-----|---------|----------------|
| **g.8** | [g8-wizard-authoring-foundation.md](../g8-wizard-authoring-foundation.md) | `integrate/phase-G` | **W1** — start first |
| **g.9** | [g9-wizard-lint-dataflow.md](../g9-wizard-lint-dataflow.md) | g.8 (dataflow spec) | **W2** |
| **g.10** | [g10-creating-wyvern-wizard-skill.md](../g10-creating-wyvern-wizard-skill.md) | g.8 (stack registry) | **W2** |
| **g.11** | [g11-wyvern-wizard-js-agent.md](../g11-wyvern-wizard-js-agent.md) | g.8, g.10 (skill shell) | **W2–W3** |
| **g.12** | [g12-wyvern-dag-wizard-js-agent.md](../g12-wyvern-dag-wizard-js-agent.md) | g.8 | **W2** (parallel with g.11) |
| **g.13** | [g13-wizard-type-refs-and-templates.md](../g13-wizard-type-refs-and-templates.md) | g.10, g.11, g.12 | **W3** |
| **g.14** | [g14-wizard-authoring-ci-and-fixes.md](../g14-wizard-authoring-ci-and-fixes.md) | g.9, g.13 | **W4** |

g.9–g.14 sprint docs are written in those sprints. Until they exist, this table is the map only — do not treat missing sibling docs as g.8 failures.

## Parallel execution

```
W1 ── g.8 foundation (spec + stack registry + author-workflow)
         │
         ├──────────────────┬──────────────────┐
         ▼                  ▼                  ▼
W2      g.9 lint-dataflow   g.10 skill L0+core  g.12 dag-js agent
         │                  │                  │
         │                  └────────┬─────────┘
         │                           ▼
         │                         g.11 wizard-js agent + vanilla templates
         │                           │
         └───────────────────────────┼────────── W3 ── g.13 type refs + sc-compose
                                     ▼
                                   W4 ── g.14 CI + known nav-lint HTML fixes
```

**Merge order → `integrate/phase-G`:**

```
g.8 → (g.9 ∥ g.10 ∥ g.12) → g.11 (after g.10 shell) → g.13 → g.14
```

Each PR: squash into `integrate/phase-G`, pull the integrate worktree, rebase downstream branches.

## Worktrees

| Branch | Purpose |
|--------|---------|
| `feature/phase-G-g8-authoring-foundation` | g.8 docs + registry + core refs |
| `feature/phase-G-g9-wizard-lint-dataflow` | g.9 dataflow lint (do not reuse `feature/phase-G-wizard-lint`) |
| `feature/phase-G-g10-wizard-skill` | g.10 Layer 0 skill + remaining core refs |
| `feature/phase-G-g11-wizard-js-agent` | g.11 `wyvern-wizard-js` + vanilla-chrome templates |
| `feature/phase-G-g12-dag-js-agent` | g.12 `wyvern-dag-wizard-js` + DAG type ref |
| `feature/phase-G-g13-wizard-refs` | g.13 type docs + sc-compose |
| `feature/phase-G-g14-authoring-ci` | g.14 CI + known lint HTML fixes |

One branch = one worktree, all cut from `integrate/phase-G` (never `develop`). After each sprint: PR → `integrate/phase-G`, QA per the repo orchestration skill.

## What each sprint owns

| Sprint | Ships |
|--------|--------|
| **g.8** | Dataflow declaration spec, stack registry, `vanilla-chrome` / `workspace-canvas` docs, gates G1–G6 |
| **g.9** | `wyvern wizard lint` rules WIZARD-LINT-005–008 against `config.dataflow` |
| **g.10** | Thin `SKILL.md` router + `platform-contract.md` + `validation-and-lint.md` |
| **g.11** | Dialog-frame page-author agent + two-step vanilla-chrome skeleton |
| **g.12** | Canvas/DAG page-author agent + `wizard-types/dag-wizards.md` |
| **g.13** | Template / hook / welcome-bridge type refs + sc-compose test snippets |
| **g.14** | CI `wyvern wizard lint` gate + fix known WIZARD-LINT-002 (template-picker review Cancel) |

## Agent model (page authors)

| Task | Agent | Load first |
|------|-------|------------|
| Dialog wizard pages, forms, hooks, welcome bridges | `wyvern-wizard-js` | `references/stacks/vanilla-chrome.md` |
| Canvas DAG / workspace wizard | `wyvern-dag-wizard-js` | `references/stacks/workspace-canvas.md` |
| `wyvern wizard lint` failures in HTML/JS/contracts | Same JS agent as the stack | `references/core/dataflow-contracts.md` |

CLI, host, and schema changes are **maintainer work** outside this skill's delegation table. Wave 3 authors touch HTML/JS/CSS, `wizard.json`, and optional `workflow.pre` / `workflow.post` scripts.

## Out of scope (Wave 3)

- New host or CLI features beyond `wizard lint` dataflow rules
- Replacing turbo-flow with a live in-repo Svelte build
- Page-JS disk I/O (finish data + workflow scripts only)
- User extension stack (`vite-spa`) beyond a registry comment
- Mapping `--invoke` stdout into Claude Code `hookSpecificOutput` (still Wave 2 non-closure)
- DAG execution (still deferred; see [agent-dag-execution-deferral.md](../agent-dag-execution-deferral.md))
