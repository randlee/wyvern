---
id: d.4
title: Page bootstrap + stack snapshot tests
status: planning
branch: feature/phase-D-d4-stack-inject
target: integrate/phase-D
---

# Sprint d.4 — Page bootstrap + stack snapshot tests

## Goal

Verify `window.wyvern` bootstrap and `stack` / `page_data` round-trip via tests — **no new stack logic or JS production changes** (bootstrap shipped in d.2).

## Hard dependencies

- **d.3** merged

## Snapshot fields (reference — normative, REQ-0024)

| Field | From stack |
|-------|------------|
| `page` | `entries[cursor].page` |
| `page_data` | `entries[cursor].data` |
| `stack` | `entries[0..cursor]` as `{page, data}` — **prior entries only**, excludes current |

## Deliverables

| File | Change |
|------|--------|
| `crates/wyvern-wizard/tests/stack_restore.rs` | Round-trip / restore tests |
| `crates/wyvern-host/tests/wizard_stack.rs` | HTTP multi-step + state GET asserts bootstrap fields |

**No JS production changes in d.4.** Assert `GET /api/wizard/state` returns `config`, `page`, `page_data`, and prior-only `stack` per d.2 bootstrap contract via host/wizard integration tests.

## Acceptance criteria

1. `page_data` restored after back/forward
2. `stack` contains prior steps only after `navigate_next` (current page via `page` + `page_data`, not in `stack`)
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

- REQ-0024, NFR-0008, ADR-0007
