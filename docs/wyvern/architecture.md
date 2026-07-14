# `wyvern` (CLI) — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0013 (local): CLI pipeline

`crates/wyvern/src/pipeline.rs` (exported via `lib.rs`) owns the stage chain; `main.rs` is a thin binary wrapper. Each stage owns a discriminated error enum:

1. `load_command_input() -> Result<Value, LoadError>` (`Parse` | `Io` | `Usage`)
2. `wyvern_schema::validate(value) -> Result<Command, ValidationError>`
3. `wyvern_host::begin` → `DialogHandle` (embedded) or `run()` (none/system/named) → **`embedded_viewer_spawn` when `--viewer embedded`** (c.15, CLI-owned) → `await_result` → `CommandResult`

**ADR-0013 HTTP exception:** Types not yet on the host handler matrix return `HostError::UnsupportedType` at run time after schema validation passes — see principal [ADR-0013 amendment](../architecture.md).

4. `emit_*` helpers on failure; `emit_stdout(CommandResult)` on success — both return `Result<_, EmitError>`

**Pipeline (c.15+):**

```text
load → validate → Command → host bind → DialogHandle
  → [CLI spawn wyvern-viewer when embedded]
  → [host browser_launch when system/named]
  → await_result → CommandResult → emit_stdout
```

`wyvern-host::run` is none/system/named only — embedded one-shot is CLI DialogHandle composition.

Load, validation, host bind, viewer spawn, and result await each map to exit ≠ 0 at the CLI boundary via [`PipelineError`]. Emit-stage serialize failures map to exit `8` (`internal` / `INTERNAL_ERROR`).

**Forbidden:** `--window-demo`, extra CLI flags, or any path that bypasses load → validate → bind → await.

### ADR-0013 amendment (c.6) — pipeline error stages

| Stage | Error type | `error` slug | `code` | Exit |
|-------|------------|--------------|--------|------|
| Load (parse) | `LoadError::Parse` | `parse` | `PARSE_ERROR` | 2 |
| Load (io) | `LoadError::Io` | `io` | `IO_ERROR` | 3 |
| Validate | `ValidationError` | `validation` / `state` | `VALIDATION_ERROR` / `STATE_ERROR` | 4 / 5 |
| Run (host bind/await) | `HostError` (`Bind`, `UiNotFound`, `ViewerNotFound`, …) | `host_bind` / `host_error` / `host_viewer` | `HOST_BIND_ERROR` / `HOST_ERROR` / `HOST_VIEWER_ERROR` | 6–7 |
| Run (viewer spawn) | `ViewerSpawnError` (missing binary, exec failure) | `host_viewer` | `HOST_VIEWER_ERROR` | 6 |
| Emit | `EmitError::Serialize` | `internal` | `INTERNAL_ERROR` | 8 |

`PipelineError::Stage` carries pre-built stderr JSON + stage exit code.
`PipelineError::Emit` triggers `emit_fatal_internal` (static JSON, no recursive serialize).

---

## ADR-0008: Interactive mode uses stdin readline loop

**Status:** Accepted

**Context:**
A persistent Wyvern window needs to receive updates over time. Options: named pipe/Unix socket, local HTTP server, or stdin readline loop.

**Decision:**
`--interactive` flag puts Wyvern into a readline loop on stdin. Each newline-delimited JSON object is a command. Blocking dialog commands use the **HTTP host** (ADR-0016) inside the loop; `show`, `hide`, and `exit` are lifecycle actions. Results go to stdout on completion.

**Amendment (c.10):** Dialog transport is HTTP (local host), not wry IPC. Stdin remains the command ingress for `--interactive`.
**Amendment (c.15):** `show`/`hide`/`exit` lifecycle and `wyvern-viewer` spawn are **`wyvern` CLI** concerns — not `HostSession` methods.
**Consequences:**
- Ephemeral HTTP port per session (configurable); remote viewers when bind allows
- Any agent or script can drive it by holding stdin/stdout handles open (background shell pattern)
- Sequential — commands processed one at a time (sufficient for UI interaction cadence)
