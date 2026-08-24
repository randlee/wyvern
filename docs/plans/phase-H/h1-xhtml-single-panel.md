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
| `docs/architecture.md` | **ADR-0025** (report command) + **ADR-0022 amendment** (Phase H exception) |
| `docs/plans/phase-F/cli-extensions-contract.md` | Phase H amendment: `command_from_file` report pattern, `report-xhtml` ids |
| `docs/wyvern/requirements.md` | **REQ-0140–REQ-0143** (report command, xhtml suffix, host options) |
| `docs/wyvern-host/requirements.md` | **REQ-HOST-0140–0141** (`/report/*`, `/shared/*` mount) |
| `docs/requirements.md` | Command-surface index: `.xhtml`, `report-xhtml` |
| `crates/wyvern-schema/src/report.rs` | `ReportMode`, `ReportPagePath`, `ReportTitle`, `ReportCommand` |
| `crates/wyvern-schema/src/validate/report.rs` | Report validator module (wired via `validate/mod.rs`) |
| `crates/wyvern-schema/src/command.rs` | `Command::Report(ReportCommand)` variant + exhaustive match sites |
| `crates/wyvern-schema/src/result.rs` | `ReportResult` / `CommandResult::Report` (view dismiss + review finish) |
| `crates/wyvern-host/src/routes/report.rs` | Static `/report/*` routes |
| `crates/wyvern-host/src/server.rs` | `Command::Report` bind arm: `/report/{page}` URL + router nest |
| `crates/wyvern-host/src/static_files.rs` | `require_report_page(ui_root, page)` — **not** `require_type_dir` |
| `crates/wyvern-host/src/handle.rs` | `DialogTypeName::Report` exhaustive arm |
| `crates/wyvern-host/src/options.rs` | Report title from command |
| `crates/wyvern-host/src/routes/result.rs` | View-mode dismiss via shared `/api/result` |
| `crates/wyvern/src/cli_args.rs` | `usage_message()` — `.xhtml` suffix on Extensions list (REQ-0137) |
| `boundaries/wyvern-host/host.toml` | `report_routes` / `report_session` in `io_owns` |
| `docs/wyvern-schema/architecture.md` | Command / CommandResult enum samples include `report` |
| `docs/wyvern-host/architecture.md` | Module shape includes `routes/report.rs` |
| `crates/wyvern/src/pipeline.rs` | Run report commands |
| `scripts/ext/xhtml_report.py` | `--mode single`; writes wrapped HTML to `{tmpdir}/pages/view.xhtml` (creates `pages/` if absent); `--mode array` stub non-zero until h.2 |
| `ui/shared/report-base.css` | Minimal pane typography |
| `share/wyvern/extensions.json` | `xhtml-suffix` entry |
| `crates/wyvern/embedded/…` | Parity for extensions + ui |
| `crates/wyvern/tests/extensions_xhtml_single.rs` | Expand + frame smoke |
| `crates/wyvern/tests/extensions_catalog.rs` | Extend `req_0137_registry_help_parity` for `xhtml-suffix` |
| `crates/wyvern-host/tests/report_view.rs` | Headless URL + bind URL `/report/{page}` resolves |
| `crates/wyvern-host/tests/report_bind.rs` | `require_report_page` rejects packaged `ui/report/index.html` pattern |

### REQ traceability (h.1 lands)

| REQ | Summary |
|-----|---------|
| REQ-0140 | `type: "report"` command JSON validated in `wyvern-schema` |
| REQ-0141 | `.xhtml` suffix expands via `xhtml-suffix` to report view (not wizard) |
| REQ-0142 | Report host serves static page under `/report/*` + shared CSS at `/shared/*` |
| REQ-0143 | View mode dismiss → `{"button":"dismissed"}` |

Host REQ text: REQ-HOST-0140 (`/report/*`), REQ-HOST-0141 (`/shared/*` during report sessions).

### Host bind (normative — ADR-0025)

Report uses a **third bind arm** analogous to wizard (not packaged dialog dirs):

1. `require_report_page(ui_root, page)` validates `{ui_root}/{page}` exists — **forbidden:**
   `require_type_dir` / `{ui_root}/report/index.html` packaged layout.
2. Dialog URL: `/report/{page}` (page path relative to `ui_root`, e.g. `/report/pages/view.xhtml`).
3. `ServeDir` nest at `/report` from session `ui_root` override.
4. `GET /api/dialog` returns wizard-class rejection for report sessions (static page only).

### Rust types (normative samples — h.1 lands)

```rust
pub enum ReportMode { View, Review }

pub struct ReportPagePath(String); // try_new + Deref/AsRef (wizard_page_newtype pattern)
pub struct ReportTitle(String);
pub enum PanelRole { Failure, Proposal, Info }
pub struct ManifestPanelPath(String); // .xhtml relative path, validated

pub struct ReportPanelEntry {
    pub path: ManifestPanelPath,
    pub label: Option<String>,
    pub role: Option<PanelRole>,
}

pub struct ReportCommand {
    pub title: ReportTitle,
    pub page: ReportPagePath,
    pub mode: ReportMode,
    pub panels: Option<Vec<ReportPanelEntry>>, // required when mode == Review
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub enum ReportTerminalButton { Dismissed, Finish }

pub struct ReportFinishData {
    pub approved: bool,
    pub comments: String,
    pub panels: Vec<ReportPanelEntry>,
}

pub struct ReportResult {
    pub button: ReportTerminalButton,
    pub data: Option<ReportFinishData>, // None in view dismiss
}
```

Exhaustive match sites that gain a `Report` arm in h.1: `command.rs` parse/validate,
`pipeline.rs`, `wyvern-host` `handle.rs`, `options.rs`, `server.rs` three-way bind
(`dialog` \| `wizard` \| `report` — report **must not** set `is_wizard`), `result.rs`
emit path (view dismiss only until h.3 finish).

**Session guards (normative):** review-only routes and finish validation use
`SessionState::complete` + mode-gated registration (runtime guards); typestate deferral
is acceptable at plan level.

### `xhtml-suffix` registry (normative)

```json
{
  "id": "xhtml-suffix",
  "description": "Open an XHTML panel as a report view (wrapped document frame).",
  "examples": ["wyvern panel.xhtml"],
  "match": { "positional_suffix": ".xhtml" },
  "preexec": {
    "cmd": "python3",
    "args": ["{wyvern_share}/scripts/ext/xhtml_report.py", "--mode", "single", "--input", "{path}", "--title", "{basename}", "--out", "{tmpdir}/pages/view.xhtml"],
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
cargo test -p wyvern-cli --test extensions_xhtml_single
cargo test -p wyvern-cli --test extensions_catalog req_0137
cargo test -p wyvern-host report_view
cargo test -p wyvern-host report_bind
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
