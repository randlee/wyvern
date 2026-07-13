# Unwrap / expect / panic inventory (`develop` @ `41e3e24`)

Audit date: 2026-07-13  
Policy: **discriminated unions through all layers**; panics only in `#[cfg(test)]` or documented extraordinary cases.

Legend:
- **REMOVE** → replace with `Result<T, E>` / error enum variant
- **ALLOW (test)** → permitted in tests / test modules
- **ALLOW (non-panic)** → `unwrap_or` / `unwrap_or_else` / `unwrap_or_default` — not `unwrap()`
- **REFACTOR** → restructure types so `unreachable!` is unnecessary

---

## Summary

| Category | `unwrap()` | `expect()` | `panic!` | `unreachable!` |
|----------|------------|------------|----------|----------------|
| Production `src/` | **0** | **4** | **0** | **1** |
| `src/` in `#[cfg(test)]` | many | many | many | 0 |
| `tests/` integration | ~120+ | ~80+ | ~40+ | 0 |

**Production violations (must fix in c.6):** 4 `expect` + 1 `unreachable!` (see §1).

---

## §1 Production `src/` — `expect` / `unreachable!` (must fix)

| File | Line | Code | Verdict | Replacement |
|------|------|------|---------|---------------|
| `wyvern-window/src/message/media.rs` | 22 | `icons::svg_markup(...).expect(...)` | **REMOVE** | `RunError::WindowCreate` via `?` |
| `wyvern-window/src/message/media.rs` | 113–114 | `parse_icon_spec(...).expect(...)` + `svg_markup(...).expect(...)` | **REMOVE** | `Result<&'static str, RunError>` → `WindowCreate` |
| `wyvern/src/error.rs` | 183 | `serde_json::to_string(result).expect(...)` | **REMOVE** | `emit_stdout` → `Result<String, EmitError>` |
| `wyvern-schema/src/stderr.rs` | 95 | `serde_json::to_string(self).expect(...)` | **REMOVE** | `to_json_string` → `Result<String, SerializeError>` |
| `wyvern/src/error.rs` | 60 | `unreachable!("Usage handled in main")` | **REFACTOR** | `emit_parse_error` / `emit_io_error`; Usage only in `main` |

**Authoritative closure checklist:** c.6 Deliverables §1 table (this section is audit trail only).

### Production `src/` — safe non-panic APIs (no change)

| File | Pattern | Notes |
|------|---------|-------|
| `wyvern/src/main.rs` | `unwrap_or(1)` on exit code | Fallback bound, not panic |
| `wyvern-window/src/window.rs` | `unwrap_or_else` for `DISPLAY` | Env default for diagnostics |
| `wyvern-window/src/run.rs` | `unwrap_or_else` / `unwrap_or_default` | Option defaults for titles, filters, outcomes |
| `wyvern-window/src/*/render.rs` | `unwrap_or_default` | Empty string defaults |
| `wyvern-window/src/input/picker.rs` | `unwrap_or` on strip_prefix | String slice logic, always returns &str |
| `wyvern-schema/src/validate.rs` | `unwrap_or(false)` | Optional JSON bool defaults |
| `wyvern-schema/src/command.rs` | `unwrap_or(&[])` | Empty custom_buttons default |

`run.rs`, `validate.rs` (lib), `icons.rs`, `pipeline.rs` (lib), `input.rs` (lib) have **zero** production `unwrap()`/`expect()`/`panic!`.

---

## §2 `#[cfg(test)]` modules inside `src/` — ALLOW (test)

These are inside `mod tests { }` or `#[cfg(test)]` fns. **Allowed** per policy; c.8 may still use `clippy::allow` in test modules only.

