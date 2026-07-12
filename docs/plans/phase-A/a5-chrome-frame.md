---
id: a.5
title: HTML chrome frame + chrome command E2E
status: planned
branch: feature/phase-A-a5-chrome-frame
target: integrate/phase-A
---

# Sprint a.5 — HTML chrome frame + `chrome` command E2E (macOS)

## Goal

- Complete Phase A acceptance: validate → run → emit for `type: "chrome"`.

## Hard Dependencies

- a.2 window API
- a.4 validation + `CommandResult` in schema

## Exact Targets

- `crates/wyvern-window/src/chrome/`
- `crates/wyvern-window/src/run.rs`
- `crates/wyvern/src/main.rs` (load → validate → run → stdout)

## Deliverables

- HTML chrome shell (title bar, content placeholder, optional status bar)
- `wyvern_window::run(command: Command) -> Result<CommandResult, RunError>`
- Single `match` on `Command` with one `Chrome` arm
- Phase A AC #2 end-to-end

## Required Work

- Title bar shows `title`; optional `status` in status bar; **status bar hidden when `status` absent**
- 72px macOS safe zone; `-webkit-app-region: drag`
- OS close → stdout `{"button":"dismissed"}` (`CommandResult` from schema)
- Auto-size with max width/height

## Explicit Code Samples

```rust
// wyvern-window — returns schema protocol type
pub fn run(command: wyvern_schema::Command) -> Result<wyvern_schema::CommandResult, RunError>;

pub fn run(command: Command) -> Result<CommandResult, RunError> {
    match command {
        Command::Chrome { title, status } => run_chrome(title, status),
    }
}
```

## This Sprint Does Not Close

- Dialog content rendering (Phase B)
- Win/Linux chrome (Phase C)
- Interactive/MCP

## Acceptance Criteria

- `wyvern '{"type":"chrome","title":"Foundation"}'` opens chrome; OS close → `{"button":"dismissed"}`
- `wyvern '{"type":"message",...}'` still fails at validation (no window)
- Dispatch is one-level `match` — no stub arms
- Status bar not rendered when `status` omitted

## Required Validation

- `cargo test --workspace`
- Manual E2E: Phase A acceptance criteria #1–#3 in `docs/plans/project-plan.md`
- `cargo clippy --workspace -- -D warnings`
