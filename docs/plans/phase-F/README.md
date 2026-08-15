# Phase F — CLI Extensions (`integrate/phase-F`)

Phase F implementation PRs target **`integrate/phase-F`**. Sprint docs (`f.1`–`f.4`) are the **sole authority** for deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals only.

**Ordering:** Phase F runs **before Phase E**. Phase E `--interactive` reuses `wyvern::extensions` inside the `wyvern` binary. MCP tools use pre-expanded Command JSON (ADR-0022 Path A).

## Core model

**Extensions expand argv → validated `Command` JSON + optional `HostOptions`.** No per-extension Rust after the runtime lands in f.1.

```
argv → match extension → optional preexec → template expand → validate → pipeline → host
```

| Layer | Responsibility |
|-------|----------------|
| **`wyvern` CLI** | Extension registry load, match, preexec, expand, `extensions list` |
| **`wyvern-schema`** | Unchanged — validates expanded command only |
| **`wyvern-host`** | Unchanged — serves wizard/markdown/etc. |
| **Bundled scripts/assets** | `scripts/ext/*.py`, `share/wyvern/ext/csv/*` — extension preexec + page JS |

See [cli-extensions-contract.md](cli-extensions-contract.md) for schema and match precedence.

## Phase goal

Declarative CLI extensions: file suffix defaults and optional subcommand aliases, without new dialog types. Shipped pack includes `.html`, `.csv` (interactive HTML table), conditional `compose render`, and `md` for CSV→markdown.

## Phase acceptance (smoke)

```bash
# HTML file (suffix) — f.2 wizard-root inference
wyvern ./examples/wizards/single-page/pages/only.html
# expands to ui_root=single-page/, page.html=pages/only.html

# CSV interactive table (suffix) — sort column, filter, Finish → JSON
wyvern ./fixtures/sample.csv

# CSV markdown variant (subcommand)
wyvern md ./fixtures/sample.csv

# Compose (when sc-compose on PATH)
wyvern compose render --root ./templates --file page.j2 --var-file vars.json
```

All exit 0 with embedded viewer; expanded commands pass existing validation.

## Sprint map

| Sprint | Adds | Not new host behavior |
|--------|------|------------------------|
| **f.1** | Extension runtime + registry + `.md` migration + `extensions list` | ✓ CLI only |
| **f.2** | `.html`, `wizard.json` suffix; `--help` examples | ✓ expands to wizard |
| **f.3** | `compose render` extension (`requires: sc-compose`) | ✓ preexec + wizard |
| **f.4** | `.csv` / `table` HTML viewer (JS DOM, sort/filter) + `md` CSV→markdown | ✓ preexec + wizard/markdown |

## What Phase F does not close

- MCP tool wrappers — **Phase E** (calls `wyvern::extensions` API per ADR-0022)
- `--interactive` argv expansion — **Phase E**
- User registry (`~/.config/wyvern/extensions.json`) — post-F
- User-authored unsigned `preexec` outside trusted project tree (post-F)
- Multi-page sc-compose site generator (defer)

## Boundaries

- Extension runtime lives in **`wyvern` CLI crate** with public `wyvern::extensions` module — not `wyvern-host` or `wyvern-wizard`
- Project `.wyvern/extensions.json` is trusted for preexec (working-tree policy)
- `preexec` must not mutate process env used by parallel tests (inject cwd/temp paths only)
- No new `Command` enum variants in Phase F

## Sprint index

| Sprint | Doc |
|--------|-----|
| f.1 | [f1-extension-runtime.md](f1-extension-runtime.md) |
| f.2 | [f2-positional-extensions.md](f2-positional-extensions.md) |
| f.3 | [f3-compose-extension.md](f3-compose-extension.md) |
| f.4 | [f4-csv-table-viewer.md](f4-csv-table-viewer.md) |
