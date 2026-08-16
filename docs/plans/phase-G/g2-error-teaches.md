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

Replace misleading `PARSE_ERROR` / generic usage fallthrough with structured `StderrError` JSON that names the skill and next argv. Preexec failures classify spawn-not-found vs nonzero-exit; recovery must not say “install binaries” when the binary ran.

## Hard dependencies

- **g.1** merged (`match_with_diagnostics` hook points, `format_skill_card`, `ExpandOutcome`)
- [agent-usability-contract.md](agent-usability-contract.md) — near-miss table + wire format

## Deliverables

| Path | Change |
|------|--------|
| `docs/architecture.md` | ADR-0022 note: CLI uses `match_with_diagnostics`; library `match_argv` unchanged for Phase E |
| `docs/wyvern/requirements.md` | REQ-0130 near-miss layer wording |
| `crates/wyvern/src/extensions/mod.rs` | `match_with_diagnostics()` → `MatchOutcome { matched, skipped }`; `match_argv()` wraps `.matched` only |
| `crates/wyvern/src/extensions/diagnostics.rs` | **New.** `NearMissKind` → `StderrError` envelope per contract table |
| `crates/wyvern/src/main.rs` | Near-miss decision table before `load_command_input` (no registry logic in `input.rs`) |
| `crates/wyvern/src/extensions/expand.rs` | `ExtensionError::MissingArgs { … }` replaces `MissingArg`; list all missing required flags |
| `crates/wyvern/src/extensions/preexec.rs` | Pipe stderr (4 KiB tail); `PreexecFailureKind` without `Timeout` |
| `crates/wyvern/src/error.rs` | Caller-facing `UnexpectedArg` / `MissingArgs` / `Preexec` recovery; in-binary recovery first |
| `crates/wyvern/tests/extension_diagnostics.rs` | **New.** Near-miss subprocess tests |
| `crates/wyvern/tests/preexec_recovery.rs` | **New.** Spawn vs nonzero-exit classification |

### Rust API (signatures)

```rust
pub struct SkippedExtension {
    pub id: String,
    pub missing: Vec<String>,
}

pub struct MatchOutcome<'a> {
    pub matched: Option<ExtensionMatch<'a>>,
    pub skipped: Vec<SkippedExtension>,
}

impl ExtensionRegistry {
    pub fn match_with_diagnostics<'a>(...) -> MatchOutcome<'a>;
    // match_argv unchanged: self.match_with_diagnostics(argv, probe).matched
}

pub enum NearMissKind {
    UnknownInput { token: String },
    IncompletePrefix { tokens: Vec<String>, hint: String },
    BarePrefix { token: String, extension_id: String, usage: String },
    SkippedRequires { path: String, skipped: Vec<SkippedExtension> },
}

pub fn emit_near_miss(kind: &NearMissKind) -> Result<String, EmitError>; // StderrError JSON

pub enum PreexecFailureKind {
    SpawnNotFound { cmd: String },
    NonZeroExit { code: i32, stderr_tail: String },
}

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

### Near-miss wire (normative)

| Kind | `code` | Exit |
|------|--------|------|
| `UnknownInput` | `USAGE_ERROR` | 2 |
| `SkippedRequires`, `IncompletePrefix`, `BarePrefix` | `VALIDATION_ERROR` | 4 |

Tests grep `recovery` / `message`; must not contain `Input was not valid JSON` for `notes.txt` or skipped csv.

### Paths to delete

None.

## Acceptance criteria

### Automated

1. `cargo test -p wyvern-cli --test extension_diagnostics` passes near-miss table cases
2. `cargo test -p wyvern-cli --test preexec_recovery` passes spawn vs nonzero-exit branches
3. `wyvern notes.txt` → `USAGE_ERROR` or `VALIDATION_ERROR`, not `PARSE_ERROR` “not valid JSON”
4. Stub probe csv skipped → names `csv-suffix`, `python3`, example argv
5. `wyvern md` → csv-md usage with `<file.csv>`
6. `wyvern compose render` (no flags) → `MissingArgs` lists `--root` and `--file` in one envelope
7. `UnexpectedArg` recovery never contains `declare them as {arg:name}`
8. Preexec nonzero: child stderr in `cause`; recovery not “install binaries”
9. No global `PATH` mutation in parallel tests
10. `cargo fmt --all --check && cargo clippy --workspace -- -D warnings` clean

### Manual (non-gating)

- Skipped-requires message readable when `python3` absent

## Required validation

```bash
cargo test -p wyvern-cli --test extension_diagnostics
cargo test -p wyvern-cli --test preexec_recovery
cargo fmt --all --check && cargo clippy --workspace -- -D warnings
./target/debug/wyvern notes.txt 2>&1 | rg 'unknown input'
./target/debug/wyvern notes.txt 2>&1 | rg -v 'not valid JSON'
```

## Non-closure

- Rich catalog JSON / `show` → **g.3**
- Typo inference → P2
- `PreexecFailureKind::Timeout` → P2 (requires async timeout infra)

## Authority

- [agent-usability-contract.md](agent-usability-contract.md)
- g.1 `format_skill_card` — reuse example lines in recovery text
