---
id: d.3
title: Browser-history navigation model
status: planning
branch: feature/phase-D-d3-history-nav
target: integrate/phase-D
---

# Sprint d.3 — Browser-history navigation model

## Goal

Implement ADR-0005 cursor-over-array history in `wyvern-wizard`; host exposes state via d.1 `GET /api/wizard/state`.

## Hard dependencies

- **d.2** merged

## Deliverables

- `wyvern-wizard` history module — cursor, push, back, forward-restore, forward-truncate
- Host routes call into `wyvern-wizard` for navigate side effects
- Unit tests in `wyvern-wizard` covering all four history cases

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. Forward navigation pushes page + data, advances cursor
3. Back moves cursor back without truncating forward history
4. Forward on same next-page restores cached page data
5. Forward on different next-page truncates forward history and pushes new page
6. History state verified by unit tests covering all four cases

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_history
```

## Non-closure

- Stack injection polish (d.4), DAG example (d.5), polish (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md) — `POST /api/wizard/navigate`
- ADR-0005
