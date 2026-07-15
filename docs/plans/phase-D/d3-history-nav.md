---
id: d.3
title: Browser-history regression tests
status: complete
branch: feature/phase-D-d3-history-nav
target: integrate/phase-D
---

# Sprint d.3 — Browser-history regression tests

## Goal

Lock ADR-0005 behaviour with the five canonical tests. **No new routes, traits, or state types** — stack shipped in d.2.

## Hard dependencies

- **d.2** merged (full `WizardSession` history behaviour — private `history.rs` module, ADR-0007)

## Deliverables

| File | Change |
|------|--------|
| `crates/wyvern-wizard/tests/history_five_cases.rs` | Five named tests (below) |
| `crates/wyvern-host/tests/wizard_history.rs` | HTTP navigate + `GET /api/wizard/state` asserts |

### Five tests (authoritative)

| Test | Proves |
|------|--------|
| `forward_push_advances_cursor` | A→B→C, cursor follows |
| `back_moves_cursor_without_truncation` | Back does not delete forward entries |
| `forward_same_page_restores_data` | Same `next` descriptor restores cached `data`; overwrite only when request `data` is a meaningful payload per d.2 overwrite predicate (`null`/`{}`/`[]`/`""` → restore) |
| `forward_different_page_truncates` | New branch drops stale forward entries |
| `forward_same_html_different_id_truncates` | Same `html`, different `id` → truncate (not restore) |

## Acceptance criteria

1. All five unit tests pass
2. `wizard_history` host test passes
3. No new public API on `wyvern-wizard`

## Required validation

```bash
cargo test -p wyvern-wizard history_five_cases
cargo test -p wyvern-host wizard_history
```

## Non-closure

- Bootstrap round-trip tests (d.4), examples (d.5), viewport (d.6+)

## Authority

- ADR-0005, ADR-0007, REQ-0020–0023
