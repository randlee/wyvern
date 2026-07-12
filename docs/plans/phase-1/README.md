# Phase 1 — Foundation (`integrate/phase-A`)

Phase A implementation PRs target **`integrate/phase-A`**. This directory is the sprint-doc authority for Phase 1.

## What Phase 1 closes

- Five-crate workspace (ADR-0011) from sprint one
- macOS window + HTML chrome shell
- JSON load → validate → dispatch → execute → emit on **one** command: `type: "chrome"`
- Structured validation/parse errors on stderr; no window on failure
- `sc-observability` at binary entry; `sc-lint` configured

## What Phase 1 explicitly does not close

- Dialog types (`message`, `input`, `markdown`, `question`, `wizard`) — Phase 2+
- Windows/Linux platform chrome — Phase 3 (`S3.2a`)
- `--interactive` / MCP — Phase 5
- Full-schema validation for all types — added incrementally as each type ships

## Direct-path execution model

Wyvern must not accumulate routing complexity. Each phase adds **one handler per `type`**, not layers of mode flags.

```
argv/stdin → load_json() → validate() → Command enum → match type → handler → stdout JSON
```

**Red flags during implementation:**

- Multiple nested `if mode` / `if interactive` branches to choose output shape
- Stub handlers that silently no-op for unimplemented types
- Validation rules for types not yet executable in the current phase
- Window opened before validation completes

If a flow needs complicated branching to determine the path, stop and simplify the command surface or split the sprint.

## Foundation command: `chrome`

Phase 1 executable JSON:

```json
{ "type": "chrome", "title": "Window title", "status": "optional status line" }
```

Success on OS close:

```json
{ "button": "dismissed" }
```

`message` and other dialog types may appear in **tests** as rejected inputs until their phase ships.

## Sprint index (8)

| Sprint | Doc | Branch |
|--------|-----|--------|
| S1.1a | [s1a-scaffold.md](s1a-scaffold.md) | `feature/p1-s1a-scaffold` |
| S1.1b | [s1b-window.md](s1b-window.md) | `feature/p1-s1b-window` |
| S1.2a | [s2a-json-io.md](s2a-json-io.md) | `feature/p1-s2a-json-io` |
| S1.2b | [s2b-validation.md](s2b-validation.md) | `feature/p1-s2b-validation` |
| S1.3a | [s3a-chrome-frame.md](s3a-chrome-frame.md) | `feature/p1-s3a-chrome-frame` |
| S1.4 | [s4-sc-observability.md](s4-sc-observability.md) | `feature/p1-s4-sc-observability` |
| S1.5 | [s5-sc-lint.md](s5-sc-lint.md) | `feature/p1-s5-sc-lint` |

`S1.3b` (Win/Linux chrome) was removed from Phase 1; see Phase 3 `s2a-win-linux-chrome.md`.
