---
id: d.4
title: Page bootstrap + stack snapshot tests
status: planning
branch: feature/phase-D-d4-stack-inject
target: integrate/phase-D
---

# Sprint d.4 — Page bootstrap + stack snapshot tests

## Goal

Pages read `window.wyvern` from `snapshot()` on load. **Verify** `stack` / `page_data` round-trip — no new stack logic (already in d.2 `snapshot()`).

## Hard dependencies

- **d.3** merged

## Snapshot fields (reference)

| Field | From stack |
|-------|------------|
| `page` | `entries[cursor].page` |
| `page_data` | `entries[cursor].data` |
| `stack` | `entries[0..=cursor]` as `{page, data}` |

## Deliverables

| File | Change |
|------|--------|
| `ui/shared/wyvern-api.js` | Document + ensure bootstrap sets `window.wyvern.{config,page,page_data,stack}` |
| `crates/wyvern-wizard/tests/stack_restore.rs` | Round-trip / restore tests |
| `crates/wyvern-host/tests/wizard_stack.rs` | HTTP multi-step + state GET |

## Acceptance criteria

1. `page_data` restored after back/forward
2. `stack` includes prior steps after `next`
3. JSON round-trip loses no opaque keys
4. REQ-0024 satisfied via HTTP (not IPC)

## Required validation

```bash
cargo test -p wyvern-wizard stack_restore
cargo test -p wyvern-host wizard_stack
```

## Non-closure

- Examples (d.5), polish/sizing (d.6)

## Authority

- REQ-0024, NFR-0008
