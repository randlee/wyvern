---
id: f.2
title: Positional extensions — HTML and wizard.json
status: planning
branch: feature/phase-F-f2-positional-extensions
worktree: ../wyvern-worktrees/feature/phase-F-f2-positional-extensions
target: integrate/phase-F
---

# Sprint f.2 — Positional extensions (HTML + wizard.json)

## Goal

Add suffix handlers for custom HTML and wizard fixture JSON. Infer `--ui-root` from file path so users need not hand-build wizard commands.

## Hard dependencies

- f.1 merged to `integrate/phase-F`

## Deliverables

### Shipped registry entries (`share/wyvern/extensions.json`)

**`.html` suffix** → single-page wizard:

```json
{
  "id": "html-suffix",
  "match": { "positional_suffix": ".html" },
  "expand": {
    "command": {
      "type": "wizard",
      "page": {
        "id": "{stem}",
        "title": "{basename}",
        "html": "{relpath_from_ui_root}"
      }
    },
    "host": { "ui_root": "{parent_dir}" }
  }
}
```

`{relpath_from_ui_root}`: path relative to parent of HTML file when HTML lives under `pages/` or at ui root (document rule in contract).

**`wizard.json` suffix** → load JSON command file + ui root inference:

```json
{
  "id": "wizard-json-suffix",
  "match": { "positional_suffix": "wizard.json" },
  "expand": {
    "command_from_file": "{path}",
    "host": { "ui_root": "{parent_dir}" }
  }
}
```

### CLI / docs

| Path | Change |
|------|--------|
| `crates/wyvern/src/cli_args.rs` | `--help` + usage: document `.html`, `wizard.json`, link to Phase F README |
| `README.md` | Quickstart: open custom HTML page |
| `examples/wizards/single-page/` | Note in comment or README for `wyvern pages/only.html` one-liner |

### Tests

| Path | Change |
|------|--------|
| `crates/wyvern/tests/extensions_html.rs` | Expand `.html` → wizard JSON + ui_root |
| `crates/wyvern/tests/extensions_wizard_json.rs` | Expand fixture wizard.json |

Host L1 optional: headless `--viewer none` smoke that expanded wizard URL resolves (reuse wizard test helpers).

## Acceptance criteria

1. `wyvern examples/wizards/single-page/pages/only.html` opens single-page wizard without manual `--ui-root` or inline JSON
2. `wyvern examples/wizards/turbo-flow/wizard.json` loads turbo-flow with `--ui-root` = turbo-flow dir
3. Expanded commands validate; missing page file → existing host `UiNotFound` error path
4. `.md` shorthand still works via f.1 registry
5. `wyvern extensions list` shows `html-suffix`, `wizard-json-suffix`

## Required validation

```bash
cargo test -p wyvern extensions_html extensions_wizard_json
cargo test -p wyvern-host wizard_state  # regression
scripts/demo-wizard.sh single-page  # manual smoke note in PR
```

## Non-closure

- CSV, compose, `md` subcommand (f.3–f.4)
- `type: "html"` new schema variant (extensions only)

## Authority

- [cli-extensions-contract.md](cli-extensions-contract.md)
- Phase D wizard + `--ui-root` routing ([d1-wizard-host.md](../phase-D/d1-wizard-host.md))
