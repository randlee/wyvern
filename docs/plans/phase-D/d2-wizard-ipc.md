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

- **`WizardNavigateRequest`**, **`WizardFinishRequest`** — wire DTOs in `wyvern-schema` (see [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md))

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

**Page descriptor equality (normative):** forward-same-page restore compares full `WizardPageDescriptor` via `PartialEq` (`id`, `title`, `html`, `layout`). Same `html` but different `id` → truncate branch, not restore.

**Opaque data write rule (normative — all navigate + finish paths):**

Whole-blob replace only — no deep merge. `entries[cursor].data = data` replaces the entire stored blob. Host and wizard never interpret keys inside `data` (ADR-0006).

**Forward-same-page overwrite predicate (normative):**

When `navigate_next` restores a cached forward entry (same `next` descriptor), overwrite cached `data` only when request `data` is a **meaningful payload**:

| Request `data` | Overwrite cached entry? |
|----------------|-------------------------|
| `null` | No — restore cached |
| `{}` (empty object) | No — restore cached |
| `[]` (empty array) | No — restore cached |
| `""` (empty string) | No — restore cached |
| Any other value | Yes — replace cached blob |

**`navigate_next` data write (normative):** apply opaque write rule to `entries[cursor]` **before** push/truncate-forward/advance. Forward-same-page restore uses overwrite predicate above.

**`navigate_back` data write (normative):** apply meaningful-payload overwrite predicate (same table as forward-same-page) to `entries[cursor]` **before** `cursor--`. `null`/`{}`/`[]`/`""` → skip write, cursor-- only. Restored `page_data` comes from destination entry.

**`navigate_back` at cursor=0:** returns `WizardError::AtFirstPage` → host maps to HTTP **400** (no silent no-op).

**Finish stack algorithm (normative — `finish` is `&self`; session is discarded after stdout):**

`finish` does **not** mutate the session. It derives stdout in memory from current `entries` + request fields:

1. Derive current entry data: `current_data = request.data` (opaque whole-blob replace of the in-memory current entry for stack derivation only).
2. Build session-derived stack: `entries[0..cursor]` as `{page, data}` plus `{ page: entries[cursor].page, data: current_data }` — **full visited stack for stdout** (includes current).
3. **`button: finish`:** `WizardResult.stack` = session-derived stack; `WizardResult.data` = `request.data`. If client supplies `stack`, it must equal session-derived stack → else `WizardError::StackMismatch` → HTTP 400.
4. **`button: cancel`:** `WizardResult.stack` = `[]` always; `WizardResult.data` = `{}`; client `stack` ignored.
5. **`button: dismissed`:** same stack reconstruction as `finish`; `WizardResult.data` = `{}`.

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

**Wizard helper contract (normative):**

| Helper | POST body | Post-navigate behavior |
|--------|-----------|------------------------|
| `wyvernWizardNext(data, next)` | `{ action: "next", data, next }` | On `{ ok, url }`: `window.location = url` (full reload); bootstrap re-runs on new page |
| `wyvernWizardBack(data?)` | `{ action: "back", data: data ?? collectCurrentPageData() }` | Same reload pattern; `data` uses meaningful-payload predicate (omit or pass `{}` to preserve current entry) |
| `wyvernWizardFinish({ button, data, stack })` | `{ button, data, stack }` where `stack` = `window.wyvern.stack` + `{ page: window.wyvern.page, data }` (full visited stack per finish algorithm) | Host stdout; session ends |

`collectCurrentPageData()` is page-author logic — returns opaque blob for current form state. Helpers never interpret keys inside `data`.

## Acceptance criteria

1. `navigate_next` / `navigate_back` / `finish` work over HTTP
2. `cancel` only via `/finish`; `navigate` + `cancel` → 400
3. Back keeps forward entries (ADR-0005)
4. Branch forward truncates stale entries
5. `navigate_back` at cursor=0 → HTTP 400 (`AtFirstPage`)
6. Finish validates client `stack` against session entries; mismatch → 400
7. `finish`/`dismissed` stdout `stack` = full visited stack (`entries[0..=cursor]` including current); `cancel` stdout `stack: []`, `data: {}`
8. `finish` stdout `data` = request `data`; `dismissed` stdout `data` = `{}`
9. `navigate_back` with `{}` preserves current entry data (meaningful-payload predicate)
10. Prior dialogs + d.1 regression pass

## Required validation

```bash
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_navigate wizard_finish
```

## Non-closure

- Four-case history test matrix formalized (d.3)
- Bootstrap round-trip tests (d.4)
- Examples (d.5), viewport sizing (d.6), chrome (d.7), dismiss (d.8)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md), ADR-0005, ADR-0007, REQ-0020–0025
