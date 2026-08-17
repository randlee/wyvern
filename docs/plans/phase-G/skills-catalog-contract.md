# Skills catalog contract (Phase G)

Normative schema for `wyvern extensions list --json` and `wyvern extensions show <id> --json`.

**Base extension semantics:** [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md)

## Registry optional fields (Phase G)

Added to each object in `extensions.json`:

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `description` | string | recommended | One-line agent-facing summary |
| `examples` | string[] | recommended | Copy-paste argv; at least one per shipped skill |

Parser must accept entries without these fields (backward compatible with project overrides).

## `SkillRecord` JSON (list --json / show --json)

Array element shape:

```json
{
  "id": "string",
  "match_kind": "string",
  "invocation": "string",
  "requires": [{ "binary": "string", "available": true }],
  "args": [{ "name": "string", "required": true, "repeat": false }],
  "expands_to": "markdown | wizard | …",
  "description": "string | null",
  "examples": ["string"],
  "extends": "string | null"
}
```

### Field rules

- `match_kind` — same human DSL as Phase F list (`suffix: .md`, `prefix: compose render`, `prefix+suffix: md .csv`, …)
- `invocation` — minimal copy-paste pattern; prefix skills include literal prefix tokens
- `requires[].available` — evaluated at list time via `RequiresProbe` (not cached)
- `args` — derived from `{arg:name}` and `{arg:name:repeat}` in preexec/expand templates via `declared_args()`
- `expands_to` — `expand.command.type` or inferred type when using `command_from_file`
- `extends` — parent `id` when `extends` key present in registry; else `null`

## Text mode

Plain `list` and `show` use `format_skill_card(&SkillRecord)` — same function as extension `--help`. Single source of truth in `catalog.rs`.

Each card includes:

- `id` and `match_kind` on the first lines
- `description` when present
- `Usage:` (`invocation`)
- `Requires:` — `(none)`, or `binary [available]` / `binary [missing]` per requires-gated skill
- `Expands to:`
- `Extends: <parent> (alias)` when `extends` is non-null (e.g. `csv-table-alias` notes alias of `csv-suffix`)
- `Example:` from `examples` (generated fallback when the registry omits them)

`list` prints one card per skill, separated by a blank line. `show <id>` prints one card.

## CLI routing

| Command | Behavior |
|---------|----------|
| `wyvern extensions` | Same as `list` |
| `wyvern extensions list` | Text catalog |
| `wyvern extensions list --json` | JSON array to stdout |
| `wyvern extensions show <id>` | One skill, text |
| `wyvern extensions show <id> --json` | One skill, JSON object |
| `wyvern extensions list --foo` | Error — unknown flag |

## Versioning

Catalog JSON has no top-level version field in g.3. Agents should treat unknown keys as ignorable. A future `catalog_version` field may be added without breaking g.3 consumers.
