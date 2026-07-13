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
- Confirm NFR-0004–NFR-0007 remain satisfied (no regression) per Phase C README claim.
- Fix cross-platform rendering regressions; confirm all Phase B acceptance criteria on ubuntu, macos, and windows CI.

## Hard Dependencies

- c.1 icon asset bundle (binary size impact)
- c.2 icon resolution complete
- c.3 Win/Linux platform chrome

## Exact Targets

- `crates/wyvern-window/tests/message_ipc.rs` — message IPC + level icon render (Phase B smoke #1)
- `crates/wyvern-window/tests/input_ipc.rs` — input text submit (Phase B smoke #2)
- `crates/wyvern/tests/cli_validation.rs` — markdown file, `.md` shorthand, inline content (Phase B smoke #3)
- `crates/wyvern-window/tests/question_ipc.rs`, `question_dismiss_ipc.rs` — AskUserQuestion wire shape (Phase B smoke #4)
- `crates/wyvern-window/tests/blank_window.rs` — chrome dismiss baseline; c.3 adds `chrome_ipc.rs` for `window_close` / `window_minimize`
- `crates/wyvern-window/tests/message_ipc.rs` (extend) — modal `window_minimize` no-op (c.3 chrome IPC)
- `crates/wyvern-window/tests/chrome_ipc.rs` (new, c.3) — chrome `window_close` → `{"button":"dismissed"}`; chrome `window_minimize` without stdout
- `crates/wyvern-window/src/message/render.rs`, `input/render.rs`, `markdown/render.rs`, `question/render.rs` — auto-size clamp unit tests (`estimate_*_size`)
- `crates/wyvern-window/src/window.rs` — `DIALOG_MIN_*` / `DIALOG_MAX_*` applied via `with_min_inner_size` / `with_max_inner_size`
- `crates/wyvern-schema/tests/validation_message.rs`, `validation_input.rs` — named icon variant + unknown-name validation (c.2)
- `docs/requirements.md` — confirm NFR wording still valid (no edit unless drift found)
- `.github/workflows/ci.yml` — optional `nfr-smoke` job (macOS-only, non-blocking) if scripted checks land
- Any dialog template/CSS fixes discovered during Win/Linux visual regression

## Deliverables

- NFR-0001: window opens **< 500ms** on macOS (product target; manual measurement preferred — see below)
- NFR-0002: resident memory **< 80MB** on macOS under normal single-dialog operation
- NFR-0003: release binary **< 10MB** on macOS (`target/release/wyvern`); if over, document mitigation (asset compression, WebP) in PR — must not ship v0.1.0 over limit without explicit acceptance
- NFR-0004: no host browser required — wry embeds OS webview (WebKit / WebView2 / WebKitGTK); no regression from icon bundle or chrome changes
- NFR-0005: all three CI OS legs green — ubuntu (WebKitGTK + xvfb), macos (WebKit), windows (WebView2)
- NFR-0006: JSON schema 1:1 MCP mapping preserved — `cargo test -p wyvern-schema` validation suites pass; no field rename or restructure in Phase C diffs
- NFR-0007: validation errors remain human-readable — unknown icon names list valid catalog; no opaque error codes introduced
- No rendering regressions on Windows or Linux CI legs
- All Phase B README smoke scenarios pass via matrix rows M2, M4, M5, M6; macOS manual spot-check for icon + chrome combo
- Auto-size bounds unchanged: dialog **min 320×200**, **max 800×600**; chrome **480×360** default open

## Required Work — NFR verification (authoritative)

### NFR-0001 (latency)

**Product target:** p95 < **500ms** on macOS release build (cold start to first visible content).

**Measurement method (authoritative — not auto-dismiss):**

1. Build release: `cargo build --release -p wyvern`
2. Run cold start with a real message dialog (user or scripted wait for window visible)
3. Time from process start to **first paint** — use one of:
   - Manual stopwatch / screen observation (**preferred**)
   - WebView `load_finished` callback timestamp logged in a dev-only harness (**acceptable**)
   - `WYVERN_INJECT_IPC` after page load (measures post-load IPC round-trip only — **not** sufficient alone)
4. Record p95 over 5 runs in the c.4 PR description (see template below)

**Do not use** `WYVERN_AUTO_DISMISS` timing as NFR-0001 authority — auto-dismiss fires on a fixed timer unrelated to paint readiness and produces false pass/fail.

If over target: profile wry init / asset embed size; optimize before c.5 tag.

**CI policy:** Do not assert 500ms in blocking CI. Optional non-blocking macOS job may include a generous smoke bound (e.g. **2000ms**) using `load_finished` hook — document the 500ms product target separately in the PR description.

### NFR-0002 (memory)

With one message dialog open on macOS: resident size < 80MB. WebKit baseline dominates — icon bundle should not materially change this vs Phase B.

**Settle delay:** wait **≥ 2 seconds** after window visible before sampling RSS (WebKit allocates lazily on first paint).

```bash
# After opening a message dialog and waiting ≥ 2s for WebKit to settle:
ps -o rss= -p $(pgrep -x wyvern | head -1)   # RSS in KB; divide by 1024 for MB
# Target: RSS < 81920 KB (80 MB)
```

Activity Monitor is an acceptable alternative for manual verification (same ≥ 2s settle rule).

### NFR-0003 (binary size)

After c.1 embeds full icon set:
```bash
cargo build --release -p wyvern
ls -lh target/release/wyvern
```

If ≥ 10MB: prefer SVG-only bundle, strip metadata, or drop lowest-priority variant before v0.1.0.

### NFR-0004 (no host browser)

**Requirement:** Wyvern does not require a browser installed on the host system.

**Confirmation:** wry uses OS-embedded webviews only (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux). No shell-out to Chrome/Firefox/Safari. Phase C icon embed and Win/Linux chrome changes must not introduce external browser dependencies.

**Validation:** grep `Cargo.toml` workspace for new browser-launch deps (none expected); CI green on all three legs confirms WebView2/WebKitGTK runtime paths unchanged.

### NFR-0005 (cross-platform)

**Requirement:** Runs on macOS (WebKit), Windows (WebView2), and Linux (WebKitGTK).

**Confirmation:** full regression matrix (below) passes on ubuntu-latest, macos-latest, and windows-latest CI legs.

**Validation:** `cargo test --workspace -- --test-threads=1` green on all three legs; no platform-gated test skips for dialog types introduced in Phase C.

### NFR-0006 (MCP 1:1 schema)

**Requirement:** JSON schema for all dialog types maps 1:1 to MCP tool parameters — no field renaming or restructuring.

**Confirmation:** Phase C changes limited to icon resolution, platform chrome, and assets — no `Command`/`CommandResult` field renames.

**Validation:** `cargo test -p wyvern-schema` (all validation suites); `question_contract_examples.rs` still passes; review diff for schema/API surface changes (should be none beyond icon catalog expansion).

### NFR-0007 (actionable validation errors)

**Requirement:** Validation error messages are human-readable and actionable without consulting documentation.

**Confirmation:** c.2 unknown-icon errors list valid names; no new opaque error codes in Phase C.

**Validation:** `validation_message.rs` / `validation_input.rs` unknown-icon cases assert stderr includes role catalog; spot-check one unknown-icon CLI run in c.4 PR if c.2 landed on same branch.

### Cross-platform regression matrix

| Row | Check | Test target(s) | ubuntu | macos | windows |
|-----|-------|----------------|--------|-------|---------|
| M1 | `cargo test --workspace -- --test-threads=1` | CI workflow | ✓ | ✓ | ✓ |
| M2 | Message + level icon render | `message_ipc.rs`; `message/render.rs` unit tests | ✓ | ✓ | ✓ |
| M3 | Named icon variant + unknown name | `validation_message.rs`, `validation_input.rs` | ✓ | ✓ | ✓ |
| M4 | Phase B smoke #2 — input text submit | `input_ipc.rs` | ✓ | ✓ | ✓ |
| M5 | Phase B smoke #3 — markdown file/shorthand/inline | `cli_validation.rs` (`cli_valid_markdown_file_emits_dismissed`, `cli_markdown_md_shorthand_emits_dismissed`, `cli_markdown_content_inline_emits_dismissed`) | ✓ | ✓ | ✓ |
| M6 | Phase B smoke #4 — question wire shape | `question_ipc.rs`, `question_dismiss_ipc.rs` | ✓ | ✓ | ✓ |
| M7 | Win/Linux decorations false (c.3) | `window.rs` cfg tests | ✓ | N/A | ✓ |
| M8 | Chrome `window_close` IPC | `chrome_ipc.rs` (new) | ✓ | ✓ | ✓ |
| M9 | Modal `window_minimize` no-op | `message_ipc.rs` (extend) | ✓ | ✓ | ✓ |
| M10 | Chrome `window_minimize` (no stdout) | `chrome_ipc.rs` (new) | ✓ | ✓ | ✓ |
| M11 | Dialog auto-size bounds | `message/render.rs`, `input/render.rs`, `markdown/render.rs`, `question/render.rs` `estimate_*_size` tests; `window.rs` min/max attrs | ✓ | ✓ | ✓ |

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

## PR description — NFR measurement template (canonical)

Copy into the c.4 PR description:

```markdown
## NFR measurements (macOS release build)

| NFR | Target | Method | Result | Pass? |
|-----|--------|--------|--------|-------|
| NFR-0001 | p95 < 500ms cold start → first paint | Manual stopwatch (5 runs) / load_finished harness | ___ ms (p95) | ☐ |
| NFR-0002 | RSS < 80 MB, single message dialog | `ps -o rss=` after ≥ 2s settle | ___ MB | ☐ |
| NFR-0003 | Binary < 10 MB | `stat -f%z target/release/wyvern` | ___ MB | ☐ |
| NFR-0004 | No host browser required | Architecture review + dep grep | No new browser deps | ☐ |
| NFR-0005 | macOS / Windows / Linux CI | Full matrix rows M1–M11 | All legs green | ☐ |
| NFR-0006 | MCP 1:1 schema | `cargo test -p wyvern-schema` | Pass | ☐ |
| NFR-0007 | Actionable validation errors | Unknown-icon spot-check | Lists valid names | ☐ |

**NFR-0001 note:** auto-dismiss timing not used. Optional non-blocking CI smoke may use 2000ms bound via load_finished.
```

## This Sprint Does Not Close

- GitHub release workflow — c.5
- Homebrew tap — optional stretch
- Wizard / MCP — later phases

## Acceptance Criteria

- NFR-0001–NFR-0007 confirmed per matrix rows and recorded in the **c.4 PR description** using the template above (canonical location; not a new permanent doc unless values drift)
- NFR-0001 measured via manual timing (**preferred**) or `load_finished` harness — **not** auto-dismiss
- NFR-0002 sampled after **≥ 2s** settle delay post first paint
- All three CI OS legs green on `integrate/phase-C` head (matrix row M1)
- Phase B README smoke #1–#4 covered by matrix rows M2, M4, M5, M6
- c.3 chrome IPC covered by matrix rows M8–M10
- No known Win/Linux rendering blockers for v0.1.0
- Dialog auto-size bounds enforced in `crates/wyvern-window/src/window.rs` (`with_min_inner_size` / `with_max_inner_size`) and verified by `estimate_*_size` unit tests in dialog `render.rs` modules (matrix row M11)

## Required Validation

- `cargo test --workspace -- --test-threads=1` (full matrix row M1)
- `cargo clippy --workspace -- -D warnings`
- `sc-lint check native --config .sc-lint.toml`
- macOS release binary size check (NFR-0003)
- macOS latency/memory spot-check (NFR-0001, NFR-0002) — manual measurement preferred; optional non-blocking CI job with generous bounds
- `cargo test -p wyvern-schema` (NFR-0006, NFR-0007)
