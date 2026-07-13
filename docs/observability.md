# Wyvern observability

Structured logging for the Wyvern CLI via [`sc-observability`](https://crates.io/crates/sc-observability) **1.2** (crates.io only — no path dependency).

## Dependency pin

| Location | Declaration |
|----------|-------------|
| Workspace `Cargo.toml` | `sc-observability = "1.2"` under `[workspace.dependencies]` |
| `crates/wyvern/Cargo.toml` | `sc-observability = { workspace = true }` |

Only the `wyvern` binary crate depends on `sc-observability`. Library crates (`wyvern-schema`, `wyvern-window`, `wyvern-wizard`, `wyvern-mcp`) must not import it.

## Environment

| Variable | Values | Effect |
|----------|--------|--------|
| `WYVERN_LOG` | `off` \| `error` \| `warn` \| `info` \| `debug` \| `trace` | Minimum severity. Unset or `off` disables logging (no-op hooks). |

When enabled, events go to **stderr** (JSONL console sink) so stdout remains clean JSON protocol output. Retained JSONL files are written under a temp-dir log root (`$TMPDIR/wyvern-observability/`).

Example:

```bash
WYVERN_LOG=debug wyvern '{"type":"chrome","title":"Foundation"}'
```

## Normative events

| Event (`action`) | Level | Where emitted |
|------------------|-------|---------------|
| `process_start` | info | `main` after `observability::init()` |
| `command_received` | info | `pipeline::run_from_loaded` entry |
| `validation_result` | info | after `wyvern_schema::validate` (field `ok`) |
| `window_open` | info | before `wyvern_window::run` |
| `window_close` | info | after successful `wyvern_window::run` |
| `result_emitted` | info | before returning stdout JSON |
| `error` | error | validation or run failure (`stage`, `detail`) |

Event keys above are normative; `sc-observability` symbol names may differ as long as behavior matches.

## Pipeline hook map

```text
main
  ├─ observability::init()
  ├─ observability::log_process_start
  └─ load → pipeline::run_from_loaded
       ├─ log_command_received
       ├─ validate
       │    ├─ log_validation_result(true|false)
       │    └─ log_error("validate", …) on failure
       ├─ log_window_open
       ├─ wyvern_window::run
       │    ├─ log_window_close on success
       │    └─ log_error("run", …) on failure
       └─ log_result_emitted
```

Implementation: `crates/wyvern/src/observability.rs` (wrapper) and `crates/wyvern/src/pipeline.rs` (hooks).

## Out of scope

- Logging inside non-binary crates
- `--interactive` / MCP logging (Phase E)
