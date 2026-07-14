---
id: d.4
title: Stack injection and data restoration
status: planning
branch: feature/phase-D-d4-stack-inject
target: integrate/phase-D
---

# Sprint d.4 — Stack injection and data restoration

## Goal

Ensure full stack and per-page data round-trip through navigation and `GET /api/wizard/state`.

## Hard dependencies

- **d.3** merged

## Deliverables

- `GET /api/wizard/state` returns complete `stack` with all prior `{page, data}` entries
- `page_data` populated with this page's previously collected data on restore
- `ui/shared/wyvern-api.js` — page bootstrap reads `stack` from wizard state
- Integration tests — back/forward data restoration

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `GET /api/wizard/state` returns `stack` with all prior `{page, data}` entries
3. `page_data` populated with this page's previously collected data on restore
4. Page JS reads `stack` from wizard state bootstrap (via `wyvern-api.js`)
5. Data round-trips correctly through JSON serialization

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-wizard stack_restore
cargo test -p wyvern-host wizard_stack
```

## Non-closure

- DAG example wizard (d.5), polish (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md) — `GET /api/wizard/state`
