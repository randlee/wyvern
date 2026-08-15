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

`wyvern report.csv` and `wyvern table report.csv` open an interactive HTML table wizard. Preexec writes JSON to `{tmpdir}/data/rows.json`; **the table DOM is built in JavaScript** via `fetch` (not server-side HTML strings). `wyvern md report.csv` renders markdown pipe table via preexec.

## Hard dependencies

- f.1 merged to `integrate/phase-F`
- f.2 patterns for ui_root + wizard page layout

## Deliverables

### Preexec script

| Path | Change |
|------|--------|
| `scripts/ext/csv_to_view.py` | Read CSV → `{tmpdir}/data/rows.json` + copy static `pages/view.html` shell |
| | `--format html` (default) or `markdown` |
| | Row cap default 10_000 with `truncated: true` in JSON metadata |

**`rows.json` shape:**

```json
{
  "columns": ["Name", "Score"],
  "rows": [["Alice", "98"], ["Bob", "87"]],
  "meta": { "source": "report.csv", "truncated": false, "total_rows": 2 }
}
```

### Static assets (JS DOM builder)

| Path | Change |
|------|--------|
| `share/wyvern/ext/csv/pages/view.html` | Static shell: loads `../shared/table.css`, `table.js`; no inline data |
| `share/wyvern/ext/csv/shared/table.js` | `fetch('../data/rows.json')` → parse JSON → build `<table>` in DOM |
| `share/wyvern/ext/csv/shared/table.css` | Zebra rows, hover, sticky header, filter row UI |

**`table.js` behavior (in scope):**

- Sortable column headers (click toggles asc/desc)
- Per-column text filter inputs (debounced)
- Global search box
- Sticky header on scroll
- Truncation banner when `meta.truncated`
- Finish button → wizard `postResult` JSON: `{ "action": "finish", "values": { "row_count": N } }` where N = displayed row count

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
      "config": { "estimated_size": { "width": 960, "height": 640 } }
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

(`csv-table-alias` uses `"extends": "csv-suffix"` per contract inheritance rule.)

### Fixtures + tests

| Path | Change |
|------|--------|
| `fixtures/sample.csv` | Small dataset for tests |
| `crates/wyvern/tests/extensions_csv.rs` | Expand + preexec output structure; `extensions_csv_requires_python3` when PATH lacks python3 |
| `scripts/ext/test_csv_to_view.py` | pytest or stdlib unittest for JSON shape |

Host L1 optional: headless wizard load with `--viewer none` if harness supports CSV fixture path.

## Acceptance criteria

1. `wyvern fixtures/sample.csv` opens wizard; table visible; column sort changes order
2. Column filter + global search reduce visible rows without reload
3. Truncated CSV (> cap) shows banner; table shows first N rows only
4. `wyvern table fixtures/sample.csv` identical to suffix form
5. `wyvern md fixtures/sample.csv` opens markdown dialog with pipe table
6. Finish returns exit 0; JSON result includes row metadata
7. Python 3 required for CSV extensions; when `python3` absent, extension does not match (requires-check) and `extensions list` shows `(requires: python3)` — same pattern as sc-compose
8. No new host dialog type; wizard + markdown only

## Required validation

```bash
python3 scripts/ext/csv_to_view.py fixtures/sample.csv --out /tmp/csv-test --format html
test -f /tmp/csv-test/data/rows.json
python3 -m pytest scripts/ext/test_csv_to_view.py
cargo test -p wyvern extensions_csv
# requires-check when python3 absent (mock PATH):
PATH=/usr/bin:/bin cargo test -p wyvern extensions_csv_requires_python3
cargo fmt --all --check && cargo clippy --workspace -- -D warnings
```

Manual: open `fixtures/sample.csv`, sort/filter, Finish.

## Non-closure

- Excel `.xlsx`, Parquet, SQL query UI
- Server-side pagination for million-row files
- MCP `show_csv` tool wrapper (Phase E)

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- Wizard finish contract ([d1-wizard-host.md](../phase-D/d1-wizard-host.md))
