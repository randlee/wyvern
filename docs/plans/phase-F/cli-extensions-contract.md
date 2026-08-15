# CLI Extensions Contract (Phase F)

Authoritative contract for declarative argv → `Command` JSON expansion. Extensions add **no** new host dialog types; they produce validated commands for the existing pipeline.

## Principles

1. Every expansion must pass `wyvern_schema::validate` before `wyvern_host::run`.
2. Extensions may set `HostOptions.ui_root` via templates — Phase F host overrides are **`ui_root` only**.
3. `preexec` runs **only** for declared extension steps. **Phase F trust:** project `.wyvern/extensions.json` may add/override preexec (trusted working tree). User config preexec deferred post-F.
4. Match precedence (first win): built-in subcommands (`browsers`, `extensions`) → extension `match_argv` on argv remainder → inline JSON / `.json` / stdin.

## CLI argv pipeline (normative)

1. Parse argv; route first-token built-ins (`browsers`, `extensions list`) unchanged.
2. Strip **host-only** flags: `--bind`, `--ui-root`, `--viewer`, `--allow-non-loopback`, `--version`. Leave all other tokens (including unknown `--*`) in the **extension remainder**.
3. Call `ExtensionRegistry::match_argv(remainder)` before JSON/usage fallback.
4. `load_command_input` must accept multi-token remainders when an extension matches (prefix extensions: `compose render`, `table`, `md`).
5. On match: optional preexec → expand → validate → existing pipeline with `ExpandedInvocation.temp_guard` held until host exit.

Required f.1 tests: `wyvern compose render --root X --file Y` survives parse; `wyvern table f.csv` and `wyvern md f.csv` reach matcher.

## Registry locations (merge order — Phase F)

| Source | Path | Phase F |
|--------|------|---------|
| Shipped defaults | `share/wyvern/extensions.json` | ✓ |
| Project | `.wyvern/extensions.json` | ✓ (trusted preexec) |
| User | `~/.config/wyvern/extensions.json` | **post-F** — not loaded in Phase F |

Later files override earlier by extension `id`.

## Install / embed mapping (normative)

| Source (repo) | Runtime `{wyvern_share}` path |
|---------------|-------------------------------|
| `share/wyvern/extensions.json` | `extensions.json` |
| `scripts/ext/*.py` | `scripts/ext/*.py` |
| `share/wyvern/ext/csv/**` | `ext/csv/**` |

f.1 resolves `{wyvern_share}` via embedded extract beside binary **and** dev-workspace path. Update `crates/wyvern/Cargo.toml` `include` / rust-embed map accordingly.

## Schema (normative)

```json
{
  "version": 1,
  "extensions": [
    {
      "id": "csv-suffix",
      "match": { "positional_suffix": ".csv" },
      "preexec": {
        "cmd": "python3",
        "args": ["{wyvern_share}/scripts/ext/csv_to_view.py", "{path}", "--out", "{tmpdir}", "--format", "html"],
        "requires": ["python3"],
        "stdout": null
      },
      "expand": {
        "command": { "type": "wizard", "page": { "html": "pages/view.html" } },
        "host": { "ui_root": "{tmpdir}" }
      }
    }
  ]
}
```

### Match kinds

| Field | Meaning |
|-------|---------|
| `positional_suffix` | Single positional ends with suffix (`.csv`, `.html`) |
| `argv_prefix` | First N argv tokens (`["compose", "render"]`, `["md"]`) |
| `arg_suffix` | Token after prefix matches suffix (for `wyvern md file.csv`) |

### `preexec.requires` (match-time)

Lives under `preexec.requires`. After `extends` resolution, if any required binary is absent on `PATH`, the extension **does not match** argv (fallthrough to next rule). `wyvern extensions list` still prints the entry with `(requires: …)`.

### Template substitution (two phases)

**Phase 1 — preexec args** (before subprocess): `{path}`, `{basename}`, `{stem}`, `{parent_dir}`, `{wizard_root}`, `{relpath_from_ui_root}`, `{tmpdir}`, `{wyvern_share}`, `{arg:name}`, `{arg:name:repeat}`.

