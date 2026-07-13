# Phase C — Error handling follow-up (reference)

Branch: `planning/phase-c-error-handling`  
Base: `develop` @ post–Phase C merge (`41e3e24`)

> **Authority:** [c6-result-propagation.md](c6-result-propagation.md), [c7-cli-test-hardening.md](c7-cli-test-hardening.md), and [c8-clippy-deny-unwrap.md](c8-clippy-deny-unwrap.md) are the **sole authority** for deliverables, acceptance criteria, and validation. This doc and [UNWRAP-INVENTORY.md](UNWRAP-INVENTORY.md) are **reference/audit only**.

## Architectural policy (decided)

**Panics are forbidden in production paths** (including `main.rs`). Test code may panic.

**All layers return discriminated unions** (`Result<T, E>` where `E` is a crate-appropriate error enum). Errors propagate upward; the CLI boundary maps each variant to structured stderr JSON and a non-zero exit code. No `unwrap()`, `expect()`, `panic!()`, or `unreachable!()` in non-test production code.

### Layer model (existing + extensions)

| Layer | Crate | Success | Error enum | Maps to stderr / exit |
|-------|-------|---------|------------|------------------------|
| Load | `wyvern` | `serde_json::Value` | `LoadError` (`Parse` \| `Io` \| `Usage`) | `parse` / `io` (JSON); **Usage → plain text, exit 1** (no JSON slug) |
| Validate | `wyvern-schema` | `Command` | `ValidationError` (`Validation` \| `State`) | `validation` / `state` |
| Run | `wyvern-window` | `CommandResult` | `RunError` (`WindowCreate` \| `EventLoop`) | `window_create` / `event_loop` |
| Emit | `wyvern` | stdout JSON string | `EmitError` (`Serialize`) | `internal` (exit 8) |

**Phase C gap:** `media.rs` and emit helpers bypass this model with `expect`. c.6 closes that gap.

### Extraordinary circumstances

**Production `unreachable!` is not permitted** (c.6/c.8). Miswired internal paths map to `EmitError` → `internal` exit 8 or `debug_assert!` in debug builds only.

Allowed panics: **`#[cfg(test)]` and integration test harnesses only.**

### Serialization

`CommandResult` and `StderrError` are protocol types. If serde can fail (it should not for these types), return `Result<String, EmitError>` from emit helpers rather than `expect`. Callers at `main` map `EmitError` to stderr + exit code.

---

## Problem statement

Phase C landed production code with `expect`/`panic!` in hot paths and CLI integration tests that crash under parallel `cargo test` on macOS (winit/objc races when multiple webview processes spawn concurrently).

## Panic inventory (from logs)

### Runtime (spawned `wyvern` child — not test assertions)

| Location | Message | Trigger |
|----------|---------|---------|
| `winit` `macos/view.rs:907` | uninitialized instance variable | Parallel CLI GUI tests |
| `objc2` `weak_id.rs:116` | misaligned pointer dereference | Parallel CLI GUI tests (input) |
| `core::panicking` | panic in a function that cannot unwind | After winit/objc abort |

**Affected tests** (`crates/wyvern/tests/cli_validation.rs`): lines 31, 77, 111, 119, 128, 201.

**Mitigation in CI:** `--test-threads=1`. c.7 hardens local/CI policy.

### Production `src` violations (must fix)

Full audit with per-site justification: **[UNWRAP-INVENTORY.md](UNWRAP-INVENTORY.md)**.

| File | Lines | Fix |
|------|-------|-----|
| `wyvern-window/src/message/media.rs` | 21–22, 112–114 | `Result<_, RunError::WindowCreate>` — icon/embed defense-in-depth |
| `wyvern/src/error.rs` | 60 | Remove `unreachable!` — split `emit_load_error` or narrow input type |
| `wyvern/src/error.rs` | 183 | `emit_stdout` → `Result<String, EmitError>` |
| `wyvern-schema/src/stderr.rs` | 95 | `to_json_string` → `Result<String, _>`; all emit helpers propagate |

