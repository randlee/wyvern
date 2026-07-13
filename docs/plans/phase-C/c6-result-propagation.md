---
id: c.6
title: Result propagation — eliminate production unwrap/expect
status: pending
branch: feature/phase-C-c6-result-propagation
target: integrate/phase-C-fixes
---

# Sprint c.6 — Result propagation (no production panics)

## Goal

- Eliminate all production `unwrap()`, `expect()`, and `unreachable!` in lib `src/` per the §1 checklist below.
- Return discriminated unions through every layer; map errors to structured stderr at the CLI boundary only.

## Hard Dependencies

- Phase C merged to `develop` (`41e3e24+`)

## Exact Targets

- `crates/wyvern-window/src/message/media.rs` — `icon_html_for_level`, `resolve_named_icon_svg`
- `crates/wyvern-window/src/message/render.rs` — propagate `RunError` from media resolution
- `crates/wyvern-window/src/run.rs` — bubble `RunError` from render/setup (already mostly `Result`)
- `crates/wyvern-schema/src/stderr.rs` — `SerializeError`, `to_json_string`
- `crates/wyvern-schema/src/error_code.rs` — `ErrorCode::InternalError`
- `crates/wyvern/src/error.rs` — emit helpers, `EmitError`, load-error split
- `crates/wyvern/src/pipeline.rs` — `PipelineError`, `run_from_loaded`
- `crates/wyvern/src/lib.rs` — export `PipelineError`; **remove** `handle_run_failure` export
- `crates/wyvern/src/main.rs` — load + pipeline `PipelineError` / `EmitError` handling
- `docs/wyvern/architecture.md` — ADR-0013 amendment + pipeline error table
- `docs/wyvern-schema/requirements.md` — **REQ-0074** emit-stage `internal` wire contract (mandatory)

## Deliverables

### §1 production panic removal (authoritative closure checklist)

| # | File | Lines (base) | Fix |
|---|------|--------------|-----|
| 1 | `wyvern-window/src/message/media.rs` | 21–22 | `icon_html_for_level` → `Result<IconHtml, RunError>` |
| 2 | `wyvern-window/src/message/media.rs` | 112–114 | `resolve_named_icon_svg` → `Result<&'static str, RunError>` |
| 3 | `wyvern/src/error.rs` | 60 | Remove `unreachable!` via `emit_parse_error` / `emit_io_error` (see samples) |
| 4 | `wyvern/src/error.rs` | 183 | `emit_stdout` → `Result<String, EmitError>` |
| 5 | `wyvern-schema/src/stderr.rs` | 95 | `to_json_string` → `Result<String, SerializeError>` |

- Icon/embed failures map to **`RunError::WindowCreate`** (REQ-0073 `window_create` slug — no new run variant)
- `ErrorCode::InternalError` — slug `internal`, exit `8` — for `EmitError` only
- **`handle_run_failure` deleted** — pipeline inlines `emit_run_error(&e)?` + exit-code match (see samples)
- **`emit_load_error` deleted** — replaced by `emit_parse_error` / `emit_io_error` only
- Static stderr fallback when structured emit fails (no recursive `to_json_string`)
- ADR-0013 amended: emit-stage failures + icon defense-in-depth documented
- [UNWRAP-INVENTORY.md](UNWRAP-INVENTORY.md) §1 rows marked **FIXED** with commit SHA (audit trail only)

## Closed decisions (wire contract)

| Decision | Choice |
|----------|--------|
| Icon/embed miss after validation | `RunError::WindowCreate { message }` → `error: "window_create"`, exit `6` |
| Stderr/stdout serialize failure | `EmitError::Serialize` → `ErrorCode::InternalError`, slug `internal`, exit `8` |
| `SerializeError` owner | `wyvern-schema` (`stderr.rs`); `EmitError` in `wyvern` wraps it |
| `LoadError::Usage` | Never reaches load emit helpers — handled only in `main` before pipeline |
| `handle_run_failure` | **Removed** — stage exit codes live in `PipelineError::Stage` |

### REQ-0074 (mandatory — wyvern-schema requirements)

Add after REQ-0073:

> **REQ-0074** — Emit-stage failures: when stdout or stderr JSON serialization fails at the CLI boundary (`EmitError::Serialize`), Wyvern emits `{ "error": "internal", "code": "INTERNAL_ERROR", "message": "..." }` and exits `8`. Applies only to emit helpers in `crates/wyvern`; does not change load/validate/run slugs.

### `SerializeError` + `to_json_string` (wyvern-schema)

```rust
// crates/wyvern-schema/src/stderr.rs
#[derive(Debug)]
pub struct SerializeError {
    pub message: String,
}

impl StderrError {
    pub fn to_json_string(&self) -> Result<String, SerializeError> {
        serde_json::to_string(self).map_err(|e| SerializeError {
            message: e.to_string(),
        })
    }
}
```

