---
id: f.3
title: Compose render extension (sc-compose preexec)
status: planning
branch: feature/phase-F-f3-compose-extension
worktree: ../wyvern-worktrees/feature/phase-F-f3-compose-extension
target: integrate/phase-F
---

# Sprint f.3 — `compose render` extension

## Goal

When `sc-compose` is on `PATH`, `wyvern compose render ...` expands to a wizard over rendered HTML. Extension is **hidden** when dependency missing (no error on `extensions list` for missing tool — entry marked unavailable).

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
    "args": ["render", "--root", "{arg:root}", "--file", "{arg:file}", "--out", "{tmpdir}/pages", "--format", "html"],
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

Pass-through flags (`--var`, `--var-file`, `--env`) forwarded via preexec arg template table (document in contract).

### Preexec arg mapping

| Path | Change |
|------|--------|
| `crates/wyvern/src/extensions/preexec.rs` | Named arg capture from remainder of argv after prefix |
| `share/wyvern/extensions.json` | `compose-render` entry |

### Tests

| Path | Change |
|------|--------|
| `crates/wyvern/tests/extensions_compose.rs` | Skip if `sc-compose` absent; expand + mock preexec in unit tests |
| `fixtures/compose-minimal/` | Tiny `--root` + one `.j2` for CI when sc-compose installed |

### Docs

| Path | Change |
|------|--------|
| `docs/plans/phase-F/README.md` | Compose smoke command |
| `README.md` | Optional section: requires `sc-compose` crate binary |

## Acceptance criteria

1. With `sc-compose` on PATH: `wyvern compose render --root ./fixtures/compose-minimal --file page.j2` opens wizard preview
2. Without `sc-compose`: `wyvern compose render ...` → structured CLI error `ExtensionError::RequiresBinary { name: "sc-compose" }` with exit code 2 (same as unknown subcommand); extension hidden from default match but explicit invocation surfaces requirement
3. `wyvern extensions list` shows `compose-render` with `(requires: sc-compose)` status
4. Preexec failure (non-zero exit) → CLI error with stderr snippet, no host launch
5. No new Rust dependency on sc-compose crate — external binary only

## Required validation

```bash
cargo test -p wyvern extensions_compose
# if sc-compose present locally:
wyvern compose render --root fixtures/compose-minimal --file page.j2
```

## Non-closure

- Full sc-compose site multi-page export (single rendered file sufficient for f.3)
- Winget/npm install of sc-compose

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- sc-compose CLI: `sc-compose render --help` (external)
