---
id: k.4
title: docli-generated CLI HTML documentation
status: planned
branch: feature/phase-K-k4-docli
worktree: ../wyvern-worktrees/feature/phase-K-k4-docli
target: integrate/phase-K
depends_on: k.3
---

# Sprint k.4 — docli CLI HTML docs

## Goal

Generate **`site/cli/`** via installed **`sc-docli`** (**0.2.0** pinned), from committed
CliModel JSON produced by **`gen-cli-model`** binary. **No** `sc-docli` Rust library
dependency in workspace crates (binary-only integration).

## Hard dependencies

- k.3 merged to `integrate/phase-K`

## Deliverables

| Path | Purpose |
|------|---------|
| `crates/wyvern/src/bin/gen-cli-model.rs` | Builds CliModel JSON from `usage_message()` + extensions — **no clap** |
| `fixtures/cli/cli-model.schema.json` | Vendored docli CliModel shape (from docli `fixtures/contract/model.json` @ sc-docli 0.2.0) |
| `fixtures/cli/wyvern-cli-model.json` | Committed CliModel input |
| `scripts/generate-cli-docs.sh` | Install pin + `gen-cli-model` + `docli generate … --html site/cli` |
| `release/docli-pin.toml` | Pinned `sc-docli` version for CI |
| `site/cli/` | Generated HTML (committed; CI drift check) |
| `.github/workflows/ci.yml` | Install sc-docli; regen; `git diff --exit-code` |
| `docs/architecture.md` | **ADR-0027** — docli binary as sole CLI HTML generator |
| `docs/wyvern/requirements.md` | **REQ-0150** — CLI reference via docli |
| `README.md` | Link to CLI HTML index |

## Explicit code samples

Invocation (normative):

```bash
cargo run --bin gen-cli-model -- --output fixtures/cli/wyvern-cli-model.json
docli generate --input fixtures/cli/wyvern-cli-model.json --html site/cli
```

Model builder seam (illustrative — logic lives in `gen-cli-model` binary):

```rust
let usage = wyvern::cli_args::usage_message();
let registry = wyvern::extensions::ExtensionRegistry::from_shipped_json()?;
let model = build_cli_model(&usage, &registry)?; // defined in gen-cli-model.rs
serde_json::to_writer(out, &model)?;
```

CI installs tool:

```bash
cargo install sc-docli --version 0.2.0 --locked
```

## Required work

1. **`gen-cli-model`** emits docli CliModel JSON (**no clap**). Source of truth:
   - `crates/wyvern/src/cli_args.rs` — **`usage_message()`** (no args; embeds catalog)
   - `SHIPPED_EXTENSIONS_JSON` for extension ids/examples
   Serde types live **inside** `gen-cli-model.rs` (or `crates/wyvern/src/cli_model.rs`
   private to the binary). Schema reference: vendored
   `fixtures/cli/cli-model.schema.json` (from
   [docli fixtures/contract/model.json](https://github.com/randlee/docli/blob/develop/fixtures/contract/model.json)).
2. **Do not** add `sc-docli` to `[dependencies]` — only invoke installed `docli` binary.
3. `generate-cli-docs.sh` orchestrates model regen + `docli generate`.
4. CI fails on drift in `fixtures/cli/wyvern-cli-model.json` and `site/cli/`.

## Non-closure

- Markdown manual (`docli --markdown`).
- docli template pack overrides (`templates validate`).
- Adding wyvern to docli `verify-candidate-repos.sh` (optional follow-up in docli repo).

## Acceptance criteria

1. `bash scripts/generate-cli-docs.sh` exits 0; tree clean after regen commit.
2. CI enforces drift on `fixtures/cli/wyvern-cli-model.json` and `site/cli/`.
3. README links to `site/cli/index.html`.
4. **ADR-0027** merged in `docs/architecture.md`.
5. **REQ-0150** in `docs/wyvern/requirements.md` + index in `docs/requirements.md`.
6. `cargo test --workspace` + clippy clean.

## Validation

```bash
bash scripts/generate-cli-docs.sh
git diff --exit-code fixtures/cli/wyvern-cli-model.json site/cli/
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Cursor orchestration

| Field | Value |
|-------|--------|
| `pr_target` | `integrate/phase-K` |
| `sprint_doc` | `docs/plans/phase-k/k4-docli-cli-html-docs.md` |
| QA-1 reviewers | req-qa, arch-qa, rust-qa-agent, rust-best-practices-agent, rust-service-hardening-agent, flaky-test-qa (phase-ending; prefer Terra on `rust-qa-agent` if available) |
| Dev agent | `rust-developer` |
