# Agent usability contract (Phase G amendment)

Amends [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md) and ADR-0022 for help surfaces, diagnostics, and skill catalog output. Phase G sprints implement this contract; do not duplicate normative rules in sprint prose without matching edits here.

## Pipeline order (`main.rs`)

After built-ins (`browsers`, `extensions`, `--version`):

1. **Global help** — if first positional token is `--help`, `-h`, or `help` (host flags may precede `--help` only when `--help` is the first positional after strip), print `usage_message()` to stdout, exit 0.
2. **Host-flag strip** — unchanged (Phase F).
3. **Extension help match** — if remainder matches an extension `argv_prefix` (prefix or prefix+suffix spec) and tokens after prefix are only `--help`/`-h`, print skill card to stdout, exit 0. Runs **before** `requires` skip and **without** requiring `arg_suffix` path token.
4. **Extension match** — `ExtensionRegistry::match_with_diagnostics(remainder)` (CLI). Library `match_argv()` remains silent first-match API returning `Option<ExtensionMatch>` (implemented as `match_with_diagnostics(...).matched` or thin wrapper — same semantics as Phase F).
5. **Expand** — `expand_and_validate` returns `ExpandOutcome` (see below).
6. **Near-miss diagnostics** — if step 4 has no match, classify remainder (decision table below) and emit structured stderr JSON; do **not** fall through to inline JSON parse for path-like tokens.
7. **Load fallthrough** — JSON file, stdin, inline JSON (Phase F).

Built-in families handle their own `--help` (`extensions`, `browsers`).

## Help outcome (not an error)

```rust
pub enum ExpandOutcome {
    Expanded(ExpandedInvocation),
    Help { text: String },
}

pub fn expand_and_validate(
    ext: &ExtensionDef,
    ctx: &MatchContext,
) -> Result<ExpandOutcome, ExtensionError>;
```

`main.rs` prints `Help.text` to stdout and exits 0. `ExtensionError` and `error.rs` remain failure-only.

## Skill card formatting (single source of truth)

g.3 owns `SkillRecord` + `build_skill_record()`. Text output for help, list, and show **must** use:

```rust
pub fn format_skill_card(record: &SkillRecord) -> String;
```

g.1 extension help and g.3 list/show call this function after `build_skill_record(ext, probe)`. No second formatter.

## Near-miss decision table (`main.rs`, before `load_command_input`)

| Remainder shape | Action |
|-----------------|--------|
| Single token starts with `{` or `[` | Inline JSON parse (Phase F) |
| Path-like token, registry suffix would match but all candidates skipped for `requires` | `NearMissKind::SkippedRequires` |
| Token matches known `argv_prefix` head, incomplete | `IncompletePrefix` or `BarePrefix` |
| Prefix+suffix extension, suffix token wrong (e.g. `wyvern md notes.txt`) | `BarePrefix` with csv-md usage |
| Otherwise unknown path / token | `UnknownInput` |

Do **not** implement registry match inside `input.rs`. Do **not** parse path-like tokens as inline JSON.

## Near-miss wire format

Near-misses emit the existing **`StderrError` JSON envelope** on stderr (same as validation/usage errors today).

| `NearMissKind` | `error` | `code` | Exit | Notes |
|----------------|---------|--------|------|-------|
| `UnknownInput` | `usage` | `USAGE_ERROR` | 2 | `recovery` lists supported suffixes + `wyvern extensions list` |
| `SkippedRequires` | `validation` | `VALIDATION_ERROR` | 4 | Names skipped extension id + missing binary + install hint |
| `IncompletePrefix` / `BarePrefix` | `validation` | `VALIDATION_ERROR` | 4 | Names extension id + usage line |

No new `ErrorCode` in `wyvern-schema` for g.2 (reuse existing codes; agents branch on `message`/`recovery`, not legacy misleading parse text).

## Extension arg errors

Replace singular `ExtensionError::MissingArg` with:

```rust
MissingArgs {
    missing: Vec<String>,
    declared: BTreeSet<String>,
    extension_id: String,
    example: String,
}
```

`UnexpectedArg` recovery is caller-facing (lists accepted flags); never registry-author text.

## Registry optional fields

Per extension object in merged registry JSON:

- `description: string` (optional, recommended)
- `examples: string[]` (optional, recommended)

Parsed by `ExtensionDef`; consumed by `SkillRecord` builder.

## Catalog JSON wire format

`wyvern extensions list --json` prints a **JSON array** of `SkillRecord` objects to stdout (no wrapper object).

`wyvern extensions show <id> --json` prints one `SkillRecord` object.

See [skills-catalog-contract.md](skills-catalog-contract.md).

## REQ amendments (implement in owning sprint)

| REQ | Amendment |
|-----|-----------|
| REQ-0130 | After host-flag strip: extension help match, then `match_with_diagnostics`, then near-miss layer, then load fallthrough |
| REQ-0132 | `extensions list` emits skill-index text or `--json` array per skills-catalog-contract |

Cross-link ADR-0022 amendment in `docs/architecture.md` (Phase G subsection).