### `ErrorCode::InternalError` (wyvern-schema)

```rust
// crates/wyvern-schema/src/error_code.rs — additive variant
InternalError,  // slug "internal", exit_code() => 8
```

### `EmitError` + emit helpers (wyvern)

```rust
// crates/wyvern/src/error.rs
#[derive(Debug)]
pub enum EmitError {
    Serialize(SerializeError),
}

pub fn emit_stdout(result: &CommandResult) -> Result<String, EmitError> {
    serde_json::to_string(result)
        .map_err(|e| EmitError::Serialize(SerializeError { message: e.to_string() }))
}

pub fn emit_parse_error(err: &LoadError) -> Result<String, EmitError> {
    let LoadError::Parse { message } = err else {
        debug_assert!(matches!(err, LoadError::Parse { .. }));
        return Err(EmitError::Serialize(SerializeError {
            message: "emit_parse_error: expected Parse".into(),
        }));
    };
    StderrError::new(ErrorCode::ParseError, message.clone())
        /* ... recovery ... */
        .to_json_string()
        .map_err(EmitError::Serialize)
}

pub fn emit_io_error(err: &LoadError) -> Result<String, EmitError> {
    let LoadError::Io { field, message } = err else {
        debug_assert!(matches!(err, LoadError::Io { .. }));
        return Err(EmitError::Serialize(SerializeError {
            message: "emit_io_error: expected Io".into(),
        }));
    };
    StderrError::new(ErrorCode::IoError, message.clone())
        .field(field.clone())
        /* ... recovery ... */
        .to_json_string()
        .map_err(EmitError::Serialize)
}

pub fn emit_run_error(err: &RunError) -> Result<String, EmitError> {
    let envelope = match err {
        RunError::WindowCreate { message } => StderrError::new(ErrorCode::WindowCreateError, message.clone()),
        RunError::EventLoop { message } => StderrError::new(ErrorCode::EventLoopError, message.clone()),
    };
    envelope.to_json_string().map_err(EmitError::Serialize)
}

// emit_validation_error — same Result<String, EmitError> + StderrError envelope pattern
pub fn emit_validation_error(err: &ValidationError) -> Result<String, EmitError> {
    let envelope = match err {
        ValidationError::Validation { field, message } => { /* existing recovery */ }
        ValidationError::State { field, message } => { /* existing recovery */ }
    };
    envelope.to_json_string().map_err(EmitError::Serialize)
}
```

**Emit cascade rule:** each stage calls `emit_*(&err).map_err(PipelineError::Emit)?` then `PipelineError::Stage { stderr, exit_code }` on success — domain exit codes 2–7 are preserved; only `PipelineError::Emit` maps to exit 8 via `emit_fatal_internal`.

### `PipelineError` (replaces `Result<String, (String, i32)>` + `handle_run_failure`)

```rust
// crates/wyvern/src/pipeline.rs
#[derive(Debug)]
pub enum PipelineError {
    /// Stage failed after structured stderr was built successfully.
    Stage { stderr: String, exit_code: i32 },
    /// Stdout or stage stderr JSON could not be serialized.
    Emit(EmitError),
}

pub fn run_from_loaded(value: Value) -> Result<String, PipelineError> {
    let command = match wyvern_schema::validate(&value) {
        Ok(cmd) => cmd,
        Err(e) => {
            let stderr = emit_validation_error(&e).map_err(PipelineError::Emit)?;
            return Err(PipelineError::Stage {
                stderr,
                exit_code: e.exit_code(),
            });
        }
    };

    let command = match load_markdown_file(command) {
        Ok(cmd) => cmd,
        Err(e) => {
            let stderr = emit_io_error(&e).map_err(PipelineError::Emit)?;
            return Err(PipelineError::Stage {
                stderr,
                exit_code: e.exit_code(),
            });
        }
    };

    let result = match wyvern_window::run(command) {
        Ok(r) => r,
        Err(e) => {
            let stderr = emit_run_error(&e).map_err(PipelineError::Emit)?;
            let exit_code = match &e {
                RunError::WindowCreate { .. } => ErrorCode::WindowCreateError.exit_code(),
                RunError::EventLoop { .. } => ErrorCode::EventLoopError.exit_code(),
            };
            return Err(PipelineError::Stage { stderr, exit_code });
        }
    };

    emit_stdout(&result).map_err(PipelineError::Emit)
}
```

**Delete** `handle_run_failure` from `error.rs` and `lib.rs` re-exports.

### media.rs — `WindowCreate` (not a new variant)

