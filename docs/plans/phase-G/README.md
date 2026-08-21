# Phase G — CLI Extension Agent Usability (`integrate/phase-G`)

Phase G has **two waves**. Implementation PRs target **`integrate/phase-G`**. **Sprint docs are sole authority** for deliverables, acceptance criteria, and required validation.

| Wave | Sprints | Ships |
|------|---------|-------|
| **1 — CLI surfaces** | g.1–g.3 | `--help`, error-teaches, skill catalog |
| **2 — Welcome & examples** | g.4–g.7 | `wyvern guide` + workflow runner + three examples |

**Review Wave 2 one example at a time:** [examples-walkthrough.md](examples-walkthrough.md) (review order only; not a second AC list).

**Wave 2 requirements / decisions:** REQ-0124–REQ-0127, ADR-0023, ADR-0024.

---

## Wave 1 — CLI surfaces (g.1–g.3)

**Ordering:** Phase G runs **after Phase F** and **before Phase E**. Phase E agents benefit from discoverable extension help, skill catalog JSON, and error-teaches recovery on argv near-misses.

**Input artifact:** [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md) — agent-usability score 2/5; P0/P1 recommendations.

Phase F shipped the extension **engine**; Phase G Wave 1 makes the CLI **speak skill** so a cold agent needs no checkout docs.

### Core model (unchanged engine)

```
argv → match extension → optional preexec → template expand → validate → pipeline → host
```

Phase G Wave 1 adds **surfaces** around that path — no new dialog types, no new host *features* (see Boundaries for the RSH-007 hardening exception):

```
host flags stripped → global/extension help → match_with_diagnostics → expand → pipeline
                      → near-miss diagnostics (g.2) on no match
```

| Surface | Phase G adds |
|---------|----------------|
| `--help` / `-h` | First-class, exit 0; lists every shipped skill with copy-paste examples |
| Extension prefix `--help` | Skill card at **match time** (before requires skip; no suffix path required) |
| Fallthrough errors | Unknown suffix, incomplete prefix, skipped `requires` name the skill |
| `extensions list` | Skill index (text + `--json`); optional `show <id>` |
| Preexec failures | Child stderr in JSON envelope; spawn vs exit vs missing-file recovery |

Registry remains declarative in `share/wyvern/extensions.json`. Phase G may add optional **`description`** and **`examples`** fields to the schema for catalog output.

**Target agent-usability score:** 4 / 5 (per review rubric in phase-F-usability-review.md).

### Wave 1 phase acceptance (smoke)

```bash
wyvern --help
wyvern -h
wyvern compose render --help
wyvern extensions list --json | jq 'length >= 7'
wyvern notes.txt
PATH=/usr/bin wyvern sample.csv
wyvern md /nonexistent/file.csv
```

| Sprint | Doc | Status |
|--------|-----|--------|
| g.1 | [g1-help-surface.md](g1-help-surface.md) | complete (integrate) |
| g.2 | [g2-error-teaches.md](g2-error-teaches.md) | complete (integrate) |
| g.3 | [g3-skill-catalog.md](g3-skill-catalog.md) | complete (integrate) |

Contracts: [agent-usability-contract.md](agent-usability-contract.md), [skills-catalog-contract.md](skills-catalog-contract.md)

### Wave 1 non-closure

- `--interactive` argv expansion — **Phase E**
- MCP tool wrappers — **Phase E**
- User registry (`~/.config/wyvern/extensions.json`) — post-G
- `wyvern skills` argv alias — P2; defer unless trivial

---

## Wave 2 — Welcome guide & agent examples (g.4–g.7)

**Ordering:** After Wave 1 merged to `develop`.

**Goal:** Ship **`wyvern guide`** (REQ-0127) and **all three examples**. g.4 introduces workflow hooks (REQ-0124–0125, ADR-0023) and wizard chaining (REQ-0126, ADR-0024); g.5–g.7 consume them. **All sprints required** before Wave 2 closes.

**Architecture:** [wizard-workflow-architecture.md](wizard-workflow-architecture.md)

| Sprint | Example | Doc |
|--------|---------|-----|
| **g.4** | Welcome hub (`wyvern guide`) + workflow/chain Rust | [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md) |
| **g.5** | (a) AskUserQuestion hook installer | [g5-askuserquestion-claude-code.md](g5-askuserquestion-claude-code.md) |
| **g.6** | (b) Template wizard starter kit | [g6-template-wizard.md](g6-template-wizard.md) |
| **g.7** | (c) DAG agent demo + export | [g7-dag-agent-execution.md](g7-dag-agent-execution.md) |

Each sprint's **Required validation** is the only command list. Do not copy commands here.

### What Wave 2 does not close

- Mapping `--invoke` stdout into Claude Code `hookSpecificOutput` / native tool-result (version-specific)
- **DAG execution** — visualize + export in Wyvern; run in a separate project post-publish
- MCP / `--interactive` auto-chain — **Phase E**
- `--emit-all`, `wyvern chain` subcommand

### Workflow docs

| Doc | Topic |
|-----|-------|
| [wizard-workflow-architecture.md](wizard-workflow-architecture.md) | Pre/post scripts + `next_wizard` (Rust, CLI) |
| [workflow-chain-contract.md](workflow-chain-contract.md) | Chaining overview |
| [claude-code-hook-workflow.md](claude-code-hook-workflow.md) | g.5 Global/Repo toggles |
| [template-catalog-workflow.md](template-catalog-workflow.md) | g.6 template inventory |
| [agent-dag-execution-deferral.md](agent-dag-execution-deferral.md) | g.7 export-only |

---

## Boundaries (both waves)

- Wave 1: **`wyvern` CLI crate** + `extensions.json` schema fields; REQ-0134–REQ-0137; ADR-0022 Phase G amendment
- Wave 2: `crates/wyvern`, package `wyvern-cli` — workflow/chain + `share/wyvern/**` + `scripts/ext/**` — **no new dialog types**
- `wyvern help` = stdout; `wyvern guide` = wizard extension (do not merge)
- Host copies `next_wizard` and ignores `workflow`; host does not spawn scripts (ADR-0023, ADR-0024)
- No new `Command` enum variants; no new `ErrorCode` variants in `wyvern-schema` for Wave 1 near-misses

## Contract reference

- Extension match/expand: [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md)
- Usability rubric: [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md)

## Plan hardening

Artifacts: [`.plan-hardening/`](.plan-hardening/). Round table: [plan-hardening-rounds.md](plan-hardening-rounds.md).
