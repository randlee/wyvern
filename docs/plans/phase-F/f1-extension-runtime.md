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
| `crates/wyvern/src/extensions/expand.rs` | Template vars + JSON expand (all contract vars — see below) |
| `crates/wyvern/src/extensions/preexec.rs` | Spawn subprocess; `{tmpdir}` lifecycle; requires-check |

### Template variables (`expand.rs` — f.1 ships all)

f.1 must implement every contract template variable so f.2–f.4 do not discover missing vars at integration time:

| Variable | f.1 test |
|----------|----------|
| `{path}`, `{basename}`, `{stem}`, `{parent_dir}` | `.md` expand unit test |
| `{relpath_from_ui_root}` | expand unit test with nested path |
| `{tmpdir}`, `{wyvern_share}` | preexec + expand unit test |
| `{preexec.stdout}` | test-only extension with `stdout: "markdown"` preexec |
| `{arg:name}` | test-only prefix extension with `--root` capture |
| `{rendered_basename}` | test-only preexec that writes `{tmpdir}/pages/preview.html` |

`MatchContext` carries preexec results (stdout, rendered basename) for expand substitution.

### Preexec cleanup test

| Path | Change |
|------|--------|
| `crates/wyvern/tests/extensions_preexec_cleanup.rs` | Test-only registry entry with preexec; assert tmpdir removed on success **and** on non-zero preexec exit |

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
    /// f.1: `defaults` + optional project `.wyvern/extensions.json` only.
    /// User config path (`~/.config/wyvern/extensions.json`) deferred post-F.
    pub fn load(defaults: &Path, project: Option<&Path>) -> Result<Self, ExtensionError>;
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
cargo test -p wyvern extensions_preexec_cleanup
cargo test -p wyvern input_md_path_loads_markdown_value  # still passes via registry
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

## Non-closure

- `.html`, `.csv`, `compose` extensions (f.2–f.4)
- User registry (`~/.config/wyvern/extensions.json`) merge — deferred post-F; f.1 `load()` accepts project path only

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- [f2-positional-extensions.md](f2-positional-extensions.md) (downstream consumer)
