---
id: S1.3a
title: HTML chrome frame + chrome command E2E
status: planned
branch: feature/p1-s3a-chrome-frame
target: integrate/phase-A
---

# Sprint S1.3a — HTML chrome frame + `chrome` command E2E (macOS)

## Goal

- Complete Phase 1 acceptance path: validate `chrome` → render HTML shell → OS close → `{"button":"dismissed"}` on stdout.

## Hard Dependencies

- S1.1b window API
- S1.2b validation (`Command::Chrome`)

## Exact Targets

- `wyvern-window/src/chrome/` (HTML template + IPC if needed)
- `wyvern-window/src/dispatch.rs` (single `match` on `Command`)
- `wyvern-window/src/result.rs`
- `wyvern/src/main.rs` (wire load → validate → run → emit)

## Deliverables

- HTML chrome: title bar, content area, optional status bar, static placeholder content
- `run_command(Command) -> CommandResult` with one `Chrome` arm
- stdout emission: `{"button":"dismissed"}` on OS close
- Phase 1 AC #2 passes end-to-end

## Required Work

- Title bar shows `title` from command; optional `status` in status bar
- 72px macOS safe zone; `-webkit-app-region: drag` on title bar
- Auto-size window with max dimensions
- No stub match arms for unimplemented types (they never reach dispatch)

## Explicit Code Samples

```rust
pub struct CommandResult {
    pub button: String, // "dismissed" for Phase 1 chrome close
}

pub fn run_command(cmd: wyvern_schema::Command) -> Result<CommandResult, RunError>;

// dispatch.rs — must stay this simple
pub fn run_command(cmd: Command) -> Result<CommandResult, RunError> {
    match cmd {
        Command::Chrome { title, status } => run_chrome(title, status),
    }
}
```

## This Sprint Does Not Close

- `message` / dialog content rendering (Phase 2)
- Windows/Linux HTML window controls (Phase 3 `S3.2a`)
- Button bar interaction (static placeholder only)
- `sc-observability` / `sc-lint`

## Acceptance Criteria

- `wyvern '{"type":"chrome","title":"Foundation"}'` opens chrome; OS close → `{"button":"dismissed"}`
- `wyvern '{"type":"message",...}'` still fails at validation (no window)
- Dispatch is single-level `match` on `Command` — no nested mode routing
- Title bar drag works on macOS

## Required Validation

- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- Manual E2E: Phase 1 acceptance criteria #1–#3 from `docs/plans/project-plan.md`
