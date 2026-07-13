# Phase A — Foundation (`integrate/phase-A`)

Phase A implementation PRs target **`integrate/phase-A`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

## What Phase A closes

- Five-crate workspace under `crates/` (ADR-0011) from sprint a.1
- macOS window + HTML chrome shell
- JSON load → validate → dispatch → run → emit on **one** command: `type: "chrome"`
- Discriminated error enums per stage; CLI maps each variant to stderr JSON
- `sc-observability` at binary entry; `sc-lint` configured

## What Phase A does not close

- Dialog types (`message`, `input`, `markdown`, `question`, `wizard`) — Phase B+
- Windows/Linux platform chrome — Phase C (`integrate/phase-C`)
- `--interactive` / MCP — later phase
- Per-type validation beyond `chrome` — added as each type ships

## Direct-path execution model

```
argv/stdin → load (LoadError) → validate (ValidationError) → Command → match type → run (RunError) → CommandResult → stdout JSON
```

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

Win/Linux chrome deferred to Phase C — not counted in Phase A completion.

## External sibling deps (worktree layout)

From `crates/wyvern/Cargo.toml`, path deps use **`../../sc-observability`** and **`../../sc-lint`** (repo root is parent of `wyvern-worktrees/`, siblings sit beside `wyvern` and `wyvern-worktrees`).
