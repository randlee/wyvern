---
id: a.7
title: sc-lint integration
status: planned
branch: feature/phase-A-a7-sc-lint
target: integrate/phase-A
---

# Sprint a.7 — sc-lint integration

## Goal

- Configure `sc-lint` from sibling repo; CI fails on lint violations.

## Hard Dependencies

- a.6 sc-observability

## Exact Targets

- `.sc-lint.toml` (repo root)
- `crates/wyvern/Cargo.toml` (if lint references workspace)
- `.github/workflows/ci.yml`
- `docs/linting.md`

## Deliverables

- Sibling path documented: `../../sc-lint`
- `.sc-lint.toml` at repo root with workspace scope
- CI lint step runs canonical command
- Phase A complete when all **seven** sprint deliverables (a.1–a.7) merge to `integrate/phase-A`

## Explicit Code Samples

```toml
# .sc-lint.toml (minimal — extend per sc-lint docs)
[workspace]
root = "."
```

```bash
# Canonical local/CI invocation (from repo root, sc-lint on PATH)
sc-lint check --config .sc-lint.toml
```

Sibling checkout (same layout as observability):

```
github/
  wyvern/              # or wyvern-worktrees/<branch>
  sc-lint/
  sc-observability/
```

## This Sprint Does Not Close

- `boundaries/*.toml` CI enforcement (Phase B planning activity)
- Per-crate boundary TOML content

## Acceptance Criteria

- `sc-lint check --config .sc-lint.toml` passes with zero warnings
- CI lint step runs on all matrix legs per [a6-sc-observability.md](a6-sc-observability.md) Phase A CI policy
- `docs/linting.md` documents sibling clone + canonical command
- Phase A acceptance criteria #1–#3 pass after a.1–a.7 integrated

## Required Validation

- `sc-lint check --config .sc-lint.toml`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- Phase A E2E manual gate from `docs/plans/project-plan.md`
