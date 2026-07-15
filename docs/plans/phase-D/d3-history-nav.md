---
id: d.3
title: Browser-history navigation model
status: planning
branch: feature/phase-D-d3-history-nav
target: integrate/phase-D
---

# Sprint d.3 — Browser-history navigation model

## Goal

Implement ADR-0005 cursor-over-array history inside **private** `wyvern-wizard` modules; host continues to call only `WizardNavigator` trait methods.

## Hard dependencies

- **d.2** merged

## Deliverables

### Private implementation (`wyvern-wizard`)

| File | Change |
|------|--------|
| `crates/wyvern-wizard/src/browser_history.rs` | Full ADR-0005 implementation (replace d.2 stub) |
| `crates/wyvern-wizard/src/engine.rs` | Delegate `navigate_*` to `BrowserHistory` |
| `crates/wyvern-wizard/tests/history_four_cases.rs` | **new** — four canonical cases below |

**Internal type (not exported from `lib.rs`):**

```rust
struct BrowserHistory {
    entries: Vec<HistoryEntry>, // { page, data, next_descriptor }
    cursor: usize,
}
```

Host and tests outside `wyvern-wizard` must **not** depend on this struct — only `WizardNavigator` outcomes.

### Four canonical test cases (names are authoritative)

| Test name | Sequence | Expected cursor / stack |
|-----------|----------|-------------------------|
| `forward_push_advances_cursor` | A→B→C | `[A,B,C]`, cursor=2 after each forward |
| `back_moves_cursor_without_truncation` | A→B→C, back, back | `[A,B,C]`, cursor=0; forward entries retained |
| `forward_same_page_restores_data` | A→B→C, back×2, forward to B (same descriptor) | B's `data` restored from cache; cursor=1 |
| `forward_different_page_truncates` | A→B→C, back to A, forward to D (new descriptor) | `[A,D]`, cursor=1; B,C truncated |

### Host (`wyvern-host`)

- `routes/wizard.rs` — no signature changes; behaviour improves via trait impl swap
- `tests/wizard_history.rs` — **new** — HTTP-level tests proving back/forward data retention through `/api/wizard/state` after navigate calls

### ADR alignment

Amend [docs/wyvern-wizard/architecture.md](../../wyvern-wizard/architecture.md) ADR-0005 consequences:

- **wyvern-wizard** owns history **logic** (cursor, truncate, restore)
- **wyvern-host** owns session **storage** of `Box<dyn WizardNavigator>` and HTTP serialization
- Pages own domain branching via opaque `data` + explicit `next` descriptors (ADR-0006)

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. All four unit tests in `history_four_cases.rs` pass
3. `cargo test -p wyvern-host wizard_history` passes
4. `browser_history` module is **private** — `pub use` list in `lib.rs` contains only trait types + errors
5. Forward navigation pushes page + data, advances cursor
6. Back moves cursor back without truncating forward history
7. Forward on same next-page restores cached page data
8. Forward on different next-page truncates forward history and pushes new page

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-wizard history_four_cases
cargo test -p wyvern-wizard
cargo test -p wyvern-host wizard_history
```

## Non-closure

- Stack injection polish (d.4), DAG example (d.5), polish (d.6)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md) — `POST /api/wizard/navigate`
- ADR-0005, ADR-0006, REQ-0020–REQ-0023
