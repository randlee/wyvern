---
id: c.4
title: Cross-platform validation and NFR pass
status: pending
branch: feature/phase-C-c4-nfr-validation
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

- NFR-0001: window opens **< 500ms** on macOS (product target; **manual measurement only** — see below)
- NFR-0002: resident memory **< 80MB** on macOS under normal single-dialog operation
- NFR-0003: release binary **< 10MB** on macOS (`target/release/wyvern`); if over, document mitigation (asset compression, WebP) in PR — must not ship v0.1.0 over limit without explicit acceptance
- No rendering regressions on Windows or Linux CI legs
- All Phase B README smoke scenarios pass via automated tests where possible; macOS manual spot-check for icon + chrome combo
- Auto-size bounds unchanged: dialog **min 320×200**, **max 800×600**; chrome **480×360** default open

## Required Work — NFR verification (authoritative)

### NFR-0001 (latency)

**Product target:** p95 < **500ms** on macOS release build (cold start to first visible content).

**Measurement method (authoritative — not auto-dismiss):**

1. Build release: `cargo build --release -p wyvern`
2. Run cold start with a real message dialog (user or scripted wait for window visible)
3. Time from process start to **first paint** — use one of:
   - Manual stopwatch / screen observation (preferred)
   - WebView `load_finished` callback timestamp logged in a dev-only harness
   - `WYVERN_INJECT_IPC` after page load (measures post-load IPC round-trip only — **not** sufficient alone)
4. Record p95 over 5 manual runs in the c.4 PR description

**Do not use** `WYVERN_AUTO_DISMISS` timing as NFR-0001 authority — auto-dismiss fires on a fixed timer unrelated to paint readiness and produces false pass/fail.

If over target: profile wry init / asset embed size; optimize before c.5 tag.

**CI policy:** Do not assert 500ms in blocking CI. Optional non-blocking macOS job may include a generous smoke bound (e.g. **2000ms**) using `load_finished` hook — document the 500ms product target separately in the PR description.

### NFR-0002 (memory)

With one message dialog open on macOS: resident size < 80MB. WebKit baseline dominates — icon bundle should not materially change this vs Phase B.

```bash
# After opening a message dialog (manual or test harness holding window open):
ps -o rss= -p $(pgrep -x wyvern | head -1)   # RSS in KB; divide by 1024 for MB
# Target: RSS < 81920 KB (80 MB)
```

Activity Monitor is an acceptable alternative for manual verification.

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
// Optional non-blocking CI smoke only — uses load_finished, NOT auto-dismiss
#[test]
#[cfg(target_os = "macos")]
fn message_open_latency_smoke() {
    let start = std::time::Instant::now();
    // harness: record Instant in WebView load_finished callback
    // CI bound: generous; product target is 500ms (see PR description)
    assert!(start.elapsed().as_millis() < 2000, "NFR-0001 CI smoke");
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

- NFR-0001–NFR-0003 measured and recorded in the **c.4 PR description** (canonical location; not a new permanent doc unless values drift)
- NFR-0001 measured via manual timing or `load_finished` hook — **not** auto-dismiss
- All three CI OS legs green on `integrate/phase-C` head
- Phase B README smoke #1–#4 covered by passing tests or documented macOS manual run
- No known Win/Linux rendering blockers for v0.1.0
- Dialog auto-size bounds match Phase B constants in `lib.rs`

## Required Validation

- `cargo test --workspace -- --test-threads=1` (full matrix)
- `cargo clippy --workspace -- -D warnings`
- `sc-lint check native --config .sc-lint.toml`
- macOS release binary size check (NFR-0003)
- macOS latency/memory spot-check (NFR-0001, NFR-0002) — manual measurement preferred; optional non-blocking CI job with generous bounds
