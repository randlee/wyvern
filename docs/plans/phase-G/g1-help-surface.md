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

Ship first-class `--help` (exit 0). Global usage lists every Phase F extension skill with copy-paste examples. Prefix extensions answer `--help`/`-h` with a skill card — not registry-author `UnexpectedArg`.

Help output goes to **stdout**; exit code **0**. Errors remain stderr JSON from existing emit paths.

## Hard dependencies

- Phase F merged to `develop` (extension runtime + shipped pack)
- [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md) P0 #1, P0 #4, P1 #9, P1 #10

## Deliverables

| Path | Change |
|------|--------|
| `crates/wyvern/src/main.rs` | Early intercept before host-flag parse: `--help`, `-h`, lone `help` → print usage, exit 0 |
| `crates/wyvern/src/cli_args.rs` | `usage_message()` — Extensions block (seven invocations), `--ui-root` wizard-root override note, env block (`WYVERN_VIEWER`, `WYVERN_UI_ROOT`, `WYVERN_SHARE`) |
| `crates/wyvern/src/extensions/skill_card.rs` | **New.** Skill card formatter (see signatures) |
| `crates/wyvern/src/extensions/expand.rs` | `expand_and_validate`: if prefix remainder is only `--help`/`-h`, return `ExtensionError::HelpRequested` (or dedicated outcome) before `parse_named_args` |
| `crates/wyvern/src/error.rs` | Emit help card on stdout, exit 0 (not validation stderr) |
| `crates/wyvern/src/extensions/list.rs` | `extensions_usage_message()`; handle `--help`/`-h` in `run_extensions_command` |
| `crates/wyvern/src/browsers_cmd.rs` | `browsers_usage_message()`; handle `--help`/`-h` |
| `crates/wyvern/tests/help_surface.rs` | **New.** Integration tests (see AC) |

### Normative Extensions block (must appear verbatim in `usage_message()`)

```
Extensions (see `wyvern extensions list`):
  wyvern doc.md
  wyvern page.html
  wyvern path/to/wizard.json
  wyvern data.csv
  wyvern table data.csv          # same interactive table as data.csv
  wyvern md data.csv             # CSV as a markdown dialog
  wyvern compose render --root DIR --file FILE.j2 [--var k=v] [--var-file vars.json]
```

### Rust API (signatures)

```rust
// skill_card.rs
pub fn is_help_only_argv(tokens: &[String]) -> bool;

pub fn format_skill_card(ext: &ExtensionDef, probe: &dyn RequiresProbe) -> String;

// expand.rs — new error variant or parallel result type
pub enum ExtensionError {
    // …existing…
    HelpRequested { text: String },
    // …
}

// list.rs
pub fn extensions_usage_message() -> String;

// browsers_cmd.rs
pub fn browsers_usage_message() -> String;
```

**Skill card text shape (compose-render reference):**

```
Usage: wyvern compose render --root <DIR> --file <FILE> [--var k=v] [--var-file F] [--env-prefix P]
Requires: sc-compose [available|missing]
Expands to: wizard
Example: wyvern compose render --root fixtures/compose-minimal --file page.j2
```

Card derives `--flag` list from `declared_args()` in `expand.rs`; `requires` availability from `RequiresProbe`; `Expands to` from `expand.command.type` (or `command_from_file` → `wizard`).

### Paths to delete

None.

## Acceptance criteria

### Automated

1. `cargo test -p wyvern-cli --test help_surface` passes
2. `wyvern --help` and `wyvern -h` exit **0**; stdout contains `.csv`, `table`, `md data.csv`, and `compose render`
3. `wyvern help` exit **0**; stdout matches `--help` body (not `PARSE_ERROR`)
4. `wyvern compose render --help` exit **0**; stdout contains `--root`, `--file`, `Requires:`, `Example:`; must not contain `declare them as {arg:name}`
5. `wyvern extensions --help` exit **0**; mentions `list` and `show`
6. `wyvern browsers --help` exit **0**; mentions `list` and `refresh`
7. Bare TTY `wyvern` (no args) behavior unchanged from Phase F (exit 1 usage) — g.1 does not change this path
8. `cargo fmt --all --check && cargo clippy --workspace -- -D warnings` clean

### Manual (non-gating)

- Visual scan: host-flag and env sections readable in terminal width

## Required validation

```bash
cargo test -p wyvern-cli --test help_surface
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
./target/debug/wyvern --help; test $? -eq 0
./target/debug/wyvern compose render --help; test $? -eq 0
./target/debug/wyvern extensions --help; test $? -eq 0
```

## Non-closure

- Near-miss diagnostics (unknown suffix, skipped requires, bare `md`) → **g.2**
- `extensions list --json`, `extensions show` → **g.3**
- Exit-code dictionary on global help → out of scope (P2)
- Bare TTY `wyvern` exit 0 → out of scope (optional follow-on)

## Authority

- [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md) — P0 #1, #4
- [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md) — match/expand semantics
- [README.md](README.md) — phase acceptance smoke
