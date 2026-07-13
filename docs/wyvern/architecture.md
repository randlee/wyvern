# `wyvern` (CLI) — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0013 (local): CLI pipeline

`crates/wyvern/src/pipeline.rs` (exported via `lib.rs`) owns the stage chain; `main.rs` is a thin binary wrapper. Each stage owns a discriminated error enum:

1. `load_command_input() -> Result<Value, LoadError>` (`Parse` | `Io` | `Usage`)
2. `wyvern_schema::validate(value) -> Result<Command, ValidationError>`
3. `wyvern_window::run(command) -> Result<CommandResult, RunError>`
4. `emit_*` helpers on failure; `emit_stdout(CommandResult)` on success — both return `Result<_, EmitError>`

Load, validation, and run failures each map to exit ≠ 0 at the CLI boundary via [`PipelineError`]. Emit-stage serialize failures map to exit `8` (`internal` / `INTERNAL_ERROR`).

**Forbidden:** `--window-demo`, extra CLI flags, or any path that bypasses load → validate → run.

### ADR-0013 amendment (c.6) — pipeline error stages

| Stage | Error type | `error` slug | `code` | Exit |
|-------|------------|--------------|--------|------|
| Load (parse) | `LoadError::Parse` | `parse` | `PARSE_ERROR` | 2 |
| Load (io) | `LoadError::Io` | `io` | `IO_ERROR` | 3 |
| Validate | `ValidationError` | `validation` / `state` | `VALIDATION_ERROR` / `STATE_ERROR` | 4 / 5 |
| Run (window) | `RunError::WindowCreate` (incl. icon/embed defense-in-depth) | `window_create` | `WINDOW_CREATE_ERROR` | 6 |
| Run (loop) | `RunError::EventLoop` | `event_loop` | `EVENT_LOOP_ERROR` | 7 |
| Emit | `EmitError::Serialize` | `internal` | `INTERNAL_ERROR` | 8 |

`PipelineError::Stage` carries pre-built stderr JSON + stage exit code.
`PipelineError::Emit` triggers `emit_fatal_internal` (static JSON, no recursive serialize).

---

## ADR-0008: Interactive mode uses stdin readline loop

**Status:** Accepted

**Context:**
A persistent Wyvern window needs to receive updates over time. Options: named pipe/Unix socket, local HTTP server, or stdin readline loop.

**Decision:**
`--interactive` flag puts Wyvern into a readline loop on stdin. Each newline-delimited JSON object is a command. Blocking dialog commands retain their normal modal behavior inside the loop; `show`, `hide`, and `exit` are lifecycle actions for that loop. Results go to stdout on completion. Process exits on `{"action":"exit"}` or window close. `--persistent` is an alias.

**Consequences:**
- No socket setup or port conflicts
- Works identically in CLI and MCP modes
- Any agent or script can drive it by holding stdin/stdout handles open (background shell pattern)
- Sequential — commands processed one at a time (sufficient for UI interaction cadence)
