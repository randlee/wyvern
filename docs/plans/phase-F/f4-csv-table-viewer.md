---
id: f.4
title: CSV table viewer — JS DOM, sort/filter, md alias
status: planning
branch: feature/phase-F-f4-csv-table-viewer
worktree: ../wyvern-worktrees/feature/phase-F-f4-csv-table-viewer
target: integrate/phase-F
---

# Sprint f.4 — CSV table viewer

## Goal

`wyvern report.csv` and `wyvern table report.csv` open an interactive HTML table wizard. Preexec writes JSON + staged static assets; **table DOM built in JavaScript** via fetch. `wyvern md report.csv` renders markdown pipe table via preexec.

## Hard dependencies

- f.1 merged to `integrate/phase-F`
- f.2 patterns for wizard page layout

## Deliverables

### Preexec script

| Path | Change |
|------|--------|
| `scripts/ext/csv_to_view.py` | Read CSV → write tmpdir layout (see contract); `--format html` or `markdown` |
| | Row cap default 10_000 with `truncated: true` in JSON metadata |

**Tmpdir layout (required):**

```
{tmpdir}/data/rows.json
{tmpdir}/pages/view.html       # from share/wyvern/ext/csv/pages/view.html
{tmpdir}/shared/table.js       # from share/wyvern/ext/csv/shared/
{tmpdir}/shared/table.css
```

### Static assets (packaged source)

| Path | Change |
|------|--------|
| `share/wyvern/ext/csv/pages/view.html` | Shell: `../shared/table.css`, `../shared/table.js`, `/shared/wyvern-api.js` |
| `share/wyvern/ext/csv/shared/table.js` | `fetch('../data/rows.json')` → build `<table>` in DOM |
| `share/wyvern/ext/csv/shared/table.css` | Zebra rows, hover, sticky header, filter row UI |

**`table.js` behavior (in scope):**

- Sortable column headers (click toggles asc/desc)
- Per-column text filter inputs (debounced)
- Global search box
- Sticky header on scroll
- Truncation banner when `meta.truncated`
- Finish via `wyvern-api.js`: `{ button: "finish", data: { row_count: N }, stack: [...] }` per Phase D

No external JS libraries (vanilla DOM).

### Registry entries

```json
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
      "width": 960,
      "height": 640
    },
    "host": { "ui_root": "{tmpdir}" }
  }
},
{
  "id": "csv-table-alias",
  "match": { "argv_prefix": ["table"], "arg_suffix": ".csv" },
  "extends": "csv-suffix"
},
{
  "id": "csv-md",
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
```

### Fixtures + tests

| Path | Change |
|------|--------|
| `fixtures/sample.csv` | Small dataset for tests |
| `crates/wyvern/tests/extensions_csv.rs` | Expand + preexec layout; `extensions_csv_requires_python3` with `RequiresProbe` stub |
| `scripts/ext/test_csv_to_view.py` | JSON shape + staged-file existence (not JS runtime DOM) |

## Acceptance criteria

### Automated

1. Preexec produces complete tmpdir layout; every path referenced by `view.html` exists
2. `python3 -m pytest scripts/ext/test_csv_to_view.py` passes (JSON shape + staged-file layout only)
3. `cargo test -p wyvern extensions_csv` passes expand + layout gates
4. `wyvern md fixtures/sample.csv` expand → valid markdown command JSON
5. `wyvern table fixtures/sample.csv` expand identical to suffix form
6. Requires-check: injected stub reports python3 absent → csv-suffix does not match
7. No new host dialog type; wizard + markdown only

### Manual (non-gating)

- Interactive table: sort column, filter rows, truncation banner, Finish (embedded viewer smoke)

## Required validation

```bash
python3 scripts/ext/csv_to_view.py fixtures/sample.csv --out /tmp/csv-test --format html
test -f /tmp/csv-test/data/rows.json && test -f /tmp/csv-test/shared/table.js
python3 -m pytest scripts/ext/test_csv_to_view.py
cargo test -p wyvern extensions_csv
cargo test -p wyvern extensions_csv_requires_python3  # uses injected RequiresProbe stub, not PATH=
cargo fmt --all --check && cargo clippy --workspace -- -D warnings
```

## Non-closure

- Excel `.xlsx`, Parquet, SQL query UI
- Server-side pagination for million-row files
- MCP `show_csv` tool wrapper (Phase E — pre-expanded Command JSON or wyvern CLI subprocess)
- Embedded viewer manual smoke (listed above)

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- Wizard finish contract ([d1-wizard-host.md](../phase-D/d1-wizard-host.md))
