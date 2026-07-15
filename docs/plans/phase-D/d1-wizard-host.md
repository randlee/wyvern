---
id: d.1
title: Wizard host — HTTP HTML load + initial state
status: planning
branch: feature/phase-D-d1-wizard-host
target: integrate/phase-D
---

# Sprint d.1 — Wizard host: HTTP HTML load + `GET /api/wizard/state`

## Goal

Serve wizard pages over HTTP and expose initial wizard state. **d.1 owns `GET /api/wizard/state`** — later sprints consume it; they do not re-implement the route.

## Hard dependencies

- Phase C **c.16** complete

## Deliverables

- `GET /wizard/**` static routes for wizard page HTML from command `page.html` paths
- **`GET /api/wizard/state`** on `wyvern-host` — initial `config`, `page`, `page_data`, `stack`, optional `width`/`height`
- `wyvern-wizard` wired into host for wizard session state (pure logic; host owns HTTP)
- `wyvern-host::run` handles `Command::Wizard` — serve first page only (no navigate/finish yet)
- Wizard command validation in `wyvern-schema` (Phase D gate)
- `cargo test -p wyvern-host` — wizard state route unit tests

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. Wizard JSON serves `page.html` over HTTP from command path
3. `GET /api/wizard/state` returns `config`, `page`, `page_data`, `stack` per [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
4. `width`/`height` passed when provided in command
5. Blocking dialog types still regression-pass with `--viewer none`

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_state
cargo test -p wyvern-host wizard_routes
```

## Non-closure

- `POST /api/wizard/navigate`, `POST /api/wizard/finish` (d.2)
- History cursor semantics (d.3), stack injection polish (d.4), example wizard (d.5), polish (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
