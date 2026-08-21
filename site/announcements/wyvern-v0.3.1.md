# wyvern v0.3.0 + v0.3.1 — CLI Extension Runtime, Help System, and Skill Catalog

**Released:** August 16–17, 2026 · **Install:** `cargo install wyvern-cli` (Rust), or download native binaries for macOS, Windows, Linux from [releases](https://github.com/randlee/wyvern/releases)

[Changelog](https://github.com/randlee/wyvern/blob/main/CHANGELOG.md) · [Release notes](https://github.com/randlee/wyvern/releases/tag/v0.3.1)

> **⚠️ v0.3.0 on crates.io is buggy.** The `wyvern-cli` crate published without its bundled extension assets, causing install failures. v0.3.1 fixes this — use `cargo install wyvern-cli` (which picks up v0.3.1) or pin to `v0.3.1`.

---

## Wizard Developer

**As a wizard developer, I want native webview dialogs that return structured JSON, so that I can collect user input in a guided flow without a browser dependency.**

v0.3.0 ships wyvern's CLI extension runtime — the first general-purpose extension mechanism. Phase F delivers four built-in extensions that enhance the wizard development loop:

- **wizard-json-suffix** — appends structured JSON metadata to wizard output, making it machine-consumable for downstream agents
- **html-suffix** — renders wizard results as self-contained HTML for preview and sharing
- **compose-render** — pipes output through `sc-compose` preexec, giving wizards access to Synaptic Canvas composition as a post-processing step
- **CSV table viewer** — renders tabular wizard output as clean CSV for spreadsheet import

Phase G hardens the developer experience with a global `--help` system, per-extension help, and a new `skill catalog` command (`list`, `json`, `show`) that surfaces all available skills with machine-readable metadata. Error-teaches near-miss diagnostics (REQ-0130, REQ-0136) catch common mistakes like missing quotes or wrong flag order and suggest corrections inline — no more opaque CLI failures.

**v0.3.1 fix:** The `wyvern-cli` crate on crates.io shipped v0.3.0 without its bundled extension assets (`extensions/` directory), causing `cargo install` failures. v0.3.1 vendors extension assets into the crate so `cargo install wyvern-cli` works out of the box. If you installed v0.3.0, upgrade immediately.

---

## Agent Orchestrator (reporting)

**As an agent orchestrator, I want orchestration paired with reporting, so that I can see not just that a workflow ran but what it produced.**

The compose-render and CSV table viewer extensions close the reporting loop. Wizard output that previously existed only as a transient dialog can now be composed into structured reports (via `sc-compose` preexec) or exported as CSV for spreadsheet analysis. Error-teaches diagnostics feed back into orchestration: when a wizard step fails, the near-miss diagnostic tells the orchestrator *why* without scraping raw error output.

---

## DAG Designer

**As a DAG designer, I want to visually design agent workflows and export them as JSON, so that I can hand a ready-to-run DAG to an orchestration engine.**

The compose-render extension is a minor quality-of-life improvement for DAG designers: it lets you preview how a DAG's JSON export would render through `sc-compose` before feeding it to an orchestration engine. No structural changes to the DAG editor or export format in this release.

---

## What's Next

v0.3.0 and v0.3.1 together comprise 47+ commits from Phases F and G. The extension runtime is the foundation — subsequent releases will expand the extension catalog, add authoring tooling, and deepen the wizard-DAG-orchestrator integration loop.