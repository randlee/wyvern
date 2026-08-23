---
id: h.1
title: Report host + single XHTML panel (basic frame)
status: planning
branch: feature/phase-H-h1-xhtml-single
worktree: ../wyvern-worktrees/feature/phase-H-h1-xhtml-single
target: integrate/phase-H
---

# Sprint h.1 — Single XHTML panel + basic frame

## Goal

Close [#115](https://github.com/randlee/wyvern/issues/115): route `.xhtml` files to a
**report** surface (not wizard) with a minimal document shell around sc-compose
fragments.

## Hard dependencies

- Phase F extension runtime on `develop`
- Contract: [xhtml-reporting-contract.md](xhtml-reporting-contract.md) § Command
  JSON, § Frame `basic-single`, § `xhtml-suffix`

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-H/h1-xhtml-single-panel.md` | This sprint doc |
| `crates/wyvern-schema/src/report.rs` | `ReportCommand` + validate |
| `crates/wyvern-schema/src/command.rs` | `Command::Report { … }` variant |
| `crates/wyvern-host/src/routes/report.rs` | Static `/report/*` + view dismiss |
| `crates/wyvern-host/src/server.rs` | Report router branch |
| `crates/wyvern/src/pipeline.rs` | Run report commands |
| `scripts/ext/xhtml_report.py` | `--mode single` fragment wrapper |
| `ui/shared/report-base.css` | Minimal pane typography |
| `share/wyvern/extensions.json` | `xhtml-suffix` entry |
| `crates/wyvern/embedded/…` | Parity for extensions + ui |
| `crates/wyvern/tests/extensions_xhtml_single.rs` | Expand + frame smoke |
| `crates/wyvern-host/tests/report_view.rs` | Headless URL resolves |

### `xhtml-suffix` registry (normative)

```json
{
  "id": "xhtml-suffix",
  "description": "Open an XHTML panel as a report view (wrapped document frame).",
  "examples": ["wyvern panel.xhtml"],
  "match": { "positional_suffix": ".xhtml" },
  "preexec": {
    "cmd": "python3",
    "args": ["{wyvern_share}/scripts/ext/xhtml_report.py", "--mode", "single", "--input", "{path}", "--title", "{basename}"],
    "requires": ["python3"]
  },
  "expand": {
    "command": {
      "type": "report",
      "title": "{basename}",
      "page": "pages/view.xhtml",
      "mode": "view"
    },
    "host": { "ui_root": "{tmpdir}" }
  }
}
```

## Acceptance criteria

1. `wyvern path/to/panel.xhtml` expands to `type: "report"`, `mode: "view"`, exit 0
   with `--viewer none` when panel file exists.
2. sc-compose-style fragment (atm-core `<section xmlns=…>`) renders in viewer after
   preexec wrap — not raw XML text dump.
3. `html-suffix` behavior unchanged; `.html` still expands to wizard per Phase F.
4. `wyvern extensions list` documents `xhtml-suffix` with example.
5. Invalid/missing path → existing extension/host error paths (no panic).

## Required validation

```bash
cargo test -p wyvern-schema report_
cargo test -p wyvern-cli extensions_xhtml_single
cargo test -p wyvern-host report_view
scripts/check-share-sync.sh
python3 scripts/ext/xhtml_report.py --mode single --input /path/to/fixture.xhtml --title t --out /tmp/xhtml-test
```

## Non-closure

- Multi-panel manifests (h.2)
- `--review` finish (h.3)
- `wyvern-reporting` skill (h.4)

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md)
- [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md)
