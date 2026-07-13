---
id: c.4
title: Cross-platform validation and NFR pass
status: pending
branch: feature/phase-C-c4-cross-platform-validation
target: integrate/phase-C
---

# Sprint c.4 — Cross-platform validation and NFR pass

## Goal

- Verify macOS NFR targets (NFR-0001–NFR-0003) after icon bundle + platform chrome land.
- Fix cross-platform rendering regressions; confirm all Phase B acceptance criteria on ubuntu, macos, and windows CI.

## Hard Dependencies

- c.1 icon asset bundle (binary size impact)
- c.2 icon resolution complete
- c.3 Win/Linux platform chrome

## Exact Targets

- `crates/wyvern-window/tests/` — cross-platform render/regression tests as needed
- `docs/requirements.md` — confirm NFR wording still valid (no edit unless drift found)
- `.github/workflows/ci.yml` — optional `nfr-smoke` job (macOS-only, non-blocking) if scripted checks land
- Any dialog template/CSS fixes discovered during Win/Linux visual regression

## Deliverables

- NFR-0001: window opens **< 500ms** on macOS (measured once; documented in sprint PR or README note)
- NFR-0002: resident memory **< 80MB** on macOS under normal single-dialog operation
- NFR-0003: release binary **< 10MB** on macOS (`target/release/wyvern`); if over, document mitigation (asset compression, WebP) in PR — must not ship v0.1.0 over limit without explicit acceptance
- No rendering regressions on Windows or Linux CI legs
- All Phase B README smoke scenarios pass via automated tests where possible; macOS manual spot-check for icon + chrome combo
- Auto-size bounds unchanged: dialog **min 320×200**, **max 800×600**; chrome **480×360** default open

## Required Work — NFR verification (authoritative)

### NFR-0001 (latency)

Measure on macOS release build:
1. Cold start `wyvern '{"type":"message",...}'` — time from process start to IPC inject hook or test harness ready signal
2. Target: p95 < 500ms over 5 runs

If over target: profile wry init / asset embed size; optimize before c.5 tag.

### NFR-0002 (memory)

With one message dialog open on macOS: resident size < 80MB. WebKit baseline dominates — icon bundle should not materially change this vs Phase B.

### NFR-0003 (binary size)

After c.1 embeds full icon set:
```bash
cargo build --release -p wyvern
ls -lh target/release/wyvern
```

If ≥ 10MB: prefer SVG-only bundle, strip metadata, or drop lowest-priority variant before v0.1.0.

### Cross-platform regression matrix

| Check | ubuntu | macos | windows |
|-------|--------|-------|---------|
| `cargo test --workspace -- --test-threads=1` | ✓ | ✓ | ✓ |
| Message + level icon render test | ✓ | ✓ | ✓ |
| Named icon variant test | ✓ | ✓ | ✓ |
| Win/Linux decorations false (c.3) | ✓ | N/A | ✓ |
| Question AskUserQuestion wire shape | ✓ | ✓ | ✓ |

No manual Win/Linux E2E required — CI is authoritative per Phase A/B policy.

## Explicit Code Samples

```rust
// Optional timing hook in test harness (non-flaky: generous bound in CI)
#[test]
#[cfg(target_os = "macos")]
fn message_open_latency_under_nfr() {
    let start = std::time::Instant::now();
    // ... harness opens message without blocking on user ...
    assert!(start.elapsed().as_millis() < 500, "NFR-0001");
}
```

```bash
# NFR-0003 manual gate (macOS release)
cargo build --release -p wyvern
stat -f%z target/release/wyvern  # must be < 10_485_760
```

## This Sprint Does Not Close

- GitHub release workflow — c.5
- Homebrew tap — optional stretch
- Wizard / MCP — later phases

## Acceptance Criteria

- NFR-0001–NFR-0003 measured and recorded (PR description or `docs/plans/phase-C/` note — not a new permanent doc unless values drift)
- All three CI OS legs green on `integrate/phase-C` head
- Phase B README smoke #1–#4 covered by passing tests or documented macOS manual run
- No known Win/Linux rendering blockers for v0.1.0
- Dialog auto-size bounds match Phase B constants in `lib.rs`

## Required Validation

- `cargo test --workspace -- --test-threads=1` (full matrix)
- `cargo clippy --workspace -- -D warnings`
- `sc-lint check native --config .sc-lint.toml`
- macOS release binary size check (NFR-0003)
- macOS latency/memory spot-check (NFR-0001, NFR-0002) — manual acceptable if automated hook too flaky for CI
