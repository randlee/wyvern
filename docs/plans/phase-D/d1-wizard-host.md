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

- `Command::Wizard`, `WizardPageDescriptor`, `WizardPageLayout`, `WizardStackEntry`, `WizardCommand`, `WizardResult`
- **`WizardStateResponse`** — wire DTO for `GET /api/wizard/state` (see [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md))
- `validate/wizard.rs` + `tests/validation_wizard.rs`
- Rules: `type: wizard`, `page.{id,title,html}`, optional `page.layout` (`dialog` | `workspace`), optional `config`, optional `width`/`height` — see [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
- Static HTML paths resolve from `page.html` relative to `--ui-root` (no separate `page_html` field)

### Stack (`wyvern-wizard`)

Single type, one private history struct:

| File | Change |
|------|--------|
| `src/session.rs` | **`WizardSession`** — `new`, `snapshot` only (d.1); navigate/finish land in d.2 |
| `src/history.rs` | private `{ entries, cursor }` — seed with first page |

```rust
pub struct WizardSession { /* private */ }

/// GET /api/wizard/state shape — prior entries only in `stack` (REQ-0024).
pub struct WizardSnapshot {
    pub config: serde_json::Value,
    pub page: WizardPageDescriptor,
    pub page_data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>, // entries[0..cursor], exclusive of current
}

pub enum WizardError {
    AtFirstPage,
    InvalidCommand(String),
    StackMismatch, // client finish stack ≠ session-derived stack
}

impl WizardSession {
    pub fn new(command: &WizardCommand) -> Result<Self, WizardError>;
    pub fn snapshot(&self) -> WizardSnapshot;
}
```

No separate `WizardEngine` / `WizardNavigator` traits unless a second impl appears — prefer one concrete type.

### Host (`wyvern-host`)

| File | Change |
|------|--------|
| `routes/wizard.rs` | `GET /api/wizard/state`, `GET /wizard/**`, `GET /shared/**` |
| `session.rs` | `WizardSession` holder |
| `tests/wizard_state.rs`, `tests/wizard_routes.rs`, `tests/wizard_shared_mount.rs` | |

**Static routing (normative — dual mount):**

| Route | Source | Purpose |
|-------|--------|---------|
| `GET /wizard/**` | `--ui-root` | Wizard page HTML + example assets (`page.html` paths) |
| `GET /shared/**` | packaged `ui/` root via `HostOptions.shared_ui_root` (not `--ui-root`) | Shared JS/CSS (`wyvern-api.js`, `wizard-nav.js`, etc.) |

**`shared_ui_root` (normative):** always resolves to packaged `ui/` (install `share/wyvern/ui/`; dev workspace = repo `ui/`). `--ui-root` overrides wizard pages only. Host test must assert `/shared/wyvern-api.js` serves when `--ui-root examples/wizards/layout-picker`.

Wizard pages load shared helpers via absolute `/shared/…` URLs regardless of `--ui-root`. Example: `--ui-root examples/wizards/layout-picker` still serves `/shared/wyvern-api.js` from packaged `ui/shared/`.

**Wizard URL rule (normative):**

- `DialogHandle.dialog_url` for `Command::Wizard` = `http://127.0.0.1:{PORT}/wizard/{page.html}`
- `page.html` is relative to `--ui-root`; host static root serves `GET /wizard/**` under that directory
- Example: `--ui-root examples/wizards/layout-picker` + `page.html: "pages/layout-picker.html"` → `/wizard/pages/layout-picker.html`

### CLI

- `pipeline.rs` dispatches `Command::Wizard` → `wyvern_host::run`

### Boundaries

- `boundaries/wyvern-host/host.toml` — verify `wyvern-host → wyvern-wizard` edge enforced

## Acceptance criteria

1. Workspace builds; clippy clean
2. `GET /api/wizard/state` returns full `WizardStateResponse` wire shape: `{ type, config, page, page_data, stack, width?, height? }` — on first page `stack: []` (REQ-0024); authority: [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
3. Wizard HTML served at `/wizard/**` from `--ui-root` + `page.html`
4. `GET /shared/wyvern-api.js` succeeds when `--ui-root` is an example directory (dual-mount)
5. `page.layout` optional field validates (`dialog` | `workspace`) when present
6. Blocking dialogs still pass `--viewer none`
7. `sc-lint check native --config .sc-lint.toml` passes

## Required validation

```bash
cargo test -p wyvern-schema validation_wizard
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_state wizard_routes wizard_shared_mount
sc-lint check native --config .sc-lint.toml
```

## Non-closure

- `POST /api/wizard/navigate`, `POST /api/wizard/finish` (d.2)
- `navigate_next` / `navigate_back` / `finish` on `WizardSession` (d.2)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), ADR-0005, ADR-0006, ADR-0007
