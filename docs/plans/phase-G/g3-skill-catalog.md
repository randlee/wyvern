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

Complete the skill catalog: rich text list, `--json` **array**, `extensions show <id>`, registry `description`/`examples`. All text output uses g.1 `format_skill_card(build_skill_record(...))` — single formatter.

## Hard dependencies

- **g.1** + **g.2** merged
- [agent-usability-contract.md](agent-usability-contract.md) + [skills-catalog-contract.md](skills-catalog-contract.md)

## Deliverables

| Path | Change |
|------|--------|
| `docs/wyvern/requirements.md` | REQ-0132 amendment (skill index + `--json` array) |
| `share/wyvern/extensions.json` | Add `description` + `examples[]` on all seven shipped extensions; verify `compose-render.preexec.args` uses `--output` / `--env-prefix` (already correct on `develop` post Phase F — regression guard) |
| `crates/wyvern/src/extensions/mod.rs` | Parse optional `description`, `examples` on `ExtensionDef` |
| `crates/wyvern/src/extensions/catalog.rs` | Complete `SkillRecord`, `build_skill_records()`, extend g.1 stub |
| `crates/wyvern/src/extensions/list.rs` | Rich list; `list --json`; `show <id>`; `show <id> --json`; update `extensions --help` to mention `show` |
| `docs/plans/phase-G/skills-catalog-contract.md` | Finalize schema (pre-existing scaffold; g.3 implementation must conform) |
| `crates/wyvern/tests/extensions_catalog.rs` | **New.** Catalog tests |

### Formatter ownership (normative)

```rust
// catalog.rs — sole text formatter
pub fn format_skill_card(record: &SkillRecord) -> String;

// list.rs / main help path — all call:
let text = format_skill_card(&build_skill_record(ext, probe));
```

No `format_skill_record_text`. g.1 help path updated to use completed `SkillRecord` if stub was minimal.

### `--json` wire

Stdout is a **JSON array** of `SkillRecord` for `list --json`; single object for `show <id> --json`.

### Paths to delete

None.

## Acceptance criteria

### Automated

1. `cargo test -p wyvern-cli --test extensions_catalog` passes
2. `wyvern extensions list --json` exit **0**; stdout parses as JSON **array**; `length >= 7`; stdout starts with `[`
3. Every record has keys: `id`, `match_kind`, `invocation`, `requires`, `args`, `expands_to`, `description`, `examples`, `extends` (`null`/`[]` allowed)
4. Plain `list` includes `[available]` or `[missing]` per requires-gated skill
5. `csv-table-alias` notes `extends` / alias of `csv-suffix`
6. `wyvern extensions show csv-md` exit **0**; same facts as JSON record text form
7. `wyvern extensions show no-such-id` exit non-zero
8. `wyvern extensions --help` mentions `show` (delivered in g.3)
9. All seven registry ids have non-empty `description` + `examples`
10. `compose-render` preexec args in shipped JSON use `--output` (not `--out`) and `--env-prefix` (not `--env`); no `--format html` token — verified by unit test parsing `SHIPPED_EXTENSIONS_JSON`
11. When `sc-compose` on PATH: `wyvern compose render --root fixtures/compose-minimal --file page.j2` expand + preexec path succeeds (manual non-gating if fixture absent in CI)
12. `cargo fmt --all --check && cargo clippy --workspace -- -D warnings` clean

### Manual (non-gating)

- Agent can pick invocation from `list --json` alone

## Required validation

```bash
cargo test -p wyvern-cli --test extensions_catalog
./target/debug/wyvern extensions list --json | jq 'length >= 7'
./target/debug/wyvern extensions show compose-render | rg '--root'
```

## Non-closure

- `extensions dump` → P2
- `wyvern skills` alias → P2
- README quickstart → docs track

## Authority

- [skills-catalog-contract.md](skills-catalog-contract.md)
- [agent-usability-contract.md](agent-usability-contract.md)
