# wyvern v0.5.0 — Headless CI and Agent Hardening

**Released:** August 26, 2026 · **Install:** `cargo install wyvern-cli` (Rust), or download native binaries for macOS, Windows, Linux from [releases](https://github.com/randlee/wyvern/releases)

[Changelog](https://github.com/randlee/wyvern/blob/main/CHANGELOG.md) · [Release notes](https://github.com/randlee/wyvern/releases/tag/v0.5.0)

---

## Agent Orchestrator (reporting)

**As an agent orchestrator, I want orchestration paired with reporting, so that I can see not just that a workflow ran but what it produced.**

v0.5.0 is a focused hardening release on top of v0.4.0 (Phase H XHTML reporting, Phase I wizard pickers, and the g.15 examples catalog), aimed squarely at the headless agent/CI path. Headless mode (`WYVERN_VIEWER=none` / `--viewer none`) now runs under a **30-second idle session budget**. An undriven blocking dialog — one where the harness never drives `WYVERN_DIALOG_URL` — now exits with **`SESSION_TIMEOUT_ERROR` (exit code 6)** instead of silently emitting dismissed JSON.

For an orchestrator that shells out to wyvern in CI, this is the difference between a misconfigured test passing silently and a hard, machine-readable failure. A headless hang that ends in exit 6 means the harness did not drive the dialog host — fix the test, don't raise the timeout. The embedded viewer (the default product path) is unchanged at **600s**, so desktop-driven flows see no behavior change.

Every exit is now unambiguous on stdout: a real result carries its structured JSON, while an undriven dialog fails fast with exit 6 and a `SESSION_TIMEOUT_ERROR` payload rather than a misleading `{ "button": "dismissed" }`.

---

## Wizard Developer

**As a wizard developer, I want native webview dialogs that return structured JSON, so that I can collect user input in a guided flow without a browser dependency.**

Wizard flows run headless get the same fail-fast semantics plus hardened tests. Playwright input picker specs now wait for mock picker field population before pressing OK, and the wizard-timeout L1 test avoids racing setup, so a wizard that times out in CI surfaces as exit 6 rather than a flaky green. `WYVERN_VIEWER=none wyvern examples list` is an instant headless smoke that runs without a dialog host. Nothing in the wizard API or DAG branching model changes.

---

## DAG Designer

*No impact this release — skipped.* The visual DAG editor is a webview-based, design-time surface; the headless idle-timeout and e2e test hardening don't touch its export format or branching model.

---

## What's Next

v0.5.0 is a small, deliberate release: it locks in fail-fast behavior for undriven headless dialogs so agents and CI pipelines can trust a hang to mean "not driven" rather than "user dismissed". Follow-ups remain Phase E `--interactive` argv expansion and the MCP server binary, the user extension registry (`~/.config/wyvern/extensions.json`), and the winget bootstrap submission.
