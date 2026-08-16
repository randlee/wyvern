---
id: g.3
title: Skill catalog — list, JSON, show
status: planning
branch: feature/phase-G-g3-skill-catalog
worktree: ../wyvern-worktrees/feature/phase-G-g3-skill-catalog
target: integrate/phase-G
---

# Sprint g.3 — Skill catalog

## Goal

Make `wyvern extensions list` a parseable skill index: rich text mode, `--json` array, and `extensions show <id>`. Shipped registry gains optional `description` and `examples` fields consumed by the catalog.

## Hard dependencies

- **g.1** + **g.2** merged to `integrate/phase-G`
- `RequiresProbe`, `declared_args()`, `format_skill_card()` from prior sprints

## Deliverables

| Path | Change |
|------|--------|
| `share/wyvern/extensions.json` | Add `description` + `examples[]` on all seven shipped extensions |
| `crates/wyvern/src/extensions/mod.rs` | Parse optional `description: Option<String>`, `examples: Vec<String>` on `ExtensionDef` |
| `crates/wyvern/src/extensions/catalog.rs` | **New.** `SkillRecord` builder (see signatures) |
| `crates/wyvern/src/extensions/list.rs` | Rich text formatter; `list --json`; `show <id>` and `show <id> --json`; reject unknown flags on `list` |
| `docs/plans/phase-G/skills-catalog-contract.md` | **New.** JSON record schema + registry optional fields |
| `crates/wyvern/tests/extensions_catalog.rs` | **New.** Catalog integration tests |

### Registry optional fields (all seven entries populated)

```json
{
  "id": "csv-md",
  "description": "Render a CSV file as a markdown pipe table dialog",
  "examples": ["wyvern md fixtures/sample.csv"],
  "match": { "argv_prefix": ["md"], "arg_suffix": ".csv" },
  "…": "…"
}
```

Minimum: every shipped `id` has non-empty `description` and at least one `example`.

### Rust API (signatures)

```rust
// catalog.rs
#[derive(Debug, Clone, Serialize)]
pub struct SkillArg {
    pub name: String,
    pub required: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillRequire {
    pub binary: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillRecord {
    pub id: String,
    pub match_kind: String,       // human summary, same DSL as list today
    pub invocation: String,       // copy-paste argv pattern
    pub requires: Vec<SkillRequire>,
    pub args: Vec<SkillArg>,
    pub expands_to: String,       // markdown | wizard | …
    pub description: Option<String>,
    pub examples: Vec<String>,
    pub extends: Option<String>,
}

pub fn build_skill_records(
    registry: &ExtensionRegistry,
    probe: &dyn RequiresProbe,
) -> Vec<SkillRecord>;

pub fn format_skill_record_text(record: &SkillRecord) -> String;

// list.rs
pub fn run_extensions_command(args: &[String]) -> Result<String, ExtensionsCmdError>;
// subcommands: list [--json], show <id> [--json], --help
```

### Normative `--json` record (compose-render)

```json
{
  "id": "compose-render",
  "match_kind": "prefix: compose render",
  "invocation": "wyvern compose render --root DIR --file FILE",
  "requires": [{ "binary": "sc-compose", "available": true }],
  "args": [
    { "name": "root", "required": true, "repeat": false },
    { "name": "file", "required": true, "repeat": false },
    { "name": "var", "required": false, "repeat": true },
    { "name": "var-file", "required": false, "repeat": true },
    { "name": "env-prefix", "required": false, "repeat": true }
  ],
  "expands_to": "wizard",
  "description": "Render an sc-compose Jinja template and preview HTML",
  "examples": ["wyvern compose render --root fixtures/compose-minimal --file page.j2"],
  "extends": null
}
```

### Normative text block (csv-table-alias)

```
csv-table-alias
  match:     prefix+suffix  wyvern table <file.csv>
  extends:   csv-suffix (same interactive HTML table)
  requires:  python3        [available]
  expands:   wizard
  example:   wyvern table fixtures/sample.csv
```

### Paths to delete

None.

## Acceptance criteria

### Automated

1. `cargo test -p wyvern-cli --test extensions_catalog` passes
2. `wyvern extensions list --json` exit **0**; stdout parses as JSON array; `length >= 7`
3. Every JSON record has keys: `id`, `match_kind`, `invocation`, `requires`, `args`, `expands_to`
4. `wyvern extensions list` (plain) exit **0**; output includes `[available]` or `[missing]` per requires-gated skill
5. Plain list for `csv-table-alias` includes `extends` or explicit alias wording referencing `csv-suffix`
6. `wyvern extensions show csv-md` exit **0**; stdout contains `md`, `markdown`, example line
7. `wyvern extensions show no-such-id` exit **non-zero**; usage or structured error names valid ids
8. `wyvern extensions list --json` is **not** silently ignored (stdout starts with `[`)
9. `share/wyvern/extensions.json` — all seven ids have `description` + `examples` (validated by test or `extensions_catalog` parse)
10. `cargo fmt --all --check && cargo clippy --workspace -- -D warnings` clean

### Manual (non-gating)

- `wyvern extensions list --json | jq` — agent can pick invocation without reading repo files

## Required validation

```bash
cargo test -p wyvern-cli --test extensions_catalog
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
./target/debug/wyvern extensions list --json | jq 'length >= 7'
./target/debug/wyvern extensions show compose-render | rg '--root'
./target/debug/wyvern extensions show missing-id; test $? -ne 0
```

## Non-closure

- `wyvern extensions dump` (full merged registry) → out of scope (P2)
- `wyvern skills` argv alias → out of scope (P2)
- README quickstart CSV lines → docs track; not a sprint deliverable
- Exit-code dictionary on global help → out of scope (P2)
- `--available` filter flag on list → out of scope (P2)

## Authority

- [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md) — P0 #3, P1 #11
- [skills-catalog-contract.md](skills-catalog-contract.md) — normative JSON schema (this sprint)
- [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md) — match/expand base schema
