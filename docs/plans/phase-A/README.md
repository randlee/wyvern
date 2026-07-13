# Phase A — Foundation (`integrate/phase-A`)

Phase A implementation PRs target **`integrate/phase-A`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

## What Phase A closes

- Five-crate workspace under `crates/` (ADR-0011) from sprint a.1
- Native window + HTML chrome shell (all platforms; Win/Linux decoration polish → Phase C)
- JSON load → validate → dispatch → run → emit on **one** command: `type: "chrome"`
- Discriminated error enums per stage; CLI maps each variant to stderr JSON
- `sc-observability` (crates.io) at binary entry; `sc-lint` (crates.io) configured in CI

## What Phase A does not close

- Dialog types (`message`, `input`, `markdown`, `question`, `wizard`) — Phase B+
- Chrome **button bar** (empty placeholder in a.5; interactive buttons in Phase B)
- Windows/Linux **platform chrome polish** (custom decorations, HTML close/minimize) — Phase C (`integrate/phase-C`)
- `--interactive` — Phase E (`integrate/phase-E`)
- MCP server (`wyvern --mcp`, REQ-0074+) — Phase E only; Phase A has stub `wyvern-mcp` crate per ADR-0011
- Per-type validation beyond `chrome` — added as each type ships

## Phase A MCP posture

- `wyvern-mcp` is a **library stub only** (`lib.rs`; no `[[bin]]`, no stdio transport, no tool mapping).
- REQ-0074+ and `wyvern --mcp` are **N/A until Phase E**.
- Boundary greps or lint rules mentioning `wyvern-mcp` validate scaffold placement and ADR-0011 edges — not MCP server behavior.

## Plan review scope exclusions

**Out of scope** for Phase A plan review:

- MCP server implementation (REQ-0074+), `wyvern --mcp`, `docs/wyvern-mcp/requirements.md` deliverables
- `--interactive` mode and lifecycle actions
- Manual Win/Linux E2E (chrome open/close on Windows or Linux)

**In scope** for Phase A plan review:

- Stub `wyvern-mcp` crate (library-only `lib.rs`)
- ADR-0011 dependency edges across all five crates
- CI `cargo test --workspace` on ubuntu, macos, and windows

## Direct-path execution model

```
argv/stdin → load (LoadError) → validate (ValidationError) → Command → run (RunError) → CommandResult → stdout JSON
```

Dispatch (`match` on `Command`) is **internal** to `wyvern_window::run` — not a separate public stage.

One `type` → one handler. **No** `--window-demo`, **no** stub handlers, **no** mode flags on the product CLI.

## Error handling (discriminated unions)

Each stage owns its enum. The CLI **re-interprets** variants to stderr JSON at the boundary — never collapses unlike failures into one generic error.

| Stage | Crate | Enum | stderr `error` values |
|-------|-------|------|------------------------|
| Load | `wyvern` | `LoadError` | `parse`, `io`, (usage → non-JSON stderr + exit) |
| Validate | `wyvern-schema` | `ValidationError` | `validation`, `state` |
| Run | `wyvern-window` | `RunError` | `window_create`, `event_loop` |

## Foundation command: `chrome`

```json
{ "type": "chrome", "title": "Window title", "status": "optional status line" }
```

Success on OS close:

```json
{ "button": "dismissed" }
```

(`CommandResult::Chrome(ChromeResult { button: "dismissed" })` — see a.4 serde contract.)

## Sprint index (7 active: a.1–a.7)

| Sprint | Doc | Branch |
|--------|-----|--------|
| a.1 | [a1-scaffold.md](a1-scaffold.md) | `feature/phase-A-a1-scaffold` |
| a.2 | [a2-window.md](a2-window.md) | `feature/phase-A-a2-window` |
| a.3 | [a3-json-io.md](a3-json-io.md) | `feature/phase-A-a3-json-io` |
| a.4 | [a4-validation.md](a4-validation.md) | `feature/phase-A-a4-validation` |
| a.5 | [a5-chrome-frame.md](a5-chrome-frame.md) | `feature/phase-A-a5-chrome-frame` |
| a.6 | [a6-sc-observability.md](a6-sc-observability.md) | `feature/phase-A-a6-sc-observability` |
| a.7 | [a7-sc-lint.md](a7-sc-lint.md) | `feature/phase-A-a7-sc-lint` |

Win/Linux decoration polish deferred to Phase C. Cross-platform window and `chrome` behavior is validated by **CI `cargo test --workspace`** on ubuntu, macos, and windows — not manual E2E on Win/Linux.

## External dependencies (crates.io)

| Crate / tool | Source | Pin |
|--------------|--------|-----|
| `sc-observability` | [crates.io](https://crates.io/crates/sc-observability) | `"1.2"` in workspace `Cargo.toml` |
| `sc-lint` | [crates.io](https://crates.io/crates/sc-lint) | `cargo install sc-lint --version 0.4` (CI + local) |

No path deps or sibling repo checkouts for either package.

## Platform policy (Phase A interim)

| Platform | Window chrome in Phase A | Deferred to Phase C |
|----------|--------------------------|---------------------|
| macOS | Transparent title bar (ADR-0010), HTML chrome shell | — |
| Windows/Linux | **Native OS decorations** on blank-window + chrome tests | `decorations: false` + HTML close/minimize (ADR-0010a, REQ-0085) |

**Cross-platform development:** Code is written with cross-platform patterns from day one. Local dev may use **xwin** Rust tooling for cross-target builds; Win/Linux validation is **automated tests in CI only** — no manual E2E on Windows or Linux in Phase A. macOS may keep optional manual chrome E2E during development.

## CI validation (authoritative)

All sprint docs reference this section for matrix closure — do not defer to `project-plan.md`.

| Leg | Prerequisites | Commands |
|-----|---------------|----------|
| `ubuntu-latest` | `libwebkit2gtk-4.1-dev`; **xvfb** for GUI tests | `xvfb-run -a cargo test --workspace` |
| `macos-latest` | — | `cargo test --workspace` |
| `windows-latest` | WebView2 runtime (preinstalled on `windows-latest`) | `cargo test --workspace` |

Every leg also runs: `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`.

After a.7: `cargo install sc-lint --version 0.4 --locked && sc-lint check --config .sc-lint.toml`.

### Phase acceptance

**CI gate (authoritative — all platforms):** `cargo test --workspace` passes on ubuntu, macos, and windows (see matrix above). This proves cross-platform window tests, validation, and chrome wiring without manual Win/Linux E2E.

**Optional macOS manual smoke (dev only — not a Win/Linux gate):**

1. `wyvern '{"type":"message","title":"T"}'` → validation stderr, exit ≠ 0, no window
2. `wyvern '{"type":"chrome","title":"Foundation"}'` → chrome opens; OS close → `{"button":"dismissed"}`
3. `wyvern '{"type":"unknown"}'` → validation stderr on `type`, exit ≠ 0, no window
