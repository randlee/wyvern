---
id: c.7
title: CLI integration test hardening
status: pending
branch: feature/phase-C-c7-cli-test-hardening
target: integrate/phase-C-fixes
---

# Sprint c.7 — CLI integration test hardening

## Goal

- Serialize **macOS GUI-spawning** CLI integration tests so local `cargo test -p wyvern` without `--test-threads=1` does not flake.
- Centralize child-process failure detection in one spawn helper.

**Non-goal:** CI already enforces `--test-threads=1` ([README.md §CI validation](README.md#ci-validation-authoritative)) — c.7 does not change CI workflow.

## Hard Dependencies

- **c.6 merged** to `integrate/phase-C-fixes` (production child must not panic on icon/embed paths)

## Exact Targets

- `crates/wyvern/tests/cli_validation.rs`
- `crates/wyvern/Cargo.toml` — dev-dependency `serial_test = "3"`
- `docs/linting.md` — local dev policy one-liner (authoritative for dev ergonomics)

## Deliverables

### 1. `serial_test` on every GUI-spawning test (macOS)

Dependency: `serial_test = "3"` in `[dev-dependencies]`.

```rust
use serial_test::serial;

/// Spawns `wyvern` with auto-dismiss; detects child panic/abort/signal failures.
fn run_wyvern(mut cmd: Command) -> std::process::Output {
    cmd.env("WYVERN_AUTO_DISMISS", "1").env_remove("WYVERN_LOG");
    let output = cmd.output().expect("spawn wyvern");
    assert_child_ok(&output);
    output
}

fn run_json(json: &str) -> (i32, String, String) {
    let output = run_wyvern(wyvern().arg(json));
    // ... extract code/stdout/stderr ...
}

fn assert_child_ok(output: &std::process::Output) {
    assert!(child_failed(
        &String::from_utf8_lossy(&output.stderr),
        output.status.code(),
    ));
}

/// Pure predicate — unit-testable without synthesizing `ExitStatus`.
fn child_failed(stderr: &str, code: Option<i32>) -> bool {
    stderr.contains("panicked at")
        || stderr.contains("misaligned pointer")
        || stderr.contains("cannot unwind")
        || stderr.contains("abort")
        || code == Some(-1)
}
```

```rust
#[test]
fn child_failed_detects_panic_marker() {
    assert!(child_failed("thread 'main' panicked at winit\n", Some(0)));
    assert!(!child_failed("", Some(0)));
}
```

Extract `child_failed` / `assert_child_ok` to a `mod child_assert` in `cli_validation.rs` (or `tests/support/child.rs`).

**Every** spawn path (including markdown shorthand) goes through `run_wyvern` / `run_json`.

### 2. GUI tests — exhaustive `#[serial]` list

| Test function | Spawns GUI |
|---------------|------------|
| `cli_valid_chrome_emits_dismissed` | yes |
| `cli_type_message_level_accepted` | yes |
| `cli_valid_message_emits_dismissed` | yes |
| `cli_valid_input_emits_dismissed` | yes |
| `cli_valid_input_file_mode_emits_dismissed` | yes |
| `cli_valid_markdown_file_emits_dismissed` | yes |
| `cli_markdown_md_shorthand_emits_dismissed` | yes |
| `cli_markdown_content_inline_emits_dismissed` | yes |
| `cli_question_auto_dismiss_emits_req_0068` | yes |

Validation-only tests (no `#[serial]` required): all others in `cli_validation.rs`.

### 3. `docs/linting.md` local dev policy

Add under **Canonical command**:

```markdown
Always pass `--test-threads=1` for workspace tests on macOS (winit/objc races when
multiple webview children spawn). CI already enforces this; local runs must match.
```

### 4. Known flake (this doc — §Known flakes)

```bash
# Fails on macOS without --test-threads=1 (winit macos/view.rs, objc2 weak_id)
cargo test -p wyvern -- --test-threads=8
```

Expected signature: `uninitialized instance variable`, `misaligned pointer dereference`, or child exit `-1`.

## This Sprint Does Not Close

- Changing CI matrix commands (already `--test-threads=1`)
- Production panic removal (c.6)
- Clippy deny (c.8)

## Acceptance Criteria

- All nine GUI tests above carry `#[serial]`
- `cli_markdown_md_shorthand_emits_dismissed` uses `run_wyvern` (not raw `.output()`)
- `child_failed_detects_panic_marker` unit test passes
- `cargo test -p wyvern -- --test-threads=1` passes on macOS
- `docs/linting.md` contains local `--test-threads=1` policy

## Required Validation

- `cargo test -p wyvern -- --test-threads=1`
- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern child_failed_detects_panic_marker -- --test-threads=1`
- `cargo clippy --workspace -- -D warnings`
