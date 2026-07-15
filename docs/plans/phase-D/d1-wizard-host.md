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

- Phase C **c.16** complete (`wyvern-host`, `ui/`, no `wyvern-window`)
- `integrate/phase-D` branched from post-c.16 baseline

## Deliverables

### Schema (`wyvern-schema`)

| File | Change |
|------|--------|
| `crates/wyvern-schema/src/command.rs` | `Command::Wizard(WizardCommand)` variant |
| `crates/wyvern-schema/src/wizard.rs` | **new** — `WizardPageDescriptor`, `WizardStackEntry`, `WizardCommand`, `WizardResult` |
| `crates/wyvern-schema/src/validate/wizard.rs` | **new** — validation module |
| `crates/wyvern-schema/tests/validation_wizard.rs` | **new** — contract examples |

**Validation rules (REQ-0017, REQ-0042, REQ-0026):**

- `type` must be `"wizard"`
- `page` required: non-empty `id`, `title`, `html`; optional `layout` ∈ `dialog|workspace` (default `dialog`) — graph/Flowise pages use `workspace` per [viewport-sizing.md](viewport-sizing.md)
- `config` optional; default `{}`
- `width` / `height` optional `u32`; when present, `width >= 320`, `height >= 200`, `width <= 800`, `height <= 600` (REQ-0041 wizard override)
- Reject unknown top-level keys (same policy as other dialog types)
- `WizardResult` wire shape: `button` ∈ `finish|cancel|dismissed`, `data` object, `stack` array of `{page, data}`

Types match [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md) wizard section.

### Wizard engine trait (`wyvern-wizard`)

| File | Change |
|------|--------|
| `crates/wyvern-wizard/src/lib.rs` | Re-export public API only |
| `crates/wyvern-wizard/src/engine.rs` | **new** — `WizardEngine` trait + `WizardSnapshot` |
| `crates/wyvern-wizard/src/browser_history.rs` | **new** — private `BrowserHistory` impl (stub in d.1; behaviour in d.3) |

**Public trait (d.1 — initial load only):**

```rust
pub trait WizardEngine {
    fn snapshot(&self) -> WizardSnapshot;
    fn current_page_url_path(&self) -> &str; // relative html path for static route
}

pub struct WizardSnapshot {
    pub config: serde_json::Value,
    pub page: WizardPageDescriptor,
    pub page_data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
```

- `WizardEngine::new(command: &WizardCommand) -> Result<impl WizardEngine, WizardError>` — seeds history with first page, empty `page_data`, stack `[{page, data: {}}]` or `[]` per contract (initial stack contains current page entry when first data collected — see d.4; d.1 may return empty stack + current `page` only)
- **No** `navigate_*` methods until d.2; trait grows in later sprints without breaking host call sites (default methods or extension trait `WizardNavigator` added in d.2)

### Host routes (`wyvern-host`)

| File | Change |
|------|--------|
| `crates/wyvern-host/src/routes/wizard.rs` | **new** — `GET /api/wizard/state`, wizard static `GET /wizard/**` |
| `crates/wyvern-host/src/session.rs` | Hold `Option<Box<dyn WizardEngine>>` for wizard sessions |
| `crates/wyvern-host/src/server.rs` | Register wizard routes when `Command::Wizard` |
| `crates/wyvern-host/src/lib.rs` | `run()` dispatches `Command::Wizard` |
| `crates/wyvern-host/tests/wizard_state.rs` | **new** |
| `crates/wyvern-host/tests/wizard_routes.rs` | **new** — static HTML serve + path traversal guard |

**Route behaviour:**

- `GET /wizard/**` — serve files relative to wizard `page.html` directory; reject `..` segments (same policy as dialog static files)
- `GET /api/wizard/state` — serialize `engine.snapshot()` → `WizardStateResponse` JSON (`type: "wizard"`)
- Host **never** reads history internals — only `snapshot()`

### CLI pipeline (`wyvern`)

- `pipeline.rs` — pass `Command::Wizard` to `wyvern_host::run`
- No wizard-specific logic in CLI beyond validation + host dispatch

### Boundaries

- Update `boundaries/wyvern-host/host.toml` — `wizard_session`, `wizard_routes` in `io_owns`
- Update `boundaries/wyvern-wizard/wizard.toml` — `wizard_engine_trait` in `io_owns`

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `cargo test -p wyvern-schema validation_wizard` passes
3. Wizard JSON with valid `page.html` serves that file at `GET /wizard/...`
4. `GET /api/wizard/state` returns `config`, `page`, `page_data`, `stack` per [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
5. `width`/`height` echoed when provided in command
6. Blocking dialog types still regression-pass with `--viewer none`
7. Host integration tests use `WizardEngine` trait only — no `use wyvern_wizard::browser_history`

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-schema validation_wizard
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_state
cargo test -p wyvern-host wizard_routes
sc-lint check native --config .sc-lint.toml
```

## Non-closure

- `POST /api/wizard/navigate`, `POST /api/wizard/finish` (d.2)
- History cursor semantics (d.3), stack injection polish (d.4), example wizard (d.5), polish (d.6)
- `ui/shared/wyvern-api.js` wizard helpers (d.2)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
- REQ-0017, REQ-0026, REQ-0042, NFR-0008
