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
- `--interactive` / MCP — later phase
- Per-type validation beyond `chrome` — added as each type ships

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

Win/Linux decoration polish deferred to Phase C — window tests and `chrome` E2E run on all CI platforms in Phase A.

## External dependencies (crates.io)

| Crate / tool | Source | Pin |
|--------------|--------|-----|
| `sc-observability` | [crates.io](https://crates.io/crates/sc-observability) | `"1.2"` in workspace `Cargo.toml` |
| `sc-lint` | [crates.io](https://crates.io/crates/sc-lint) | `cargo install sc-lint --version 0.4` (CI + local) |

No path deps or sibling repo checkouts for either package.
