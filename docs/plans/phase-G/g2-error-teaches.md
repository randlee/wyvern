---
id: g.2
title: Error-teaches — near-miss diagnostics and preexec recovery
status: complete
branch: feature/phase-G-g2-error-teaches
worktree: ../wyvern-worktrees/feature/phase-G-g2-error-teaches
target: integrate/phase-G
---

# Sprint g.2 — Error-teaches

## Goal

Replace misleading `PARSE_ERROR` / generic usage fallthrough with structured `StderrError` JSON that names the skill and next argv. Preexec failures classify spawn-not-found vs nonzero-exit; recovery must not say “install binaries” when the binary ran.

## Hard dependencies

- **g.1** merged (global/extension help pipeline, `format_skill_card`, `match_extension_help`, catalog.rs stub). **g.2** introduces `match_with_diagnostics()` and refactors the `main.rs` match path — not a g.1 deliverable.
- [agent-usability-contract.md](agent-usability-contract.md) — near-miss table + wire format

## Deliverables

| Path | Change |
|------|--------|
| `docs/architecture.md` | Reference only — ADR-0022 Phase G amendment landed on plan branch |
| `docs/wyvern/architecture.md` | Reference only — near-miss pipeline rows landed on plan branch |
| `crates/wyvern/src/extensions/mod.rs` | `match_with_diagnostics()` → `MatchOutcome { matched, skipped }`; `match_argv()` wraps `.matched` only |
| `crates/wyvern/src/extensions/diagnostics.rs` | **New.** `NearMissKind` → `StderrError` envelope per contract table |
| `crates/wyvern/src/main.rs` | Near-miss decision table before `load_command_input` (no registry logic in `input.rs`) |
| `crates/wyvern/src/extensions/expand/mod.rs` | `ExtensionError::MissingArgs { … }` replaces `MissingArg`; list all missing required flags; submodules `env.rs`, `template.rs`, `preexec_orchestration.rs` |
| `crates/wyvern/src/extensions/preexec.rs` | Pipe stderr (4 KiB tail); `PreexecFailureKind` including sync-poll `Timeout` (`WYVERN_PREEXEC_TIMEOUT_SECS`) |
| `crates/wyvern/src/error/mod.rs` | `CliError` / `LoadError` types; `ExtensionError` recovery fields including `help_command` |
| `crates/wyvern/src/error/emit.rs` | Caller-facing `UnexpectedArg` / `MissingArgs` / `Preexec` recovery; in-binary recovery first |
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
    IncompletePrefix { extension_id: String, hint: String },
    BarePrefix { extension_id: String, usage: String },
    SkippedRequires { path: String, skipped: Vec<SkippedExtension> },
}

pub fn emit_near_miss(kind: &NearMissKind) -> Result<String, EmitError>; // StderrError JSON

pub enum PreexecFailureKind {
    SpawnNotFound { cmd: String },
    NonZeroExit { code: i32, stderr_tail: String },
    Timeout { cmd: String, timeout_secs: u64 },
}

pub enum ExtensionError {
    MissingArgs {
        missing: Vec<String>,
        declared: BTreeSet<String>,
        extension_id: String,
        example: String,
        help_command: String, // invocation-prefix help, e.g. `wyvern compose render --help`
    },
    // …
}
```

### Near-miss wire (normative)

| Kind | `code` | Exit |
|------|--------|------|
| `UnknownInput` | `PARSE_ERROR` | 2 |
| `SkippedRequires`, `IncompletePrefix`, `BarePrefix` | `VALIDATION_ERROR` | 4 |

`UnknownInput` uses `ErrorCode::ParseError` with agent-facing `message`/`recovery` — must not contain `Input was not valid JSON`.

### Paths to delete

None.

## Acceptance criteria

### Automated

1. `cargo test -p wyvern-cli --test extension_diagnostics` passes near-miss table cases
2. `cargo test -p wyvern-cli --test preexec_recovery` passes spawn vs nonzero-exit branches
3. `wyvern notes.txt` → `PARSE_ERROR` with `unknown input` in message; must not contain `Input was not valid JSON`
4. Stub probe csv skipped → names `csv-suffix`, `python3`, example argv
5. `wyvern md` → csv-md `BarePrefix` usage with `<file.csv>`
6. `wyvern compose` → `IncompletePrefix` naming `compose-render` and `compose render`
7. `wyvern compose render` (no flags) → `MissingArgs` lists `--root` and `--file` in one envelope
8. `UnexpectedArg` recovery never contains `declare them as {arg:name}`
9. Preexec nonzero: child stderr in `cause`; recovery not “install binaries”
10. `wyvern md /nonexistent/file.csv` → structured preexec/IO envelope with child stderr in `cause`; recovery must not recommend binary install when `python3` ran
11. Subprocess tests use isolated temp dirs and per-command env overrides — no global `PATH` mutation in parallel tests
13. `cargo fmt --all --check && cargo clippy --workspace -- -D warnings` clean
14. Implements **REQ-0130** (near-miss layer) and **REQ-0136** per [docs/wyvern/requirements.md](../../wyvern/requirements.md)

### Manual (non-gating)

- Skipped-requires message readable when `python3` absent

## Required validation

```bash
cargo test -p wyvern-cli --test extension_diagnostics
cargo test -p wyvern-cli --test preexec_recovery
cargo fmt --all --check && cargo clippy --workspace -- -D warnings
./target/debug/wyvern notes.txt 2>&1 | rg 'unknown input'
./target/debug/wyvern notes.txt 2>&1 | rg -v 'not valid JSON'
./target/debug/wyvern compose 2>&1 | rg 'compose render'
./target/debug/wyvern md /nonexistent/file.csv 2>&1 | rg -v 'install.*binary'
```

## Non-closure

- Rich catalog JSON / `show` → **g.3**
- Typo inference → P2
- Async/event-loop preexec timeout infrastructure → P2 (`PreexecFailureKind::Timeout` via sync poll and `WYVERN_PREEXEC_TIMEOUT_SECS` recovery is in-scope for g.2)

## Phase-end host exception (RSH-007)

g.2 itself does not change `wyvern-host`. Phase-end integration may include minimal host hardening for the session-timeout / result-token race (**RSH-007**) so preexec recovery tests complete reliably instead of racing `POST /api/result` against shutdown. Landed in `dc4eaae`. See [phase-G README Boundaries](README.md#boundaries).

## Authority

- [agent-usability-contract.md](agent-usability-contract.md)
- g.1 `format_skill_card` — reuse example lines in recovery text
