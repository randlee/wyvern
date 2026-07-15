---
id: d.1
title: Wizard host — HTTP + initial stack snapshot
status: planning
branch: feature/phase-D-d1-wizard-host
target: integrate/phase-D
---

# Sprint d.1 — Wizard host: HTTP + initial stack snapshot

## Goal

Wire wizard HTTP and seed the stack: one session, cursor at first page, `GET /api/wizard/state` returns `snapshot()`.

## Hard dependencies

- Phase C **c.16** complete

## Deliverables

### Schema (`wyvern-schema`)

- `Command::Wizard`, `WizardPageDescriptor`, `WizardStackEntry`, `WizardCommand`, `WizardResult`
- `validate/wizard.rs` + `tests/validation_wizard.rs`
- Rules: `type: wizard`, `page.{id,title,html}`, optional `config`, optional `width`/`height` — see [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)

### Stack (`wyvern-wizard`)

Single type, one private history struct:

| File | Change |
|------|--------|
| `src/session.rs` | **`WizardSession`** — `new`, `snapshot` (d.1); `next`/`back`/`finish` stub or deferred to d.2 |
| `src/history.rs` | private `{ entries, cursor }` — seed with first page |

```rust
pub struct WizardSession { /* private */ }

impl WizardSession {
    pub fn new(command: &WizardCommand) -> Result<Self, WizardError>;
    pub fn snapshot(&self) -> WizardSnapshot;
}
```

No separate `WizardEngine` / `WizardNavigator` traits unless a second impl appears — prefer one concrete type.

### Host (`wyvern-host`)

| File | Change |
|------|--------|
| `routes/wizard.rs` | `GET /api/wizard/state`, `GET /wizard/**` |
| `session.rs` | `WizardSession` holder |
| `tests/wizard_state.rs`, `tests/wizard_routes.rs` | |

### CLI

- `pipeline.rs` dispatches `Command::Wizard` → `wyvern_host::run`

## Acceptance criteria

1. Workspace builds; clippy clean
2. `GET /api/wizard/state` → `{ config, page, page_data, stack }` for first page
3. Wizard HTML served at `/wizard/**`
4. Blocking dialogs still pass `--viewer none`

## Required validation

```bash
cargo test -p wyvern-schema validation_wizard
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_state wizard_routes
```

## Non-closure

- `POST /api/wizard/navigate`, `POST /api/wizard/finish` (d.2)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), ADR-0005, ADR-0006
