---
id: d.2
title: Wizard HTTP navigation + finish + browser stack
status: planning
branch: feature/phase-D-d2-wizard-ipc
target: integrate/phase-D
---

# Sprint d.2 — Wizard HTTP navigation + finish + browser stack

## Goal

Complete the stack: `navigate_next`, `navigate_back`, `finish` on `WizardSession` plus HTTP routes and page JS helpers. **Ship full ADR-0005 browser history here** — not a stub for d.3 to replace.

## Hard dependencies

- **d.1** merged

## Deliverables

### Stack (`wyvern-wizard`)

Extend `WizardSession` (same type as d.1):

```rust
/// Host uses this to build navigate response URL + state refresh.
pub struct NavigateOutcome {
    pub page: WizardPageDescriptor,
    pub page_data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>, // entries[0..cursor], prior only
}

impl WizardSession {
    pub fn navigate_next(&mut self, data: Value, next: WizardPageDescriptor) -> Result<NavigateOutcome, WizardError>;
    pub fn navigate_back(&mut self, data: Value) -> Result<NavigateOutcome, WizardError>;
    pub fn finish(&self, button: ButtonLabel, data: Value, stack: Vec<WizardStackEntry>) -> Result<WizardResult, WizardError>;
}
```

**`snapshot()` derivation (normative — REQ-0024):**

```rust
// cursor indexes current entry; stack = prior entries only
let page = entries[cursor].page.clone();
let page_data = entries[cursor].data.clone();
let stack: Vec<_> = entries[0..cursor]
    .iter()
    .map(|e| WizardStackEntry { page: e.page.clone(), data: e.data.clone() })
    .collect();
```

`history.rs` implements ADR-0005 (cursor, truncate on branch, restore on forward-same-page).

**`navigate_next` data write (normative):** write `data` onto `entries[cursor]` **before** push/truncate-forward/advance. Forward-same-page restore returns cached entry data unless new `data` is non-empty (overwrite).

**`navigate_back` data write (normative):** write `data` onto `entries[cursor]` **before** `cursor--`. Restored `page_data` comes from destination entry.

**`navigate_back` at cursor=0:** returns `WizardError::AtFirstPage` → host maps to HTTP **400** (no silent no-op).

**Finish stack algorithm (normative):**

1. Apply `request.data` onto `entries[cursor].data` (merge/replace per opaque blob rules).
2. Build session-derived stack: `entries[0..=cursor]` as `{page, data}` — **full visited stack for stdout** (includes current).
3. **`button: finish`:** `WizardResult.stack` = session-derived stack. If client supplies `stack`, it must equal session-derived stack → else `WizardError::StackMismatch` → HTTP 400. Host may omit client validation and always reconstruct (recommended).
4. **`button: cancel`:** `WizardResult.stack` = `[]` always; client `stack` ignored.
5. **`button: dismissed`:** same as finish for stack reconstruction (full visited stack); `data` typically `{}`.

```rust
pub fn finish(&self, button: ButtonLabel, data: Value, stack: Vec<WizardStackEntry>)
    -> Result<WizardResult, WizardError>;
```

### Host (`wyvern-host`)

- `POST /api/wizard/navigate` → `navigate_next` / `navigate_back`
- `POST /api/wizard/finish` → `finish`; stdout = body
- `tests/wizard_navigate.rs`, `tests/wizard_finish.rs` (include finish-stack validation + cursor=0 back → 400)

### UI (`ui/shared/wyvern-api.js`)

- `wyvernWizardState`, `wyvernWizardNext`, `wyvernWizardBack`, `wyvernWizardFinish`
- **Production bootstrap:** on load, `GET /api/wizard/state` sets `window.wyvern.{config,page,page_data,stack}` (d.4 adds round-trip tests only — no new bootstrap logic)

## Acceptance criteria

1. `navigate_next` / `navigate_back` / `finish` work over HTTP
2. `cancel` only via `/finish`; `navigate` + `cancel` → 400
3. Back keeps forward entries (ADR-0005)
4. Branch forward truncates stale entries
5. `navigate_back` at cursor=0 → HTTP 400 (`AtFirstPage`)
6. Finish validates client `stack` against session entries; mismatch → 400
7. Prior dialogs + d.1 regression pass

## Required validation

```bash
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_navigate wizard_finish
```

## Non-closure

- Four-case history test matrix formalized (d.3)
- Bootstrap round-trip tests (d.4)
- Examples (d.5), polish/sizing (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), ADR-0005, ADR-0007, REQ-0020–0025
