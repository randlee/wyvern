# wyvern v0.6.0 — Kit-Managed Distribution (Phase J)

**Released:** August 28, 2026 · **Install:** `brew install randlee/homebrew-tap/wyvern` (macOS/Linux), `scoop bucket add randlee https://github.com/randlee/scoop-bucket && scoop install wyvern` (Windows), `winget install randlee.wyvern` (Windows), `cargo install wyvern-cli` (Rust), or download native binaries for macOS, Windows, Linux from [releases](https://github.com/randlee/wyvern/releases)

[Changelog](https://github.com/randlee/wyvern/blob/main/CHANGELOG.md) · [Release notes](https://github.com/randlee/wyvern/releases/tag/v0.6.0)

---

## Wizard Developer

**As a wizard developer, I want native webview dialogs that return structured JSON, so that I can collect user input in a guided flow without a browser dependency.**

v0.6.0 is the first kit-managed release — and for the wizard developer, that means one-command install on every platform. wyvern now ships through **Homebrew** (macOS/Linux), **Scoop** (Windows), and **winget** (Windows), all driven by the `sc-publish` kit pipeline. No more downloading tarballs from GitHub Releases manually: `brew install randlee/homebrew-tap/wyvern` pulls a prebuilt native binary with all bundled UI assets (`share/wyvern/ui/`) — the same ~5 MB footprint, now installable in a single shell command.

Under the hood, the kit enforces a strict publish plan: five crates (`wyvern-schema`, `wyvern-wizard`, `wyvern-host`, `wyvern-viewer`, `wyvern-cli`) publish to crates.io in dependency order with timed waits between them, so `cargo install wyvern-cli` works on the first try. The kit also regenerates Homebrew formulas, Scoop manifests, and winget manifests from Jinja2 templates on every release, keeping them in lockstep with the tag.

The wizard API surface is unchanged: all dialog types (`message`, `input`, `markdown`, `question`, `chrome`, `wizard`, `report`), the HTTP dialog host, wizard session navigation, and CLI extensions (`wyvern extensions list`, `.html`/`.csv`/`compose render` suffixes) carry forward from v0.5.0 with no breaking changes.

---

## Agent Orchestrator (reporting)

**As an agent orchestrator, I want orchestration paired with reporting, so that I can see not just that a workflow ran but what it produced.**

The kit pipeline is built for CI. A new **release-candidate workflow** runs on the default branch so that RC dispatches don't depend on a feature-branch checkout — agents triggering `workflow_dispatch` against `origin/develop` get consistent RC artifacts regardless of which branch is checked out. Preflight checks are hardened: the publish plan now validates only the first crate in the dependency chain (later crates need earlier ones live on crates.io), sc-lint boundary smoke tests use `forbidden_edges` tables compatible with sc-lint 0.4.0, and the report-finish transient retry is extended so CI preflight doesn't flake on a timing-sensitive dialog close.

For orchestrators that shell out to wyvern in headless CI, `brew install` or `scoop install` in a GitHub Actions step means reproducible, versioned installs with no curl-the-tarball boilerplate. The matrix build (macOS ARM/Intel, Windows x64, Linux x64) continues to produce release artifacts, now with kit-managed checksums and archive naming.

---

## DAG Designer

**As a DAG designer, I want to visually design agent workflows and export them as JSON, so that I can hand a ready-to-run DAG to an orchestration engine.**

Homebrew puts wyvern on the same footing as every other design-time tool in a macOS workflow. `brew install randlee/homebrew-tap/wyvern` drops `wyvern` and `wyvern-viewer` into `/opt/homebrew/bin`, so DAG designers working alongside `graphviz`, `d2`, or `mermaid-cli` get wyvern without leaving their package manager. The wizard JSON schema, DAG branching model (`wyvernWizardNext` / `wyvernWizardFinish`), and workspace layout mode are unchanged — this release is purely about how you get the binary, not what the binary does.

---

## What's Next

v0.6.0 locks in the distribution surface. With Homebrew, Scoop, and winget live, every future release inherits one-command install for free — the kit regenerates formulas and manifests automatically. Follow-ups remain Phase E `--interactive` argv expansion and the MCP server binary, and the user extension registry (`~/.config/wyvern/extensions.json`). The winget bootstrap (PR #425477 against `microsoft/winget-pkgs`) is in flight and will be superseded by the kit-managed `winget-publish.yml` on the next release.