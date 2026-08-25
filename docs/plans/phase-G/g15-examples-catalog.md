---
id: g.15
title: Examples catalog — `wyvern examples list`
status: complete
branch: feature/phase-G-g15-examples-catalog
worktree: ../wyvern-worktrees/feature/phase-G-g15-examples-catalog
target: develop
---

# Sprint g.15 — Examples catalog (progressive discovery)

## Goal

Ship **`wyvern examples list`** so agents and users can discover bundled examples
without reading checkout docs or opening `wyvern guide`. Discovery is **filesystem-driven**:
each example is a `README.md` under `{wyvern_share}/examples/` with mandatory YAML
frontmatter (`name`, `description`). New examples appear automatically when a README
is added — no registry edit.

Complements Wave 1 **`wyvern extensions list`** (argv skills) and Wave 2 **`wyvern guide`**
(welcome hub + example bridges).

## Hard dependencies

- Phase G Wave 1 merged (`--help`, `extensions list`) — REQ-0134–0137
- Phase G Wave 2 merged (`wyvern guide`, bundled examples under `share/wyvern/examples/`)
- `{wyvern_share}` resolution (Phase F)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g15-examples-catalog.md` | This sprint doc |
| `docs/wyvern/requirements.md` | **REQ-0146** — examples catalog command |
| `crates/wyvern/src/examples/mod.rs` | Discover README frontmatter under `examples/` |
| `crates/wyvern/src/examples/frontmatter.rs` | Parse mandatory `name` + `description` |
| `crates/wyvern/src/examples_cmd.rs` | `wyvern examples [list] [--json]` |
| `crates/wyvern/src/main.rs` | Early return for `examples` subcommand |
| `crates/wyvern/src/lib.rs` | Export `examples` module + command |
| `crates/wyvern/src/cli_args.rs` | Global `--help` mentions `wyvern examples list` |
| `crates/wyvern/src/error/mod.rs` | `BuiltinDomain::Examples` |
| `crates/wyvern/src/error/emit.rs` | Usage recovery for unknown `examples` subcommands |
| `share/wyvern/examples/*/README.md` | Frontmatter on every shipped example |
| `crates/wyvern/share/wyvern/examples/` | Packaged parity (share-sync) |
| `crates/wyvern/tests/examples_catalog.rs` | Integration tests |
| `crates/wyvern/tests/help_surface.rs` | Global help mentions examples |

### REQ traceability (g.15 lands)

| REQ | Summary |
|-----|---------|
| REQ-0146 | `wyvern examples list` discovers bundled examples from README frontmatter; `--json` emits `{name, description, readme}` records |

### Discovery rules (normative)

Scan `{wyvern_share}/examples/`:

1. **`examples/README.md`** — optional base-folder README when one doc covers multiple related examples.
2. **`examples/<dir>/README.md`** — per-example README (typical case).

Each README **must** begin with:

```yaml
---
name: Human-readable title
description: One-line summary for catalog output
---
```

Records include `readme` path relative to `{wyvern_share}` (e.g. `examples/agent-dag/README.md`).
Examples without valid frontmatter are **skipped** (not errors). I/O errors reading the examples tree fail the command.

### CLI surface

```
wyvern examples [list] [--json]
wyvern examples --help
```

**Text output** (one block per example):

```
Agent DAG
Configure agents on a canvas and export data.dag; execution is deferred.
README: examples/agent-dag/README.md
```

**JSON output** (`--json`): array of `{ "name", "description", "readme" }`.

Global `wyvern --help` lists `wyvern examples list` alongside `wyvern extensions list`.

## Acceptance criteria

1. `wyvern examples list` exits **0** and prints every shipped example with frontmatter.
2. `wyvern examples list --json` exits **0**; stdout is a JSON array with required keys.
3. Bare `wyvern examples` matches `wyvern examples list`.
4. `wyvern examples --help` documents frontmatter contract and exits **0**.
5. `wyvern --help` mentions `wyvern examples list`.
6. Adding a new `examples/<slug>/README.md` with frontmatter appears in list without code changes.
7. `scripts/check-share-sync.sh` passes.
8. `cargo test -p wyvern-cli examples_catalog help_surface` passes; clippy clean.

## Validation

```bash
cargo test -p wyvern-cli examples_catalog
cargo test -p wyvern-cli help_surface
cargo clippy -p wyvern-cli -- -D warnings
bash scripts/check-share-sync.sh
wyvern examples list
wyvern examples list --json | jq 'length >= 4'
```

## Non-goals (this sprint)

- `wyvern examples run <name>` — use `wyvern path/to/wizard.json` or guide bridges
- Listing example files beyond README paths
- User/project example dirs outside `{wyvern_share}/examples/`
- Linking `path-picker` in welcome guide (Phase I / separate sprint)
