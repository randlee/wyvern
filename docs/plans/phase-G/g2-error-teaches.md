---
id: g.2
title: Error-teaches — near-miss diagnostics and preexec recovery
status: planning
branch: feature/phase-G-g2-error-teaches
worktree: ../wyvern-worktrees/feature/phase-G-g2-error-teaches
target: integrate/phase-G
---

# Sprint g.2 — Error-teaches

## Goal

Replace misleading `PARSE_ERROR` / generic usage fallthrough with structured diagnostics that name the skill and the next argv. Preexec failures classify spawn-not-found vs nonzero-exit vs missing input; recovery must not say “install binaries” when the binary ran.

## Hard dependencies

- **g.1** merged to `integrate/phase-G` (`format_skill_card`, help patterns)
- Phase F extension runtime on `develop`

## Deliverables

| Path | Change |
|------|--------|
| `crates/wyvern/src/extensions/mod.rs` | `match_with_diagnostics()` → `MatchOutcome { matched, skipped }`; `SkippedExtension` records id + missing binary |
| `crates/wyvern/src/extensions/diagnostics.rs` | **New.** Near-miss formatters + JSON envelope builders |
| `crates/wyvern/src/main.rs` | Before JSON parse fallthrough: unknown suffix, incomplete prefix, bare prefix token, skipped-only match |
| `crates/wyvern/src/input.rs` | Stop treating unknown file paths as inline JSON when extension diagnostics apply |
| `crates/wyvern/src/extensions/expand.rs` | `MissingArg`: report all missing required args in one error; include full declared list + example |
| `crates/wyvern/src/extensions/preexec.rs` | Pipe child stderr (tail cap e.g. 4 KiB); attach to `ExtensionError::Preexec` |
| `crates/wyvern/src/error.rs` | Caller-facing `UnexpectedArg` / `MissingArg` / `Preexec` recovery; in-binary recovery before doc paths |
| `crates/wyvern/tests/extension_diagnostics.rs` | **New.** Near-miss subprocess tests (scoped env, no global `PATH` mutation) |
| `crates/wyvern/tests/preexec_recovery.rs` | **New.** Mock cmd / fixture preexec failure classification |

### Rust API (signatures)

```rust
// mod.rs
pub struct SkippedExtension {
    pub id: String,
    pub missing: Vec<String>, // requires binaries absent on PATH
}

pub struct MatchOutcome<'a> {
    pub matched: Option<ExtensionMatch<'a>>,
    pub skipped: Vec<SkippedExtension>,
}

impl ExtensionRegistry {
    pub fn match_with_diagnostics<'a>(
        &'a self,
        argv: &'a [String],
        probe: &dyn RequiresProbe,
    ) -> MatchOutcome<'a>;
}

// diagnostics.rs
pub enum NearMissKind {
    UnknownInput { token: String },
    IncompletePrefix { tokens: Vec<String>, hint: String },
    BarePrefix { token: String, extension_id: String, usage: String },
    SkippedRequires { path: String, skipped: Vec<SkippedExtension> },
}

pub fn format_near_miss(kind: &NearMissKind) -> String;

// preexec.rs
pub enum PreexecFailureKind {
    SpawnNotFound { cmd: String },
    NonZeroExit { code: i32, stderr_tail: String },
    Timeout,
}

// expand.rs — MissingArg carries full context
pub enum ExtensionError {
    MissingArgs {
        missing: Vec<String>,
        declared: BTreeSet<String>,
        extension_id: String,
        example: String,
    },
    // …
}
```

### Normative stderr snippets (automated tests grep these substrings)

| Invocation | Must contain | Must not contain |
|------------|--------------|------------------|
| `wyvern notes.txt` | `unknown input`, `.md`, `extensions list` | `Input was not valid JSON` |
| subprocess with stub probe: csv skipped | `csv-suffix`, `python3`, `wyvern sample.csv` | `PARSE_ERROR` |
| `wyvern md` | `csv-md`, `<file.csv>` | `not valid JSON` |
| `wyvern compose` | `compose render`, `--root` | `PARSE_ERROR` |
| `wyvern compose render` (no flags) | `--root`, `--file` in same envelope | `declare {arg:name}` |
| preexec missing CSV file | `could not read` or path hint in `cause` | `Install binaries listed in preexec.requires` |

### Paths to delete

None.

## Acceptance criteria

### Automated

1. `cargo test -p wyvern-cli --test extension_diagnostics` passes all near-miss cases above
2. `cargo test -p wyvern-cli --test preexec_recovery` passes spawn-not-found vs nonzero-exit vs missing-file branches
3. `UnexpectedArg` recovery strings never contain `declare them as {arg:name}`
4. `MissingArg` for `compose render` lists both `--root` and `--file` before retry is required
5. Preexec nonzero exit: JSON envelope includes child stderr fragment in `cause` or structured `details`
6. Preexec spawn `ENOENT`: recovery mentions installing the named binary only
7. Tests use subprocess + injected probe or temp fixtures — **no** `std::env::set_var("PATH", …)` in parallel test threads
8. `cargo fmt --all --check && cargo clippy --workspace -- -D warnings` clean

### Manual (non-gating)

- `PATH=/usr/bin wyvern fixtures/sample.csv` when `python3` absent — readable multi-line hint

## Required validation

```bash
cargo test -p wyvern-cli --test extension_diagnostics
cargo test -p wyvern-cli --test preexec_recovery
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
./target/debug/wyvern notes.txt 2>&1 | rg -v 'not valid JSON'
./target/debug/wyvern compose render 2>&1 | rg '--root'
./target/debug/wyvern compose render 2>&1 | rg '--file'
```

## Non-closure

- `extensions list --json`, `extensions show`, registry `description`/`examples` → **g.3**
- Typo inference (`wyvern compsoe`) → out of scope (P2)
- `wyvern extensions dump` → out of scope (P2)

## Authority

- [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md) — P0 #2, P0 #5 (recovery half), P1 #6–#8
- [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md)
- g.1 `skill_card.rs` — reuse example lines in error recovery
