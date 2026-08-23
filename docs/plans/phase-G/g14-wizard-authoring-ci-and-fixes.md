---
id: g.14
title: Wizard authoring CI lint gate + known nav HTML fixes
status: complete (integrate)
branch: feature/phase-G-g14-authoring-ci
worktree: ../wyvern-worktrees/feature/phase-G-g14-authoring-ci
target: integrate/phase-G
---

# Sprint g.14 — Authoring CI and known lint fixes

## Goal

Add a **CI gate** that runs `wyvern wizard lint` on shipped example wizards,
and fix known WIZARD-LINT-002 findings (template-picker `review.html` missing
Cancel button).

## Hard dependencies

- g.9 dataflow lint merged (#110)
- g.13 type refs merged (docs-only; parallel OK for CI wiring)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g14-wizard-authoring-ci-and-fixes.md` | This sprint doc |
| `.github/workflows/ci.yml` | `wizard-lint` job on example packages |
| `share/wyvern/examples/template-picker/pages/review.html` | Add Cancel button (WIZARD-LINT-002) |
| `crates/wyvern/tests/wizard_lint.rs` | Update test: template-picker lint clean for 002 |

### CI job contract

Run after `cargo build -p wyvern-cli` on ubuntu-latest:

```bash
wyvern wizard lint share/wyvern/examples/template-picker
wyvern wizard lint share/wyvern/examples/askuserquestion-hook
wyvern wizard lint fixtures/wizard-dataflow-lint
```

Exit non-zero fails the job. template-picker must be **clean** after HTML fix.

## Acceptance criteria

1. CI workflow includes a `wizard-lint` job that runs `wyvern wizard lint` on
   at least template-picker, askuserquestion-hook, and dataflow fixture.
2. template-picker `review.html` has a cancel control (`data-wizard-cancel` or
   equivalent) and lint reports no WIZARD-LINT-002 for that package.
3. `wizard_lint.rs` integration test expects template-picker **clean** (or no 002).
4. Sprint doc lists validation commands.

## Required validation

```bash
test -f docs/plans/phase-G/g14-wizard-authoring-ci-and-fixes.md
rg -n "wizard lint" .github/workflows/ci.yml
rg -n "data-wizard-cancel|wizard-cancel" share/wyvern/examples/template-picker/pages/review.html

cargo build -p wyvern-cli
./target/debug/wyvern wizard lint share/wyvern/examples/template-picker
./target/debug/wyvern wizard lint share/wyvern/examples/askuserquestion-hook
./target/debug/wyvern wizard lint fixtures/wizard-dataflow-lint

cargo test -p wyvern-cli --test wizard_lint -- --test-threads=1
```

## Non-closure

- Declaring `config.dataflow` on all Wave 2 examples
- Lint gate on every example (agent-dag nav-limited profile)

## Authority

- [g9-wizard-lint-dataflow.md](g9-wizard-lint-dataflow.md)
- [validation-and-lint.md](../../.claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md)
