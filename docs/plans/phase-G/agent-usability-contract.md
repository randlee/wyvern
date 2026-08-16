# Agent usability contract (Phase G amendment)

Amends [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md) and ADR-0022 for help surfaces, diagnostics, and skill catalog output. Phase G sprints implement this contract; do not duplicate normative rules in sprint prose without matching edits here.

## Pipeline order (`main.rs`)

After built-ins (`browsers`, `extensions`, `--version`):

1. **Host-flag strip** — unchanged (Phase F).
2. **Global help** — if first token in remainder is `--help`, `-h`, or `help`, print `usage_message()` to stdout, exit 0.
3. **Extension help match** — if remainder matches an extension `argv_prefix` (prefix or prefix+suffix spec) and tokens after prefix are only `--help`/`-h`, print skill card to stdout, exit 0. Runs **before** `requires` skip and **without** requiring `arg_suffix` path token. Implemented by `match_extension_help()` only — not `expand_and_validate`.
4. **Extension match** — `ExtensionRegistry::match_with_diagnostics(remainder)` (CLI). Library `match_argv()` returns `match_with_diagnostics(...).matched` (same semantics as Phase F).
5. **Expand** — `expand_and_validate` → `Result<ExpandedInvocation, ExtensionError>` (no help branch on CLI path).
6. **Near-miss diagnostics** — if step 4 has no match, classify remainder (decision table below) and emit structured stderr JSON; do **not** fall through to inline JSON parse for path-like tokens.
7. **Load fallthrough** — JSON file, stdin, inline JSON (Phase F).

Built-in families handle their own `--help` (`extensions`, `browsers`).

## Skill card formatting (single source of truth)

g.1 stubs `SkillRecord`, `build_skill_record()`, and `format_skill_card()` in `catalog.rs`; g.3 completes fields and adds `build_skill_records()`. Text output for help, list, and show **must** use:

```rust
pub fn format_skill_card(record: &SkillRecord) -> String;
```

g.1 extension help and g.3 list/show call this function after `build_skill_record(ext, probe)`. No second formatter.

## Near-miss decision table (`main.rs`, before `load_command_input`)

| Remainder shape | `NearMissKind` | Example |
|-----------------|----------------|---------|
| Single token starts with `{` or `[` | *(none — inline JSON parse)* | `wyvern '{"type":"message"}'` |
| Path-like token; suffix would match but all candidates skipped for `requires` | `SkippedRequires` | `PATH=/bin wyvern sample.csv` |
| Full `argv_prefix` matched; required suffix path absent or wrong extension | `BarePrefix` | `wyvern md`, `wyvern table`, `wyvern md notes.txt` |
| Prefix head matched; later prefix tokens missing | `IncompletePrefix` | `wyvern compose` → hint `compose render` |
| Otherwise unknown path / token | `UnknownInput` | `wyvern notes.txt` |

Do **not** implement registry match inside `input.rs`. Do **not** parse path-like tokens as inline JSON.

### Variant fields

```rust
pub enum NearMissKind {
    UnknownInput { token: String },
    IncompletePrefix { extension_id: String, hint: String },
    BarePrefix { extension_id: String, usage: String },
    SkippedRequires { path: String, skipped: Vec<SkippedExtension> },
}
```

## Near-miss wire format

Near-misses emit the existing **`StderrError` JSON envelope** on stderr. **No new `ErrorCode` variants** in `wyvern-schema`.

| `NearMissKind` | `error` | `code` | Exit | Notes |
|----------------|---------|--------|------|-------|
| `UnknownInput` | `parse` | `PARSE_ERROR` | 2 | Reuses `ErrorCode::ParseError`; `message`/`recovery` name supported suffixes — **must not** contain `Input was not valid JSON` |
| `SkippedRequires` | `validation` | `VALIDATION_ERROR` | 4 | Names skipped extension id + missing binary + install hint |
| `IncompletePrefix` / `BarePrefix` | `validation` | `VALIDATION_ERROR` | 4 | Names `extension_id` + usage/hint line |

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
| REQ-0130 | After host-flag strip: global help, extension help match, `match_with_diagnostics`, near-miss layer, load fallthrough |
| REQ-0132 | `extensions list` emits skill-index text or `--json` array per skills-catalog-contract |

Cross-link ADR-0022 amendment in `docs/architecture.md` (Phase G subsection).
