---
id: S1.5
title: sc-lint integration
status: planned
branch: feature/p1-s5-sc-lint
target: integrate/phase-A
---

# Sprint S1.5 — sc-lint integration

## Goal

- Integrate `sc-lint` from sibling repo; CI fails on lint errors. Identify boundary rules for Phase 2 enforcement.

## Hard Dependencies

- S1.4 sc-observability (final Phase 1 code stable)

## Exact Targets

- `wyvern` repo lint config (path per `sc-lint` conventions)
- `.github/workflows/ci.yml`
- `docs/linting.md`
- `boundaries/` (placeholder TOMLs noted for Phase 2)

## Deliverables

- `sc-lint` configured from `../sc-lint` sibling path
- CI lint job step
- `docs/linting.md` documents how to run locally
- List of planned sc-lint-boundary rules for Phase 2 (no enforcement yet)

## Required Work

- Document sibling clone requirement alongside `sc-observability`
- Lint passes on all Phase 1 code with zero warnings
- sc-lint-boundary **planning** note only — enforcement starts Phase 2 per project plan

## This Sprint Does Not Close

- `boundaries/*.toml` enforcement in CI (Phase 2)
- sc-lint-boundary sprint (planning activity, not implementation)

## Acceptance Criteria

- `sc-lint` runs clean on workspace from repo root
- CI workflow includes lint step and fails on violations
- `docs/linting.md` documents local invocation
- Phase 1 complete: all eight sprint deliverables integrated on `integrate/phase-A`

## Required Validation

- `sc-lint` (or project-documented equivalent command)
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- Phase 1 acceptance criteria #1–#3 from `docs/plans/project-plan.md`