| File | Count (approx) | Notes |
|------|----------------|-------|
| `wyvern-window/src/message/media.rs` | 12 `expect` | Render resolution unit tests |
| `wyvern-window/src/message/render.rs` | 9 `expect`, 1 `unwrap` | HTML fixture tests |
| `wyvern-window/src/input/render.rs` | 10 `expect`, 5 `unwrap`, 2 `panic!` | Render + IPC parse tests |
| `wyvern-window/src/question/render.rs` | 1 `expect`, 4 `unwrap`, 2 `panic!` | Preview + IPC tests |
| `wyvern-window/src/markdown/render.rs` | 1 `unwrap` | IPC parse test |
| `wyvern-window/src/chrome/render.rs` | 2 `expect` | Title bar fixture tests |
| `wyvern-window/src/icons/mod.rs` | 2 `expect` | Embed catalog tests |
| `wyvern-window/src/window.rs` | 1 `expect`, 1 `panic!` | `logical_inner` test helper only |
| `wyvern-schema/src/validate.rs` | 15 `expect`, 14 `panic!` | Validation unit tests |
| `wyvern-schema/src/result.rs` | 7 `expect`, 2 `unwrap` | Serde round-trip tests |
| `wyvern-schema/src/stderr.rs` | 1 `expect` | JSON round-trip test |
| `wyvern-schema/src/error_code.rs` | 2 `expect` | Serde test |
| `wyvern-schema/src/button.rs` | 1 `expect` | Serde test |
| `wyvern/src/error.rs` | 1 `expect`, 15 `unwrap` | Recovery JSON tests |
| `wyvern/src/input.rs` | 8 `expect`, 4 `unwrap`, 1 `panic!` | Load path tests |
| `wyvern/src/pipeline.rs` | 4 `expect`, 3 `unwrap`, 3 `panic!` | Markdown load tests |
| `wyvern/src/observability.rs` | 4 `expect` | Observability config tests |

**No justification required** for test-only panics beyond "test harness."

---

## §3 `tests/` integration crates — ALLOW (test) + c.7 hardening

| Crate | Files | Primary patterns | c.7 action |
|-------|-------|------------------|------------|
| `wyvern` | `cli_validation.rs` | Nine GUI tests (see c.7 exhaustive list) | `#[serial]` + `run_wyvern` helper |
| `wyvern-window` | 15 `*_ipc.rs`, `support.rs`, `blank_window.rs` | `expect` on `run_with_test_inject` completion | Keep; ensure `--test-threads=1` on macOS |
| `wyvern-schema` | `validation_*.rs`, `question_contract_examples.rs` | `expect("valid")`, `panic!` on wrong variant | Standard test style — no change |

---

## §4 Extraordinary circumstances — none currently justified in production

Per policy, production `expect` claiming "impossible" is **not** extraordinary unless enforced by:

1. **Types** — e.g. `ValidatedIconSpec` newtype only constructible after `validate()`
2. **Compile-time** — `include_bytes!` with const-checked layout
3. **Exhaustive enum** — no `unreachable!` needed

Current `"schema validated"` and `"CommandResult serializes"` claims fail (1) and serde (2).

### Optional future hardening (not c.6 — closure is `RunError::WindowCreate` in c.6 §1)

```rust
// After validate() — carry proof into window layer
pub struct ValidatedCommand(Command); // or per-field newtypes for icon specs

// media.rs — no expect; only accepts validated specs or returns RunError
pub fn resolve_named_icon_svg(spec: &ValidatedIconSpec) -> Result<&'static str, RunError>;
```

---

## §5 Sprint mapping

| Inventory § | Sprint |
|-------------|--------|
| §1 production violations | **c.6** — result propagation |
| §3 CLI child panics (winit/objc) | **c.7** — test harness |
| §2 + §3 test `unwrap`/`expect` | **c.8** — Clippy `deny` on lib roots; tests exempt |

## §6 Regression enforcement (sc-lint vs Clippy)

**sc-lint 0.4.x does not expose panic/unwrap detection in `.sc-lint.toml`.** Wyvern uses:

1. **c.8** — `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable, ...)]` on `wyvern`, `wyvern-schema`, `wyvern-window` `lib.rs`
2. **CI** — existing `cargo clippy --workspace -- -D warnings` (already in `.github/workflows/ci.yml`)
3. **`sc-lint check native`** — compile gate only; does **not** replace (2)

Optional doc alignment: `sc-lint clippy native --config .sc-lint.toml` as wrapper equivalent to (2). See [ERROR-HANDLING-PLAN.md](ERROR-HANDLING-PLAN.md) enforcement table.