`run.rs` has no direct `unwrap()`/`expect()` (good pattern to preserve). `unwrap_or*` in production is acceptable (not panics).

---

## Proposed sprint split

```
develop ──► integrate/phase-C-fixes ──► c.6 ──► c.7 ──► c.8
```

| Sprint | Branch | Focus |
|--------|--------|-------|
| **c.6** | `feature/phase-C-c6-result-propagation` | `Result` propagation; `WindowCreate` for icon misses; `InternalError` for emit; ADR-0013 amendment |
| **c.7** | `feature/phase-C-c7-cli-test-hardening` | `serial_test` on nine GUI CLI tests; shared `run_wyvern` helper |
| **c.8** | `feature/phase-C-c8-clippy-deny-unwrap` | Clippy deny on three lib roots; `docs/linting.md` panic policy |

c.6 and c.8 are **separate sprints** — c.8 owns clippy deny only.

## Enforcement — sc-lint vs Clippy (audited 2026-07-13)

**sc-lint 0.4.x has no dedicated “unauthorized panic” rule.** Wyvern’s [`.sc-lint.toml`](../../../../.sc-lint.toml) only sets `[workspace].root`; the schema also supports `[logging]` but not panic/unwrap policy knobs.

| Surface | Detects production `unwrap`/`expect`/`panic!`? | Wyvern CI today |
|---------|-----------------------------------------------|-----------------|
| `sc-lint check native` | **No** — wraps `cargo check --workspace` | Yes |
| `sc-lint clippy native` | **Indirect** — wraps `cargo clippy -D warnings`; honors crate `#![deny(...)]` | No (direct `cargo clippy` instead) |
| `sc-lint lint sc-boundary` | **No** — dependency/ownership graph (`boundaries/*.toml`) | Stub only |
| `sc-lint lint sc-runtime` | **No** — `SCB-RUNTIME-001/002` condvar liveness only | Not run |

**Authoritative regression gate for panics:** crate-root Clippy denies (c.8) + existing `cargo clippy --workspace -- -D warnings` in [`.github/workflows/ci.yml`](../../../../.github/workflows/ci.yml).

```rust
// crates/*/src/lib.rs — c.8 deliverable
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]
```

c.8 also updates [`docs/linting.md`](../../../../docs/linting.md) to document that panic policy is enforced via Clippy denies (not a `.sc-lint.toml` key). Optional alignment: add `sc-lint clippy native --config .sc-lint.toml` as a documented alias for the same clippy gate.

**Out of scope:** a future `sc-lint` source-scan rule (e.g. `SCB-RUNTIME-003` panic-in-production) — track only if sc-lint ships it; do not block c.6–c.8 on new sc-lint features.

## Resolved decisions

| # | Decision |
|---|----------|
| Error model | **Discriminated unions through all layers** — no `expect` on recoverable paths |
| Serialize | **`Result` from emit helpers** — map at CLI boundary |
| Integrate branch | **`integrate/phase-C-fixes`** off `develop` (Phase C already merged) |
| Panic regression gate | **Clippy `deny` on three lib roots** (c.8) — not sc-lint.toml |
| Icon/embed failures | **`RunError::WindowCreate`** — no new REQ-0073 slug |
| Emit serialize failures | **`ErrorCode::InternalError`** — slug `internal`, exit `8` |
| Sprint split | **c.6 / c.7 / c.8 separate** — no merge |

## Hardened sprint docs

| Sprint | Doc |
|--------|-----|
| c.6 | [c6-result-propagation.md](c6-result-propagation.md) |
| c.7 | [c7-cli-test-hardening.md](c7-cli-test-hardening.md) |
| c.8 | [c8-clippy-deny-unwrap.md](c8-clippy-deny-unwrap.md) |

## Next steps

1. ~~Harden c.6–c.8 sprint docs~~ (done)
2. ~~Plan-scope + critical review round 1~~ (done — fixes applied)
3. Plan-scope + critical review round 2 (models swapped)
4. Create `integrate/phase-C-fixes` + worktrees; run `/cursor-orchestration`
