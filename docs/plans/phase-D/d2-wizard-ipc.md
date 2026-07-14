---
id: d.2
title: Wizard HTTP navigation + finish
status: planning
branch: feature/phase-D-d2-wizard-ipc
target: integrate/phase-D
---

# Sprint d.2 — Wizard HTTP navigation (was IPC)

## Goal

Wire non-terminal navigation and terminal finish routes. **Regression/navigation only** — `GET /api/wizard/state` owned by d.1.

## Hard dependencies

- **d.1** merged

## Deliverables

- `POST /api/wizard/navigate` — `next`, `back` per contract
- `POST /api/wizard/finish` — `finish`, `cancel`, `dismissed` (terminal only; cancel **not** on `navigate`)
- `ui/shared/wyvern-api.js` wizard helpers — `fetch` navigation + finish
- `cargo test -p wyvern-host` — navigate + finish integration tests
- L2 smoke: wizard next/back/finish with `--viewer none`

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `POST /api/wizard/navigate` with `{ "action": "next", ... }` advances history and returns new `url`
3. `POST /api/wizard/navigate` with `{ "action": "back" }` restores prior page
4. `POST /api/wizard/finish` with `{ "button": "finish", "data": {}, "stack": [...] }` completes wizard; stdout matches body
5. `POST /api/wizard/finish` with `{ "button": "cancel" }` returns cancel result (not via `navigate`)
6. Prior dialog types + d.1 wizard state regression passes

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_navigate
cargo test -p wyvern-host wizard_finish
# L2: wizard navigation smoke (headless)
```

## Non-closure

- History cursor edge cases (d.3), stack injection (d.4), DAG example (d.5), polish (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
- Historical wry `action` IPC — git history only
