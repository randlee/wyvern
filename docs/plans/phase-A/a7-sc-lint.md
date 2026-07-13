---
id: a.7
title: sc-lint integration
status: planned
branch: feature/phase-A-a7-sc-lint
target: integrate/phase-A
---

# Sprint a.7 — sc-lint integration

## Goal

- Configure **`sc-lint` from crates.io**; CI fails on lint violations on all platforms.

## Hard Dependencies

- a.6 sc-observability

## Exact Targets

- `.sc-lint.toml` (repo root)
- `.github/workflows/ci.yml`
- `docs/linting.md`

## Deliverables

- `.sc-lint.toml` at repo root with workspace scope
- CI installs `sc-lint` from crates.io and runs canonical command on every matrix leg
- Phase A complete when all **seven** sprint deliverables (a.1–a.7) merge to `integrate/phase-A`

## Explicit Code Samples

```toml
# .sc-lint.toml (minimal — extend per sc-lint docs)
[workspace]
root = "."
```

```bash
# Local: install once from crates.io
cargo install sc-lint --version 0.4 --locked

# Canonical invocation (from repo root)
sc-lint check --config .sc-lint.toml
```

```yaml
# .github/workflows/ci.yml — lint job or step (all matrix legs)
- name: Install sc-lint from crates.io
  run: cargo install sc-lint --version 0.4 --locked

- name: sc-lint check
  run: sc-lint check --config .sc-lint.toml
```

**No** sibling checkout or path reference to a local `sc-lint` repo.

## This Sprint Does Not Close

- `boundaries/*.toml` CI enforcement (Phase B planning activity)
- Per-crate boundary TOML content

## Acceptance Criteria

- `sc-lint check --config .sc-lint.toml` passes with zero warnings (after `cargo install sc-lint`)
- CI lint step runs on `ubuntu-latest`, `macos-latest`, and `windows-latest`
- `docs/linting.md` documents crates.io install + canonical command
- Phase A acceptance criteria #1–#3 pass after a.1–a.7 integrated

## Required Validation

- `cargo install sc-lint --version 0.4 --locked && sc-lint check --config .sc-lint.toml`
- `cargo test --workspace` (all CI matrix legs)
- `cargo clippy --workspace -- -D warnings`
- Phase A E2E gate from `docs/plans/project-plan.md`
