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
        "requires": []
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
      "preexec": { "cmd": "python3", "args": ["...", "--format", "markdown"], "stdout": "markdown" },
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
| `requires` | All binaries must exist on `PATH` or extension is hidden |

### Template variables

| Var | Description |
|-----|-------------|
| `{path}` | Matched file path (positional or after prefix) |
| `{basename}`, `{stem}`, `{parent_dir}` | Path parts |
| `{tmpdir}` | Secure temp dir (auto-created, deleted after run) |
| `{wyvern_share}` | `share/wyvern` beside binary / embedded extract root |
| `{preexec.stdout}` | Captured stdout from preexec when `stdout` mode set |

## Subcommand vs suffix

Both are the same engine:

```bash
wyvern report.csv           # positional_suffix .csv
wyvern table report.csv     # argv_prefix ["table"] (alias)
wyvern md report.csv        # argv_prefix ["md"] + arg_suffix .csv → markdown
```

## CSV HTML table (f.4)

Preexec writes `{tmpdir}/pages/view.html` + `{tmpdir}/data/rows.json`. Page loads JSON and **builds DOM in JS** (`share/wyvern/ext/csv/table.js`): sortable columns, per-column filters, global search, sticky header, row-cap banner.

No new Rust in host; wizard + packaged static assets only.