```rust
pub fn icon_html_for_level(level: MessageLevel) -> Result<IconHtml, RunError> {
    let markup = icons::svg_markup(level.as_str(), 1).ok_or_else(|| RunError::WindowCreate {
        message: format!("missing level icon embed for {}", level.as_str()),
    })?;
    Ok(markup.to_string())
}

fn resolve_named_icon_svg(spec: &str) -> Result<&'static str, RunError> {
    let (role, index) = schema_icons::parse_icon_spec(spec).map_err(|()| RunError::WindowCreate {
        message: format!("invalid icon spec '{spec}'"),
    })?;
    icons::svg_markup(&role, index).ok_or_else(|| RunError::WindowCreate {
        message: format!("missing embed for {role}:{index}"),
    })
}
```

### `main.rs` — load stage + pipeline (authoritative)

```rust
// crates/wyvern/src/main.rs
use wyvern::{
    emit_fatal_internal, emit_io_error, emit_parse_error, load_command_input,
    run_from_loaded, usage_message, LoadError, PipelineError,
};

fn main() -> ExitCode {
    // ... version, usage, observability unchanged ...

    let value = match load_command_input(&args, io::stdin()) {
        Ok(value) => value,
        Err(LoadError::Usage { message }) => {
            eprintln!("{message}");
            return ExitCode::from(1);
        }
        Err(err) => return emit_load_stage_failure(&err),
    };

    match run_from_loaded(value) {
        Ok(stdout) => {
            let mut out = io::stdout().lock();
            let _ = writeln!(out, "{stdout}");
            ExitCode::SUCCESS
        }
        Err(PipelineError::Stage { stderr, exit_code }) => {
            eprintln!("{stderr}");
            ExitCode::from(u8::try_from(exit_code).unwrap_or(1))
        }
        Err(PipelineError::Emit(e)) => emit_fatal_internal(&e),
    }
}

fn emit_load_stage_failure(err: &LoadError) -> ExitCode {
    let emit_result = match err {
        LoadError::Parse { .. } => emit_parse_error(err),
        LoadError::Io { .. } => emit_io_error(err),
        LoadError::Usage { .. } => unreachable!("Usage handled in main match above"),
    };
    match emit_result {
        Ok(stderr) => {
            eprintln!("{stderr}");
            ExitCode::from(u8::try_from(err.exit_code()).unwrap_or(1))
        }
        Err(e) => emit_fatal_internal(&e),
    }
}

// crates/wyvern/src/error.rs
pub fn emit_fatal_internal(err: &EmitError) -> ! {
    let EmitError::Serialize(e) = err;
    let msg_json =
        serde_json::to_string(&e.message).unwrap_or_else(|_| "\"serialization failed\"".into());
    eprintln!(r#"{{"error":"internal","code":"INTERNAL_ERROR","message":{msg_json}}}"#);
    std::process::exit(ErrorCode::InternalError.exit_code());
}
```

### ADR-0013 amendment (normative text for architecture.md)

```markdown
## ADR-0013 amendment (c.6) — pipeline error stages

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
```

### Error mapping table (emit at CLI)

| Source | `error` slug | `code` | Exit |
|--------|--------------|--------|------|
| `RunError::WindowCreate` (incl. icon/embed) | `window_create` | `WINDOW_CREATE_ERROR` | 6 |
| `RunError::EventLoop` | `event_loop` | `EVENT_LOOP_ERROR` | 7 |
| `EmitError::Serialize` | `internal` | `INTERNAL_ERROR` | 8 |

## This Sprint Does Not Close

- `input/render.rs` — no production `unwrap`/`expect` today (grep gate below); no code change required
- CLI test serialization — **c.7**
- Clippy deny regression gate — **c.8**
- `wyvern-wizard` / `wyvern-mcp` lib denies — **c.8** non-closure (no violations today)

## Acceptance Criteria

- §1 checklist rows 1–5 **FIXED** in code
- Unit test: missing level embed → `RunError::WindowCreate`
- Unit test: `to_json_string` / `emit_stdout` serialize path returns `Err` on forced failure (test-only mock type or `#[cfg(test)]` hook)
- `cargo test --workspace -- --test-threads=1` passes
- `cargo clippy --workspace -- -D warnings` clean (c.8 adds denies; c.6 must not introduce new production panics)
- `sc-lint check native --config .sc-lint.toml` clean
- REQ-0074 present in `docs/wyvern-schema/requirements.md` with slug `internal` and exit `8`
- `rg 'emit_load_error|handle_run_failure' crates/wyvern/src` returns **zero** matches (deleted)
- `rg 'unwrap\(|\.expect\(|panic!|unreachable!' crates/wyvern-window/src/input/render.rs` — zero matches outside `mod tests` (proves out-of-scope)

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo clippy --workspace -- -D warnings`
- `sc-lint check native --config .sc-lint.toml`
- `rg 'unwrap\(|\.expect\(|panic!|unreachable!' crates/wyvern-window/src/input/render.rs` (must be test-only)
- `rg 'REQ-0074' docs/wyvern-schema/requirements.md` (must match)
- `rg 'emit_load_error|handle_run_failure' crates/wyvern/src` (must be zero)
