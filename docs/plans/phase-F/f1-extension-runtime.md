---
id: f.1
title: Extension runtime — registry, match, preexec, expand
status: qa_pending
branch: feature/phase-F-f1-extension-runtime
worktree: ../wyvern-worktrees/feature/phase-F-f1-extension-runtime
target: integrate/phase-F
---

# Sprint f.1 — Extension runtime

## Goal

Ship the CLI extension engine: load merged registry, match argv remainder, optional two-phase preexec+expand, validate, dispatch existing pipeline. Migrate hardcoded `.md` shorthand into shipped registry. Expose `wyvern::extensions` library API for Phase E.

## Hard dependencies

- Phase D complete (wizard + `--ui-root` on host)
- v0.2.x on `develop`

## Deliverables

### Registry + contract

| Path | Change |
|------|--------|
| `docs/plans/phase-F/cli-extensions-contract.md` | Normative schema (already in plan branch) |
| `docs/architecture.md` | ADR-0022 entry (extensions preprocessor; MCP Path A) |
| `share/wyvern/extensions.json` | Shipped defaults (`.md` only in f.1) |
| `crates/wyvern/Cargo.toml` | rust-embed maps workspace `share/wyvern/**` and `scripts/ext/**` from crate-relative `../../` paths. Cargo `include` cannot package files outside the crate directory; embed compiles them into the binary. |
| `crates/wyvern/src/extensions/mod.rs` | Loader, merge, `extends`, match precedence, `{wyvern_share}` resolve |
| `crates/wyvern/src/extensions/expand.rs` | Phase-1 + phase-2 template substitution (all contract vars) |
| `crates/wyvern/src/extensions/preexec.rs` | Subprocess spawn, requires-check, `{arg:*}` capture, stdout capture |

### CLI integration

| Path | Change |
|------|--------|
| `crates/wyvern/src/main.rs` | Extension dispatch: strip host flags → `match_argv(remainder)` before input fallback |
| `crates/wyvern/src/cli_args.rs` | Host-only flag split; pass extension remainder; apply expanded `host.ui_root` |
| `crates/wyvern/src/input.rs` | Multi-token remainder when extension matched; else positional JSON path |
| `crates/wyvern/src/extensions/list.rs` | `wyvern extensions list` subcommand |

### Template variables (`expand.rs` — f.1 ships all)

| Variable | f.1 test |
|----------|----------|
| `{path}`, `{basename}`, `{stem}`, `{parent_dir}` | `.md` suffix expand |
| `{wizard_root}`, `{relpath_from_ui_root}` | wizard-root walk unit test |
| `{tmpdir}`, `{wyvern_share}` | preexec + embed path resolve test |
| `{preexec.stdout}` | test-only markdown stdout capture |
| `{arg:name}`, `{arg:name:repeat}` | prefix extension with `--root`, two `--var-file` |
| `{rendered_basename}` | preexec writes one html under `{tmpdir}/pages/` |

### Rust API (signatures)

```rust
pub struct ExtensionRegistry { /* merged extensions */ }

pub enum ExtensionMatch<'a> {
    Suffix { ext: &'a ExtensionDef, path: &'a str },
    Prefix { ext: &'a ExtensionDef, args_after_prefix: &'a [String] },
    PrefixSuffix { ext: &'a ExtensionDef, path: &'a str, args_after_prefix: &'a [String] },
}

impl ExtensionRegistry {
    pub fn load(defaults: &Path, project: Option<&Path>) -> Result<Self, ExtensionError>;
    pub fn match_argv(&self, argv: &[String]) -> Option<ExtensionMatch<'_>>;
}

pub struct MatchContext<'a> {
    pub path: Option<&'a str>,           // None for Prefix-only (compose)
    pub args_after_prefix: &'a [String],
    pub preexec_stdout: Option<String>,
    pub rendered_basename: Option<String>,
    pub tmpdir: Option<PathBuf>,         // path to the temporary directory if one was created
    pub wyvern_share: PathBuf,           // resolved path to the wyvern share directory
}

pub struct HostOverrides {
    pub ui_root: Option<PathBuf>,
}

pub struct ExpandedInvocation {
    pub command: serde_json::Value,
    pub host_overrides: HostOverrides,
    pub temp_guard: Option<TempDir>,      // kept until host exit when ui_root = tmpdir
}

pub fn build_match_context<'a>(m: &'a ExtensionMatch<'a>, ext: &ExtensionDef) -> MatchContext<'a>;

/// Phase 1: expand preexec.cmd/args only.
pub fn expand_preexec_args(ext: &ExtensionDef, ctx: &MatchContext) -> Result<(String, Vec<String>), ExtensionError>;

/// Phase 2: expand command + host after preexec.
pub fn expand_command_host(ext: &ExtensionDef, ctx: &MatchContext) -> Result<(serde_json::Value, HostOverrides), ExtensionError>;

pub fn expand_and_validate(ext: &ExtensionDef, ctx: &MatchContext) -> Result<ExpandedInvocation, ExtensionError>;
```

### Tests

| Path | Change |
|------|--------|
| `crates/wyvern/tests/extensions_preexec_cleanup.rs` | Tmpdir deleted after mocked host exit; deleted on preexec failure; present during host |
| `crates/wyvern/tests/extensions_argv_pipeline.rs` | `compose render --root` survives parse; prefix+suffix reach matcher |
| `crates/wyvern/tests/extensions_embed_paths.rs` | `{wyvern_share}` resolves extensions.json + scripts in dev + embedded layouts |

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

## Acceptance criteria

1. `wyvern doc.md` behaves identically to pre-f.1 (markdown file shorthand via registry)
2. Unknown suffix falls through to inline JSON / usage error unchanged
3. `wyvern extensions list` prints each extension id, match-kind summary (e.g. `suffix: .md`), and `(requires: …)` when applicable
4. Expanded command always passes `wyvern_schema::validate` before host run
5. Invalid registry JSON → structured CLI error (load time), not panic
6. `{tmpdir}` temp guard: deleted **after host exit** on success; deleted immediately on preexec failure; present during host run
7. Prefix extensions (`table`, `md`, `compose render`) reachable via argv remainder pipeline
8. `--version` / `-V` built-in unchanged; extension `host.ui_root` overrides CLI `--ui-root` when set
9. Workspace builds; clippy clean; no new host routes; `wyvern::extensions` pub mod for Phase E `--interactive`

## Required validation

```bash
cargo test -p wyvern extensions extensions_preexec_cleanup extensions_argv_pipeline extensions_embed_paths
cargo test -p wyvern input_md_path_loads_markdown_value
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

## Non-closure

- `.html`, `.csv`, `compose` registry entries (f.2–f.4)
- User registry (`~/.config/wyvern/extensions.json`) — post-F

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- ADR-0022 (extensions preprocessor — see contract Phase E section)
