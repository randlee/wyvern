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

**Amendment (Phase F / ADR-0022):** Before `load_command_input`, `main.rs` matches the host-flag-stripped argv remainder against the extension registry. A match expands to validated `Command` JSON plus optional `host.ui_root`, then enters this same pipeline. Unmatched remainder falls through to JSON / `.json` / stdin load. See principal [ADR-0022](../architecture.md) and the [ADR-0013 amendment](#adr-0013-amendment-phase-f--extension-argv-pipeline) below.

**Amendment (Phase G / ADR-0022):** After host-flag strip and before extension match: global `--help` / `-h` / `help` (exit 0); extension prefix `--help` skill cards via `match_extension_help` (exit 0); on no match, near-miss diagnostics (REQ-0136) before load fallthrough. CLI uses `match_with_diagnostics`; library `match_argv()` returns `.matched` only. See [agent-usability-contract.md](../plans/phase-G/agent-usability-contract.md) and REQ-0134–REQ-0137.

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

### ADR-0013 amendment (Phase F) — extension argv pipeline

| Stage | Error type | `error` slug | `code` | Exit |
|-------|------------|--------------|--------|------|
| Extension registry load | `ExtensionError::InvalidRegistry` | `parse` | `PARSE_ERROR` | 2 |
| Extension match miss (multi-token) | `LoadError::Usage` | `parse` | `PARSE_ERROR` | 2 |
| Extension expand / args | `ExtensionError::MissingArgs` / `UnexpectedArg` / `Template` | `validation` | `VALIDATION_ERROR` | 4 |
| Extension near-miss (Phase G) | `NearMissKind` via `emit_near_miss` | `parse` / `validation` | `PARSE_ERROR` / `VALIDATION_ERROR` | 2 / 4 |
| Extension preexec / I/O | `ExtensionError::Preexec` / `Io` | `io` | `IO_ERROR` | 3 |

Cross-link: [ADR-0022](../architecture.md) (extensions are an argv preprocessor; they do not add pipeline stages after validate).

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
