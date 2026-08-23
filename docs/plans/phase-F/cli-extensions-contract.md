# CLI Extensions Contract (Phase F)

Authoritative contract for declarative argv → `Command` JSON expansion. Extensions add **no** new host dialog types; they produce validated commands for the existing pipeline.

## Principles

1. Every expansion must pass `wyvern_schema::validate` before `wyvern_host::run`.
2. Extensions may set `HostOptions.ui_root` via templates — Phase F host overrides are **`ui_root` only**.
3. `preexec` runs **only** for declared extension steps. **Phase F trust:** project `.wyvern/extensions.json` may add/override preexec (trusted working tree). User config preexec deferred post-F.
4. Match precedence (first win): built-in subcommands (`browsers`, `extensions`) → extension `match_argv` on argv remainder → inline JSON / `.json` / stdin.

## CLI argv pipeline (normative)

1. Parse argv; route first-token built-ins (`browsers`, `extensions list`) unchanged.
2. Strip **host-only** flags: `--bind`, `--ui-root`, `--viewer`, `--allow-non-loopback`. Leave all other tokens in the **extension remainder**.
3. **Built-ins (step 1, before extension match):** `browsers`, `extensions list`, `--version` / `-V` → early return (unchanged behavior).
4. Call `ExtensionRegistry::match_argv(remainder)` before JSON/usage fallback. **First match wins:** walk merged `extensions` array in merge order; first matching id wins (project override replaces earlier id at same index).
5. When **no** extension matches, `load_command_input` handles a single positional JSON / `.json` path or stdin. Multi-token unmatched remainders are unknown-subcommand usage (structured stderr, exit 2). Extension match is **not** implemented inside `input.rs`.
6. On match: optional preexec → expand → validate → existing pipeline with `ExpandedInvocation.temp_guard` held until host exit.
7. **Host override precedence:** when `ExpandedInvocation.host_overrides.ui_root` is `Some`, it **replaces** CLI `--ui-root`. CLI `--ui-root` applies only when extension does not set `host.ui_root`.

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
| `positional_suffix` | Single positional ends with suffix (`.csv`, `.html`, `.md`) |
| `filename` | Exact basename match (`wizard.json` only — not `ends_with`) |
| `argv_prefix` | First N argv tokens (`["compose", "render"]`, `["md"]`) |
| `arg_suffix` | Token after prefix matches suffix (for `wyvern md file.csv`) |

### `preexec.requires` (match-time)

Lives under `preexec.requires`. After `extends` resolution, if any required binary is absent on `PATH`, the extension **does not match** argv (fallthrough to next rule). `wyvern extensions list` still prints the entry with `(requires: …)`.

### Template substitution (two phases)

**Phase 1 — preexec args** (before subprocess): `{path}`, `{basename}`, `{stem}`, `{parent_dir}`, `{wizard_root}`, `{relpath_from_ui_root}`, `{tmpdir}`, `{wyvern_share}`, `{arg:name}`, `{arg:name:repeat}`.

- `{arg:name:repeat}` splices to **zero or more separate argv tokens** (`--name val` per occurrence). Omit placeholder when flag absent.
- `{arg:name}` — parses `--name VAL` token pair from argv remainder (`--name=VAL` also accepted). Missing required arg → `ExtensionError::MissingArg`. Unknown tokens after successful prefix match → `ExtensionError::UnexpectedArg`.

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

Compose preexec invokes `sc-compose render` with `--output {tmpdir}/pages/page.html` (not legacy `--out` / `--format`). Optional pass-through: `--var`, `--var-file`, `--env-prefix` (via `{arg:*:repeat}` templates).

## CSV HTML table (f.4)

Preexec writes:

```
{tmpdir}/data/rows.json
{tmpdir}/pages/view.html      # copies packaged shell
{tmpdir}/shared/table.js
{tmpdir}/shared/table.css
```

`view.html` loads `../shared/table.js`, `../shared/table.css`, and packaged `/shared/wyvern-api.js` for wizard finish. `table.js` fetches `../data/rows.json`.

Finish uses `wyvernWizardFinish` with the session-derived stack (`window.wyvern.stack` plus current `{ page, data }`) per Phase D wizard contract (REQ-0024 / REQ-0133).

## Phase E consumption (ADR-0022 — Path A)

Extensions are an **argv preprocessor** inside the `wyvern` binary producing existing `Command` JSON. Public API: `wyvern::extensions` module used by `wyvern` CLI and `--interactive` (Phase E).

**MCP (Phase E):** accepts **pre-expanded `Command` JSON** from tool handlers — does **not** call argv expansion. `wyvern-mcp` boundary unchanged (`wyvern-host`, `wyvern-schema` only). Phase E e.3 tools that need CSV/HTML compose the Command JSON in Rust or shell out to `wyvern` CLI for expand-only mode.

f.1 deliverables include `docs/architecture.md` ADR-0022 entry (Path A — no mcp.toml edge change).

## Phase G — agent CLI surfaces

Phase G does not change match/expand semantics above. It adds **in-binary discoverability** requirements (REQ-0134–REQ-0137): global and extension `--help`, skill catalog (`extensions list --json`, `extensions show`), near-miss diagnostics, and registry/help parity tests.

Normative amendment: [agent-usability-contract.md](../phase-G/agent-usability-contract.md). Help surface (g.1): [g1-help-surface.md](../phase-G/g1-help-surface.md) — global `--help` / `-h` / `help` (exit 0) and match-time extension skill cards via `match_extension_help`. Principal REQ text: [docs/wyvern/requirements.md](../../wyvern/requirements.md). ADR-0022 Phase G consequence: new shipped extensions must update help, catalog, and parity tests in the same change.

## Phase H — XHTML report extensions

Phase H adds **`type: "report"`** (ADR-0025). Extensions still use match → preexec → expand → validate; **no new expand template vars**.

| Extension | Match | Expand pattern |
|-----------|-------|----------------|
| `xhtml-suffix` | `.xhtml` suffix | Inline `expand.command` (`type: "report"`, `mode: "view"`) |
| `report-xhtml` | `report-xhtml` + `.json` suffix | **`command_from_file`:** `{tmpdir}/report-command.json` written by preexec |
| `report-xhtml-review` | `report-xhtml --review` + `.json` suffix | Same; preexec `--force-mode review` |

Preexec script `scripts/ext/xhtml_report.py` reads manifest, stitches frame HTML, emits validated command JSON. **`report-xhtml-review` must register before `report-xhtml`** (longer prefix wins).

Contract: [xhtml-reporting-contract.md](../phase-H/xhtml-reporting-contract.md). REQ-0140–0144 land in h.1/h.3.
