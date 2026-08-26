# wyvern v0.4.0 — XHTML Reporting, Native Path Pickers, and Examples Catalog

**Released:** August 24, 2026 · **Install:** `cargo install wyvern-cli` (Rust), or download native binaries for macOS, Windows, Linux from [releases](https://github.com/randlee/wyvern/releases)

[Changelog](https://github.com/randlee/wyvern/blob/main/CHANGELOG.md) · [Release notes](https://github.com/randlee/wyvern/releases/tag/v0.4.0)

---

## Wizard Developer

**As a wizard developer, I want native webview dialogs that return structured JSON, so that I can collect user input in a guided flow without a browser dependency.**

v0.4.0 expands wyvern's surface beyond the wizard stack with Phase H XHTML reporting. A new **Report command** binds static XHTML pages at `/report/{page}` — document surfaces that render without any wizard state machine, opened by suffix (`wyvern panel.xhtml`) or by manifest (`wyvern report-xhtml manifest.json`). Multi-panel reports stitch an ordered array of XHTML panels into a single view, with per-panel `path`, `label`, and `role` fields. For interactive review, `wyvern report-xhtml --review` adds a review shell with per-panel comments and **Cancel** / **Approve** actions.

Phase I brings native path pickers into wizard sessions. New `WyvernApi.postPickerFile()` and `postPickerFolder()` calls open in-page OS file/folder choosers while a wizard is live, backed by `POST /api/picker/file` and `POST /api/picker/folder` routes (ADR-0026). The bundled `path-picker` example demonstrates a two-page wizard that collects seed paths through native dialogs and finishes with JSON path strings — closing [#99](https://github.com/randlee/wyvern/issues/99).

Finally, the g.15 examples catalog (`wyvern examples list`, with `--json`) makes bundled examples discoverable from their README frontmatter, so you can find a starting point without reading the source tree.

---

## Agent Orchestrator (reporting)

**As an agent orchestrator, I want orchestration paired with reporting, so that I can see not just that a workflow ran but what it produced.**

XHTML reporting closes the reporting loop on the orchestration side. Review mode completes through `POST /api/report/finish`, returning structured JSON on stdout — `{ "button": "finish", "data": { "approved", "comments", "panels" } }` — so an orchestrator gets a machine-readable approval verdict with per-panel comments rather than scraping a dialog. View-mode dismissal returns `{ "button": "dismissed" }` through the same shared result semantics, keeping every report surface programmatically consumable.

The bundled `wyvern-reporting` skill (`.claude/skills/wyvern-reporting/`) gives agents panel-authoring guidance, the manifest schema, and finish-parsing instructions, so report generation is a first-class agent capability rather than a hand-rolled script.

---

## DAG Designer

**As a DAG designer, I want to visually design agent workflows and export them as JSON, so that I can hand a ready-to-run DAG to an orchestration engine.**

Native path pickers are a focused quality-of-life win for DAG designers: file and folder selection that previously required stringing paths together by hand now happens through OS-native choosers inside the wizard session. The `path-picker` example is the canonical reference, but the picker API applies to any wizard that needs validated filesystem input. No changes to the DAG export format or branching model in this release.

---

## What's Next

v0.4.0 spans Phases H and I plus the g.15 catalog work — 105 commits since v0.3.1. XHTML reporting and native pickers extend the same JSON-in/JSON-out contract, and the examples catalog makes the growing extension surface discoverable. Later releases will deepen the reporting pipeline (MCP server, `--interactive` expansion) and the authoring toolchain.
