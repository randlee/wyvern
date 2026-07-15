---
id: d.4
title: Stack injection and data restoration
status: planning
branch: feature/phase-D-d4-stack-inject
target: integrate/phase-D
---

# Sprint d.4 — Stack injection and data restoration

## Goal

Ensure full stack and per-page data round-trip through navigation and `GET /api/wizard/state` per REQ-0024.

## Hard dependencies

- **d.3** merged

## Deliverables

### Semantics (authoritative)

| Field | Meaning |
|-------|---------|
| `stack` | All **completed** prior pages as `{ "page": descriptor, "data": opaque }` **plus** the current page's entry when it has collected data |
| `page_data` | Opaque data for the **current** page at the history cursor — restored on back/forward, empty `{}` on first visit |
| `page` | Current page descriptor at cursor |

**Restoration rule:** When cursor moves (back or forward-restore), `snapshot().page_data` must equal the cached `data` for that page id at that cursor position. Host does not compute this — `WizardEngine::snapshot()` reads from private history.

### `wyvern-wizard`

| File | Change |
|------|--------|
| `crates/wyvern-wizard/src/engine.rs` | `snapshot()` populates `stack` + `page_data` from `BrowserHistory` |
| `crates/wyvern-wizard/tests/stack_restore.rs` | **new** |

**Tests (`stack_restore.rs`):**

- `stack_includes_prior_pages_after_next`
- `page_data_empty_on_first_visit`
- `page_data_restored_after_back_forward`
- `stack_serializes_round_trip_json` — `serde_json` round-trip without data loss

### `wyvern-host`

| File | Change |
|------|--------|
| `crates/wyvern-host/tests/wizard_stack.rs` | **new** — multi-step HTTP navigate + state GET assertions |

### UI (`ui/shared/wyvern-api.js`)

- Bootstrap sets `window.wyvern.stack` from `GET /api/wizard/state`
- Document in file header: pages read `window.wyvern.stack` for DAG context; host never interprets stack contents (NFR-0008)

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `GET /api/wizard/state` returns `stack` with all prior `{page, data}` entries after multi-step flow
3. `page_data` populated with current page's previously collected data on restore
4. Page JS reads `stack` from wizard state bootstrap (via `wyvern-api.js`)
5. Data round-trips correctly through JSON serialization (no key loss for arbitrary JSON objects)
6. REQ-0024 satisfied on HTTP path (replaces IPC injection wording in requirements doc)

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
- REQ-0024, NFR-0008
