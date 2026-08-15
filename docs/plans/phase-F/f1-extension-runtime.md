---
id: f.1
title: Extension runtime — registry, match, preexec, expand
status: planning
branch: feature/phase-F-f1-extension-runtime
worktree: ../wyvern-worktrees/feature/phase-F-f1-extension-runtime
target: integrate/phase-F
---

# Sprint f.1 — Extension runtime

## Goal

Ship the CLI extension engine: load merged registry, match argv, optional `preexec`, template expand to `Command` JSON + `HostOptions`, validate, dispatch existing pipeline. Migrate hardcoded `.md` shorthand into shipped registry.

## Hard dependencies

- Phase D complete (wizard + `--ui-root` on host)
- v0.2.x on `develop`

## Deliverables

### Registry + contract

| Path | Change |
|------|--------|
| `docs/plans/phase-F/cli-extensions-contract.md` | Normative schema (already in plan branch) |
| `share/wyvern/extensions.json` | Shipped defaults (`.md` only in f.1) |
| `crates/wyvern/src/extensions/mod.rs` | Loader, merge, `extends` resolution, match precedence |
| `crates/wyvern/src/extensions/expand.rs` | Template vars + JSON expand |
| `crates/wyvern/src/extensions/preexec.rs` | Spawn subprocess; `{tmpdir}` lifecycle |

### CLI integration

| Path | Change |
|------|--------|
| `crates/wyvern/src/input.rs` | Delegate positional loading to extension matcher before inline JSON |
| `crates/wyvern/src/cli_args.rs` | Apply expanded `host.ui_root` when present; extend `usage_message` one line |
| `crates/wyvern/src/extensions/list.rs` | `wyvern extensions list` subcommand |

### Rust API (signatures)

```rust
pub struct ExtensionRegistry { /* merged extensions */ }

pub enum ExtensionMatch<'a> {
    Suffix { ext: &'a ExtensionDef, path: &'a str },
    Prefix { ext: &'a ExtensionDef, args: &'a [String] },
}

impl ExtensionRegistry {
    pub fn load(defaults: &Path, project: Option<&Path>, user: Option<&Path>) -> Result<Self, ExtensionError>;
    pub fn match_argv(&self, argv: &[String]) -> Option<ExtensionMatch<'_>>;
}

pub struct ExpandedInvocation {
    pub command: serde_json::Value,
    pub host_overrides: HostOverrides, // ui_root, etc.
}

pub fn expand_and_validate(
    ext: &ExtensionDef,
    ctx: &MatchContext,
) -> Result<ExpandedInvocation, ExtensionError>;

/// Resolve `extends` chain; child match overrides parent.
pub fn resolve_extension<'a>(registry: &'a ExtensionRegistry, id: &str) -> Result<&'a ExtensionDef, ExtensionError>;
```

### Shipped extension (f.1 only)

```json
{
  "id": "markdown-suffix",
  "match": { "positional_suffix": ".md" },
  "expand": {
    "command": { "type": "markdown", "file": "{path}" }
  }
}
```

Removes duplicate logic from `load_positional` `.md` branch.

## Acceptance criteria

1. `wyvern doc.md` behaves identically to pre-f.1 (markdown file shorthand via registry)
2. Unknown suffix falls through to inline JSON / usage error unchanged
3. `wyvern extensions list` prints shipped ids + match kind + description
4. Expanded command always passes `wyvern_schema::validate` before host run
5. Invalid registry JSON → structured CLI error (load time), not panic
6. `preexec` temp dirs removed on success and failure
7. Workspace builds; clippy clean; no new host routes

## Required validation

```bash
cargo test -p wyvern extensions
cargo test -p wyvern input_md_path_loads_markdown_value  # still passes via registry
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

## Non-closure

- `.html`, `.csv`, `compose` extensions (f.2–f.4)
- User registry override merge beyond shipped + `.wyvern/extensions.json` path stub (may land if trivial)

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- [f2-positional-extensions.md](f2-positional-extensions.md) (downstream consumer)
