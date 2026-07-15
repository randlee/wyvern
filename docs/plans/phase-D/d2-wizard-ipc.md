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

### Trait extension (`wyvern-wizard`)

Add navigation methods to the public API (prefer extending `WizardEngine` or a composable `WizardNavigator` trait re-exported from `lib.rs`):

```rust
pub enum WizardNavAction { Next, Back }

pub struct NavigateOutcome {
    pub url_path: String,       // e.g. "pages/step-2.html" for host to build full URL
    pub snapshot: WizardSnapshot,
}

pub trait WizardNavigator: WizardEngine {
    fn navigate_next(
        &mut self,
        data: serde_json::Value,
        next: WizardPageDescriptor,
    ) -> Result<NavigateOutcome, WizardError>;
    fn navigate_back(&mut self, data: serde_json::Value) -> Result<NavigateOutcome, WizardError>;
    fn finish(
        &self,
        button: ButtonLabel,
        data: serde_json::Value,
        stack: Vec<WizardStackEntry>,
    ) -> WizardResult;
}
```

- d.2 may use **stub** history (push/pop) inside private `BrowserHistory`; full ADR-0005 semantics land in d.3 without changing the trait surface
- Host holds `Box<dyn WizardNavigator>` (or concrete type implementing both traits)

### Host routes (`wyvern-host`)

| File | Change |
|------|--------|
| `crates/wyvern-host/src/routes/wizard.rs` | Add `POST /api/wizard/navigate`, `POST /api/wizard/finish` |
| `crates/wyvern-host/tests/wizard_navigate.rs` | **new** |
| `crates/wyvern-host/tests/wizard_finish.rs` | **new** |

**`POST /api/wizard/navigate`:**

- Deserialize `WizardNavigateRequest`
- Reject `action: cancel` → **400** `{"error":"invalid_action","message":"cancel uses POST /api/wizard/finish"}`
- `next`: call `navigate_next(data, next)`; respond `{"ok":true,"url":"http://127.0.0.1:PORT/wizard/..."}`
- `back`: call `navigate_back(data)`; same response shape
- On `WizardError` → **400** with structured body per [http-post-schema.md](../phase-C/http-post-schema.md)

**`POST /api/wizard/finish`:**

- Deserialize `WizardFinishRequest`
- `button` ∈ `finish|cancel|dismissed`
- Build `WizardResult`, write to session result channel, initiate graceful shutdown (one-shot)
- Stdout body **identical** to POST body

### UI (`ui/shared/wyvern-api.js`)

Add wizard helpers (names are authoritative):

```javascript
// GET state — used on every page load
export async function wyvernWizardState() { ... }

// Non-terminal navigation
export async function wyvernWizardNext({ data, next }) { ... }
export async function wyvernWizardBack({ data = {} }) { ... }

// Terminal
export async function wyvernWizardFinish({ button, data, stack }) { ... }
```

- Bootstrap: on `DOMContentLoaded`, call `wyvernWizardState()`, set `window.wyvern = { config, page, page_data, stack, ... }`
- All helpers use `fetch` against same-origin host; no wry IPC

### L2 smoke

- New spec under `tests/l2/wizard-navigate.spec.ts` (or extend existing Playwright suite): headless `--viewer none`, mock wizard pages under `tests/fixtures/wizard/`

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `POST /api/wizard/navigate` `{ "action": "next", "data": {...}, "next": {...} }` advances and returns new `url`
3. `POST /api/wizard/navigate` `{ "action": "back" }` returns prior page `url`
4. `POST /api/wizard/finish` `{ "button": "finish", "data": {}, "stack": [...] }` completes; stdout matches body
5. `POST /api/wizard/finish` `{ "button": "cancel" }` returns cancel result (not via `navigate`)
6. `POST /api/wizard/navigate` with `"action": "cancel"` returns **400**
7. Prior dialog types + d.1 wizard state regression passes
8. Host route handlers do not import `wyvern_wizard::browser_history` (trait-only)

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_navigate
cargo test -p wyvern-host wizard_finish
# L2: wizard navigation smoke (headless)
npx playwright test tests/l2/wizard-navigate.spec.ts
```

## Non-closure

- History cursor edge cases (d.3), stack injection (d.4), DAG example (d.5), polish (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
- REQ-0025 (HTTP amendment), REQ-0066
- Historical wry `action` IPC — git history only