- `{arg:name:repeat}` splices to **zero or more separate argv tokens** (`--name val` per occurrence). Omit placeholder when flag absent.
- Missing required `{arg:name}` (non-repeat) → structured `ExtensionError::MissingArg`.

**Phase 2 — command/host expand** (after preexec): `{preexec.stdout}`, `{rendered_basename}` plus phase-1 vars still available.

`stdout` capture modes (Phase F): `"markdown"` only.

### Template variables

| Var | Description |
|-----|-------------|
| `{path}` | Matched file path (Suffix / PrefixSuffix); **absent** for Prefix-only |
| `{basename}`, `{stem}`, `{parent_dir}` | Path parts; error if `{path}` absent |
| `{wizard_root}` | Directory containing `wizard.json` or `pages/` ancestor (see algorithm below) |
| `{relpath_from_ui_root}` | `{path}` relative to `{wizard_root}` |
| `{tmpdir}` | Secure temp dir; owned by `ExpandedInvocation` until host exit |
| `{wyvern_share}` | Embedded extract root beside binary |
| `{preexec.stdout}` | Captured stdout when `preexec.stdout: "markdown"` |
| `{arg:name}` | Named flag value from argv remainder |
| `{arg:name:repeat}` | Repeatable flag → multiple argv token pairs |
| `{rendered_basename}` | Lexicographically first `*.html` in `{tmpdir}/pages/`; error if none |

### Wizard-root inference (html / wizard.json)

For suffix matches on `.html` or `wizard.json`:

1. Start at matched file's directory.
2. Walk up until a directory contains `wizard.json` **or** a `pages/` subdirectory — that directory is `{wizard_root}`.
3. If none found, use immediate parent of file as `{wizard_root}`.
4. `{relpath_from_ui_root}` = path from `{wizard_root}` to matched file.

Example: `examples/wizards/single-page/pages/only.html` → `{wizard_root}` = `single-page/`, `{relpath_from_ui_root}` = `pages/only.html`, expand `host.ui_root` = `{wizard_root}`, `page.html` = `pages/only.html`.

### Expand fields

| Field | Meaning |
|-------|---------|
| `command` | Inline `Command` JSON after phase-2 substitution |
| `command_from_file` | Load JSON from `{path}`; host overrides still apply |
| `host` | `{ "ui_root": "..." }` only in Phase F |

### Registry inheritance

`"extends": "<id>"` reuses parent `preexec` + `expand`; child `match` replaces parent. `preexec.requires` inherited unless overridden.

## Temp directory lifecycle

`ExpandedInvocation` carries an owned temp-dir handle:

- Created before preexec when `{tmpdir}` referenced.
- **Kept alive through host run** when ui_root = `{tmpdir}`.
- Dropped (and deleted) **after host exit** on success.
- Dropped immediately on preexec non-zero exit (no host launch).

## Subcommand vs suffix

```bash
wyvern report.csv           # positional_suffix .csv
wyvern table report.csv     # argv_prefix ["table"] + arg_suffix .csv
wyvern md report.csv        # argv_prefix ["md"] + arg_suffix .csv → markdown
wyvern compose render --root R --file F.j2   # argv_prefix ["compose","render"]
```

## CSV HTML table (f.4)

Preexec writes:

```
{tmpdir}/data/rows.json
{tmpdir}/pages/view.html      # copies packaged shell
{tmpdir}/shared/table.js
{tmpdir}/shared/table.css
```

`view.html` loads `../shared/table.js`, `../shared/table.css`, and packaged `/shared/wyvern-api.js` for wizard finish. `table.js` fetches `../data/rows.json`.

Finish uses `wyvern-api.js` → `{ button: "finish", data: { row_count: N }, stack: [...] }` per Phase D wizard contract.

## Phase E consumption (ADR-0022)

Extensions are an **argv preprocessor** producing existing `Command` JSON — no new schema variants. Public API: `wyvern::extensions` module (library surface) that `wyvern` binary and Phase E `--interactive` / MCP argv expansion both call. `wyvern-mcp` may depend on `wyvern` library extensions API (ADR-0011 amendment in f.1). MCP tools accept pre-expanded Command JSON or call the shared expand helper — not duplicate registry logic.
