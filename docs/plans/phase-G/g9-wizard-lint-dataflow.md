---
id: g.9
title: wyvern wizard lint dataflow rules (WIZARD-LINT-005–008)
status: in-review
branch: feature/phase-G-g9-wizard-lint-dataflow
worktree: ../wyvern-worktrees/feature/phase-G-g9-wizard-lint-dataflow
target: integrate/phase-G
---

# Sprint g.9 — Dataflow lint (`config.dataflow`)

## Goal

Implement **WIZARD-LINT-005–008** in `wyvern wizard lint`, reading declared
page `exports` / `requires` from `config.dataflow` (and optional HTML meta).
Nav rules 001–004 remain as shipped on PR #105. Omitted `config.dataflow` skips
005–008 for that package.

## Hard dependencies

- g.8 `dataflow-contracts.md` on `integrate/phase-G`
- Nav lint WIZARD-LINT-001–004 on `integrate/phase-G` (#105)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g9-wizard-lint-dataflow.md` | This sprint doc (sole AC / validation authority) |
| `crates/wyvern-wizard/src/dataflow.rs` | Pure dataflow lint (005–008) |
| `crates/wyvern-wizard/src/lint.rs` | Extended `LintCode` 005–008 |
| `crates/wyvern/src/wizard_cmd.rs` | CLI wiring: graph + JS scan + dataflow pass |
| `fixtures/wizard-dataflow-lint/` | Minimal wizard fixture for integration tests |
| `crates/wyvern/tests/wizard_lint_dataflow.rs` | Integration tests for 005–008 |

Update `validation-and-lint.md` to mark 005–008 **implemented** (not reserved).

### Contracts

Algorithms match [dataflow-contracts.md](../../.claude/skills/creating-wyvern-wizard/references/core/dataflow-contracts.md) §5:

| Code | Rule |
|------|------|
| **WIZARD-LINT-005** | `requires` missing on any reachable path; export type conflict |
| **WIZARD-LINT-006** | Terminal `post_input` key not in page `exports`; dag field literals |
| **WIZARD-LINT-007** | `next_wizard.input` key not in target `requires` (when target resolvable) |
| **WIZARD-LINT-008** | Literal `data.*` / `page_data.*` / `stack[].data.*` read with no export |

## Acceptance criteria

1. `wyvern wizard lint` emits WIZARD-LINT-005–008 when `config.dataflow` is declared and a rule is violated.
2. Packages **without** `config.dataflow` still lint nav only (001–004); no 005–008 findings.
3. `wyvern wizard lint --help` lists codes 001–008.
4. `fixtures/wizard-dataflow-lint/` demonstrates clean dataflow for a two-step wizard.
5. Integration test proves 005 fires on a fixture with unsatisfied `requires`.
6. `validation-and-lint.md` documents 005–008 as implemented (not “reserved for g.9”).

## Required validation

```bash
test -f docs/plans/phase-G/g9-wizard-lint-dataflow.md
test -f crates/wyvern-wizard/src/dataflow.rs
test -f fixtures/wizard-dataflow-lint/wizard.json

cargo test -p wyvern-wizard dataflow
cargo test -p wyvern wizard_lint
cargo test -p wyvern wizard_lint_dataflow

cargo build -p wyvern-cli
./target/debug/wyvern wizard lint --help | rg -q "WIZARD-LINT-005"
./target/debug/wyvern wizard lint --help | rg -q "WIZARD-LINT-008"
./target/debug/wyvern wizard lint fixtures/wizard-dataflow-lint

rg -n "W005UnsatisfiedRequire|WIZARD-LINT-005" crates/wyvern-wizard/src/
rg -n "implemented|005–008" .claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md
```

## Non-closure

- Declaring `config.dataflow` on all Wave 2 examples (authors add when ready)
- CI gate for lint — **g.14**
- template-picker WIZARD-LINT-002 HTML fix — **g.14**
- Runtime host type checks inside finish `data`

## Authority

- [g8-wizard-authoring-foundation.md](g8-wizard-authoring-foundation.md)
- [dataflow-contracts.md](../../.claude/skills/creating-wyvern-wizard/references/core/dataflow-contracts.md)
- [wave-3-wizard-authoring/README.md](wave-3-wizard-authoring/README.md)
