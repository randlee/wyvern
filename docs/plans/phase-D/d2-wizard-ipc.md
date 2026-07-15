---
id: d.2
title: Wizard HTTP navigation + finish + browser stack
status: planning
branch: feature/phase-D-d2-wizard-ipc
target: integrate/phase-D
---

# Sprint d.2 — Wizard HTTP navigation + finish + browser stack

## Goal

Complete the stack: `next`, `back`, `finish` on `WizardSession` plus HTTP routes and page JS helpers. **Ship full ADR-0005 browser history here** — not a stub for d.3 to replace.

## Hard dependencies

- **d.1** merged

## Deliverables

### Stack (`wyvern-wizard`)

Extend `WizardSession` (same type as d.1):

```rust
impl WizardSession {
    pub fn navigate_next(&mut self, data: Value, next: WizardPageDescriptor) -> Result<NavigateOutcome, WizardError>;
    pub fn navigate_back(&mut self, data: Value) -> Result<NavigateOutcome, WizardError>;
    pub fn finish(&self, button: ButtonLabel, data: Value, stack: Vec<WizardStackEntry>) -> WizardResult;
}
```

`history.rs` implements ADR-0005 (cursor, truncate on branch, restore on forward-same-page). `snapshot()` derives `page`, `page_data`, `stack` from `entries` + `cursor` — **d.4 only adds tests for this, not new logic**.

### Host (`wyvern-host`)

- `POST /api/wizard/navigate` → `navigate_next` / `navigate_back`
- `POST /api/wizard/finish` → `finish`; stdout = body
- `tests/wizard_navigate.rs`, `tests/wizard_finish.rs`

### UI (`ui/shared/wyvern-api.js`)

- `wyvernWizardState`, `wyvernWizardNext`, `wyvernWizardBack`, `wyvernWizardFinish`
- Bootstrap `window.wyvern` from `GET /api/wizard/state` (expanded in d.4)

## Acceptance criteria

1. `next` / `back` / `finish` work over HTTP
2. `cancel` only via `/finish`; `navigate` + `cancel` → 400
3. Back keeps forward entries (ADR-0005)
4. Branch forward truncates stale entries
5. Prior dialogs + d.1 regression pass

## Required validation

```bash
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_navigate wizard_finish
```

## Non-closure

- Four-case history test matrix formalized (d.3)
- `window.wyvern.stack` bootstrap docs (d.4)
- Examples (d.5), polish/sizing (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), ADR-0005, REQ-0020–0025
