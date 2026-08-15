# CLI Extensions Contract (Phase F)

Authoritative contract for declarative argv → `Command` JSON expansion. Extensions add **no** new host dialog types; they produce validated commands for the existing pipeline.

## Principles

1. Every expansion must pass `wyvern_schema::validate` before `wyvern_host::run`.
2. Extensions may set `HostOptions` fields (`ui_root`, etc.) via templates — not new host routes.
3. `preexec` runs **only** for declared extension steps; user override files cannot add arbitrary shell unless explicitly enabled (future: `extensions.local.json` trusted path).
4. Match precedence (first win): built-in flags → exact `argv_prefix` → single positional suffix → inline JSON / `.json` / stdin.

## Registry locations (merge order)

| Source | Path |
|--------|------|
| Shipped defaults | `share/wyvern/extensions.json` (embedded + extracted beside binary) |
| Project | `.wyvern/extensions.json` |
| User | `~/.config/wyvern/extensions.json` (optional v1) |

Later files override earlier by extension `id`.

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
        "requires": ["python3"]
      },
      "expand": {
        "command": {
          "type": "wizard",
          "page": { "id": "{stem}", "title": "{basename}", "html": "pages/view.html", "layout": "workspace" },
          "config": { "estimated_size": { "width": 960, "height": 640 } }
        },
        "host": { "ui_root": "{tmpdir}" }
      }
    },
    {
      "id": "md-csv",
      "match": { "argv_prefix": ["md"], "arg_suffix": ".csv" },
      "preexec": {
        "cmd": "python3",
        "args": ["{wyvern_share}/scripts/ext/csv_to_view.py", "{path}", "--out", "{tmpdir}", "--format", "markdown"],
        "requires": ["python3"],
        "stdout": "markdown"
      },
      "expand": {
        "command": { "type": "markdown", "title": "{basename}", "content": "{preexec.stdout}" }
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
| `requires` | All binaries must exist on `PATH` or extension is **excluded from argv match**; `extensions list` shows entry with `(requires: …)` status |

### Template variables

| Var | Description |
|-----|-------------|
| `{path}` | Matched file path (positional or after prefix) |
| `{basename}`, `{stem}`, `{parent_dir}` | Path parts |
| `{relpath_from_ui_root}` | Path of matched file relative to `{parent_dir}` (ui_root); e.g. `pages/only.html` when ui_root is wizard fixture dir |
| `{tmpdir}` | Secure temp dir (auto-created, deleted after run) |
| `{wyvern_share}` | `share/wyvern` beside binary / embedded extract root |
| `{preexec.stdout}` | Captured stdout from preexec when `stdout: "markdown"` (or other mode) set on preexec |
| `{arg:name}` | Named flag value from argv remainder after prefix match (e.g. `{arg:root}`, `{arg:file}`) |
| `{arg:name:repeat}` | Repeatable flag capture — emits `--name value` pairs for each occurrence (e.g. `{arg:var-file:repeat}` → `--var-file vars.json`) |
| `{rendered_basename}` | Basename of primary rendered HTML file under `{tmpdir}/pages` after preexec |

### Expand fields

| Field | Meaning |
|-------|---------|
| `command` | Inline `Command` JSON object after template substitution |
| `command_from_file` | Load JSON command from `{path}` (wizard.json suffix); host overrides still apply |
| `host` | `HostOptions` overrides (`ui_root`, etc.) |

### Registry inheritance

Extensions may declare `"extends": "<id>"` to reuse another extension's `preexec` + `expand` block; child `match` replaces parent. Implemented in f.1 loader merge — no duplicate expand blocks required.

## Subcommand vs suffix

Both are the same engine:

```bash
wyvern report.csv           # positional_suffix .csv
wyvern table report.csv     # argv_prefix ["table"] (alias)
wyvern md report.csv        # argv_prefix ["md"] + arg_suffix .csv → markdown
```

## CSV HTML table (f.4)

Preexec writes `{tmpdir}/data/rows.json` and copies static `pages/view.html` shell into `{tmpdir}/pages/`. `table.js` fetches `../data/rows.json` and **builds DOM in JS**: sortable columns, per-column filters, global search, sticky header, row-cap banner.

No new Rust in host; wizard + packaged static assets only.
