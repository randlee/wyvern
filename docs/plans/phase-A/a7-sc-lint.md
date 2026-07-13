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
- CI installs `sc-lint` from crates.io on every matrix leg
- `docs/linting.md` documents install + canonical command

## Explicit Code Samples

```toml
# .sc-lint.toml (minimal — extend per sc-lint docs)
[workspace]
root = "."
```

```bash
cargo install sc-lint --version 0.4 --locked
sc-lint check --config .sc-lint.toml
```

```yaml
- name: Install sc-lint from crates.io
  run: cargo install sc-lint --version 0.4 --locked
- name: sc-lint check
  run: sc-lint check --config .sc-lint.toml
```

## This Sprint Does Not Close

- `boundaries/*.toml` CI enforcement (Phase B planning)
- Phase completion gate (see [README.md](README.md))

## Acceptance Criteria

- `sc-lint check --config .sc-lint.toml` passes with zero warnings
- CI lint step on ubuntu, macOS, and Windows
- `docs/linting.md` complete

## Required Validation

- `cargo install sc-lint --version 0.4 --locked && sc-lint check --config .sc-lint.toml`
- CI matrix: [README.md — CI validation](README.md#ci-validation-authoritative)
- Manual phase gates: [README.md — Phase acceptance](README.md#phase-acceptance-manual--not-ci-automated)
- `cargo clippy --workspace -- -D warnings`
