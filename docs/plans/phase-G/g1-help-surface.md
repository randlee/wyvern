---
id: g.1
title: Help surface — global and extension-local
status: planning
branch: feature/phase-G-g1-help-surface
worktree: ../wyvern-worktrees/feature/phase-G-g1-help-surface
target: integrate/phase-G
---

# Sprint g.1 — Help surface

## Goal

Ship first-class `--help` (exit 0). Global usage lists every Phase F extension skill with copy-paste examples. Prefix extensions answer `--help`/`-h` with a skill card at **match time** — before `requires` skip and without requiring a suffix path token.

Help output goes to **stdout**; exit code **0**. Failures remain stderr JSON from existing emit paths.

## Hard dependencies

- Phase F merged to `develop`
- [agent-usability-contract.md](agent-usability-contract.md) — pipeline order, match-time help

## Deliverables

| Path | Change |
|------|--------|
| `docs/architecture.md` | ADR-0022 Phase G amendment subsection (help intercept + match-time help) |
| `docs/plans/phase-F/cli-extensions-contract.md` | Phase G cross-link to agent-usability-contract |
| `docs/wyvern/requirements.md` | REQ-0130 amendment row (pipeline order) |
| `crates/wyvern/src/main.rs` | Global help after host-flag strip; extension help via `match_extension_help` before `match_argv` |
| `crates/wyvern/src/cli_args.rs` | `usage_message()` — Extensions block, wizard-root note, env block |
| `crates/wyvern/src/extensions/mod.rs` | `match_extension_help(registry, argv) -> Option<&ExtensionDef>`; `is_help_only_tokens` — prefix + help-only remainder, ignores requires/suffix |
| `crates/wyvern/src/extensions/catalog.rs` | **Stub only in g.1:** `SkillRecord`, `build_skill_record()`, `format_skill_card()` — minimal fields for help (g.3 extends same types) |
| `crates/wyvern/src/extensions/expand.rs` | `expand_and_validate` → `Result<ExpandedInvocation, ExtensionError>` (CLI help never reaches expand) |
| `crates/wyvern/src/extensions/list.rs` | `extensions_usage_message()`; `--help`/`-h` (mentions `list` only — `show` is g.3) |
| `crates/wyvern/src/browsers_cmd.rs` | `browsers_usage_message()`; `--help`/`-h` |
| `crates/wyvern/tests/help_surface.rs` | **New.** Integration tests (see AC) |

### Normative Extensions block

```
Extensions (see `wyvern extensions list`):
  wyvern doc.md
  wyvern page.html
  wyvern path/to/wizard.json
  wyvern data.csv
  wyvern table data.csv          # same interactive table as data.csv
  wyvern md data.csv             # CSV as a markdown dialog
  wyvern compose render --root DIR --file FILE.j2 [--var k=v] [--var-file vars.json] [--env-prefix PREFIX]
```

Optional `{arg:env-prefix:repeat}` flags appear in shipped `compose-render` preexec args; global help and skill cards must document them alongside `--var` / `--var-file`.

### Rust API (signatures)

```rust
// mod.rs — match-time help (before requires / suffix)
pub fn match_extension_help<'a>(
    registry: &'a ExtensionRegistry,
    argv: &'a [String],
) -> Option<&'a ExtensionDef>;

pub fn is_help_only_tokens(tokens: &[String]) -> bool;

// catalog.rs (stub; g.3 completes list/show fields)
pub struct SkillRecord { /* id, invocation, requires, args, expands_to, examples, … */ }

pub fn build_skill_record(ext: &ExtensionDef, probe: &dyn RequiresProbe) -> SkillRecord;

pub fn format_skill_card(record: &SkillRecord) -> String;

// expand.rs — unchanged return type on CLI path (help handled in mod.rs)
pub fn expand_and_validate(
    ext: &ExtensionDef,
    ctx: &MatchContext,
) -> Result<ExpandedInvocation, ExtensionError>;
```

Extension `--help` is handled exclusively by `match_extension_help` + `format_skill_card(build_skill_record(...))` in `main.rs` — not via expand.

### Paths to delete

None.

## Acceptance criteria

### Automated

1. `cargo test -p wyvern-cli --test help_surface` passes
2. `wyvern --help` / `wyvern -h` exit **0**; stdout contains `.csv`, `table`, `md data.csv`, `compose render`
3. `wyvern help` exit **0**; same body as `--help`
4. `wyvern compose render --help` exit **0** with `--root`, `--file`, `Requires:`, `Example:` — even when `RequiresProbe` reports `sc-compose` missing
5. `wyvern md --help` and `wyvern table --help` exit **0** with skill card (no `.csv` path required)
6. `wyvern extensions --help` exit **0**; mentions `list` (does **not** require `show` — g.3)
7. `wyvern browsers --help` exit **0**; mentions `list` and `refresh`
8. `cargo fmt --all --check && cargo clippy --workspace -- -D warnings` clean

### Manual (non-gating)

- Host-flag + env sections readable at default terminal width

## Required validation

```bash
cargo test -p wyvern-cli --test help_surface
cargo fmt --all --check && cargo clippy --workspace -- -D warnings
./target/debug/wyvern --help; test $? -eq 0
./target/debug/wyvern compose render --help; test $? -eq 0
./target/debug/wyvern md --help; test $? -eq 0
./target/debug/wyvern extensions --help; test $? -eq 0
```

## Non-closure

- Near-miss diagnostics → **g.2**
- `extensions list --json`, `extensions show`, full `SkillRecord` catalog → **g.3**
- Exit-code dictionary → P2
- Bare TTY `wyvern` (exit 1) unchanged — g.1 must not regress; no new test required

## Authority

- [agent-usability-contract.md](agent-usability-contract.md)
- [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md) — P0 #1, #4
