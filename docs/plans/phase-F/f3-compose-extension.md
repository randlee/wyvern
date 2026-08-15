---
id: f.3
title: Compose render extension (sc-compose preexec)
status: complete
branch: feature/phase-F-f3-compose-extension
worktree: ../wyvern-worktrees/feature/phase-F-f3-compose-extension
target: integrate/phase-F
---

# Sprint f.3 — `compose render` extension

## Goal

When `sc-compose` is on `PATH`, `wyvern compose render ...` expands to a wizard over rendered HTML. When `sc-compose` is **absent**, the extension does **not** match argv (same as unknown subcommand). `wyvern extensions list` still shows `compose-render` marked `(requires: sc-compose)`.

## Hard dependencies

- f.1 merged to `integrate/phase-F`
- f.2 recommended (shared preexec + ui_root patterns) but not blocking

## Deliverables

### Registry entry

```json
{
  "id": "compose-render",
  "match": { "argv_prefix": ["compose", "render"] },
  "preexec": {
    "cmd": "sc-compose",
    "args": [
      "render", "--root", "{arg:root}", "--file", "{arg:file}",
      "--out", "{tmpdir}/pages", "--format", "html",
      "{arg:var-file:repeat}", "{arg:var:repeat}", "{arg:env:repeat}"
    ],
    "requires": ["sc-compose"]
  },
  "expand": {
    "command": {
      "type": "wizard",
      "page": {
        "id": "compose-preview",
        "title": "Compose preview",
        "html": "pages/{rendered_basename}"
      }
    },
    "host": { "ui_root": "{tmpdir}" }
  }
}
```

Pass-through flags `--var`, `--var-file`, `--env` captured via `{arg:var:repeat}`, `{arg:var-file:repeat}`, `{arg:env:repeat}` (see contract).

### Fixture (`fixtures/compose-minimal/`)

| File | Purpose |
|------|---------|
| `page.j2` | Single-variable template (`{{ title }}`) |
| `vars.json` | `{ "title": "Compose preview" }` |

### Preexec arg mapping

Named `{arg:*}` and `{arg:*:repeat}` capture is owned by f.1 `preexec.rs` (two-phase substitution per contract). f.3 adds registry entry + integration tests only.

| Path | Change |
|------|--------|
| `share/wyvern/extensions.json` | `compose-render` entry |

### Tests

| Path | Change |
|------|--------|
| `crates/wyvern/tests/extensions_compose.rs` | Skip if `sc-compose` absent; expand + mock preexec in unit tests |
| `fixtures/compose-minimal/` | `page.j2` + `vars.json` (see table above) |

### Docs

| Path | Change |
|------|--------|
| `docs/plans/phase-F/README.md` | Compose smoke command |
| `README.md` | Optional section: requires `sc-compose` crate binary |

## Acceptance criteria

1. With `sc-compose` on PATH: `wyvern compose render --root ./fixtures/compose-minimal --file page.j2 --var-file vars.json` opens wizard preview; expand asserts `page.html` = `pages/page.html`
2. Without `sc-compose` on PATH: `wyvern compose render ...` does **not** match any extension → standard unknown-subcommand usage error (exit 2); `wyvern extensions list` shows `compose-render (requires: sc-compose)`
3. Preexec failure (non-zero exit) → CLI error with stderr snippet, no host launch
4. `--var-file` forwarded to sc-compose preexec args when present
5. No new Rust dependency on sc-compose crate — external binary only

## Required validation

```bash
cargo test -p wyvern-cli extensions_compose
# if sc-compose present locally:
wyvern compose render --root fixtures/compose-minimal --file page.j2
```

## Non-closure

- Full sc-compose site multi-page export (single rendered file sufficient for f.3)
- Winget/npm install of sc-compose

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- sc-compose CLI: `sc-compose render --help` (external)
