---
id: c.9
title: Delete wyvern-window stack (compile optional)
status: planning
branch: feature/phase-C-c9-deletion
target: integrate/phase-C
---

# Sprint c.9 — Delete `wyvern-window` (compile optional)

## Principle (locked)

**Delete → verify → rebuild.** c.9 is demolition only. Rebuild starts in c.10.

## Goal

Remove the entire embedded delivery stack. No `wyvern-host`, no new UI, no compile gate.

## Hard dependencies

- c.6–c.8 merged to `develop`
- [c9-deletion-and-rework.md](c9-deletion-and-rework.md) — authoritative inventory (§c deletion-only; §d rework pointers are non-normative)

## Deliverables

- Delete entire `crates/wyvern-window/` per inventory (§c)
- Delete `wyvern-schema/src/icons.rs` + catalog validation/tests
- Rework `wyvern-schema` icon handling: drop `icons` module from `lib.rs`; remove `is_named_icon_spec` / `validate_named_icon` from `validate/helpers.rs`; `validate/message.rs` + `validate/input.rs` accept opaque `icon`/`image` strings only; drop catalog cases from `validate/tests.rs` + `tests/validation_*.rs`
- Remove `wyvern-window` from workspace `Cargo.toml` `members`
- Remove nine GUI `#[serial]` CLI tests from `crates/wyvern/tests/cli_validation.rs` (inventory table)
- Remove `serial_test` dev-dependency from `crates/wyvern/Cargo.toml` when no serial tests remain
- Delete `boundaries/wyvern-window/` directory
- Add `scripts/verify-c9-deletion.sh` — automated inventory gate for QA

## Acceptance criteria

1. Zero files under `crates/wyvern-window/`
2. `icons.rs` absent; named-icon catalog tests removed
3. `wyvern-schema` rework: no `mod icons` / `NamedIconSpec` re-export; no `is_named_icon_spec` in helpers
4. Nine GUI serial tests absent from `cli_validation.rs` (see inventory)
5. `serial_test` absent from `crates/wyvern/Cargo.toml` `[dev-dependencies]`
6. `wyvern-window` absent from workspace `members`
7. `boundaries/wyvern-window/` absent
8. **`cargo build --workspace` not required** — may fail until c.10

## Required validation

```bash
./scripts/verify-c9-deletion.sh          # exit 0 — gates ALL deliverables above
test ! -d crates/wyvern-window
test ! -f crates/wyvern-schema/src/icons.rs
! rg -l 'wyvern-window' Cargo.toml boundaries/ 2>/dev/null | grep -v verify-c9-deletion
```

`sc-lint` / `cargo test --workspace` — **not** merge gates for c.9.

## Non-closure

- `wyvern-host`, `ui/`, HTTP tests (c.10+)
- Any dialog type on host (c.10–c.14)
- Viewer, release (c.15–c.16)

## Authority

- [c9-deletion-and-rework.md](c9-deletion-and-rework.md) — §c authoritative for deletion scope
- ADR-0018
