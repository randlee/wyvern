# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] — 2026-08-18

Patch release — completes crates.io publish for `wyvern-cli` (extension assets vendored under `crates/wyvern/` in PR #90). Same feature set as 0.3.0.

### Fixed

- `wyvern-cli` crates.io publish verify (embed paths + `check-share-sync` gate)

## [0.3.0] — 2026-08-17

Phase F — declarative CLI extensions. Phase G — agent-facing help, skill catalog, and error-teaches recovery on top of the extension runtime.

### Phase F — CLI extensions

- Extension runtime: bundled registry, argv match (suffix + subcommand), optional Python preexec, template expand → validated `Command` JSON
- Positional suffix defaults — open `.html` wizard pages and `wizard.json` roots without hand-authored JSON
- `compose render` — sc-compose preexec for slide/markdown composition workflows
- CSV — interactive HTML table viewer (`.csv` suffix) and `wyvern md` markdown variant
- `wyvern extensions list` groundwork (expanded in Phase G)

### Phase G — Agent usability

- First-class `--help` / `-h` and `wyvern help` listing shipped skills with copy-paste examples
- Extension-prefix `--help` skill cards at match time (`wyvern compose render --help`)
- Skill catalog: `wyvern extensions list` (text + `--json`) and `wyvern extensions show <id>`
- Near-miss diagnostics that name the skill and teach the next command (unknown suffix, incomplete prefix, skipped `requires`)
- Structured preexec failure recovery (child stderr in JSON; spawn vs exit vs missing-file)

### Distribution

- **crates.io** — publish order unchanged; all published crates bump to 0.3.0
- **GitHub Releases** — tag `v0.3.0` triggers matrix build (macOS/Windows/Linux)

### Not in 0.3.0

- `--interactive` argv expansion and MCP tool wrappers — Phase E
- User registry (`~/.config/wyvern/extensions.json`)

## [0.2.1] — 2026-08-15

Patch release — fixes failed v0.2.0 publish (release workflow only; same Phase D feature set as 0.2.0).

### Fixed

- Release workflow `release-gates` job installs Linux GTK/Wayland deps before `cargo clippy` and `cargo test` (matches main CI)

## [0.2.0] — 2026-08-15

Phase D — multi-page **wizard** flows on the HTTP host stack.

### Wizard runtime

- New `type: "wizard"` command with browser-style stack navigation (ADR-0005/0007)
- `wyvern-wizard` crate: `WizardSession` with `navigate_next`, `navigate_back`, `finish`, `snapshot`
- Host HTTP API: `GET /api/wizard/state`, `POST /api/wizard/navigate`, `POST /api/wizard/finish`
- Dual-mount static assets: `/wizard/**` + `/shared/**`
- Viewer dismiss returns full visited stack JSON on OS window close (ADR-0021)
- Shared wizard chrome (`wizard-nav.js`, `chrome.html`) and viewport sizing helpers

### Examples

- `layout-picker` — DAG branching with back-navigation and data restore
- `turbo-flow` — Svelte Flow workspace graph (dark/light themes)
- `two-page`, `single-page`, `workspace-hint` fixtures

### Quality / CI

- UI sync check (`ui/` ↔ packaged UI) and boundary grep enforcement in CI
- Phase D L2 Playwright specs (layout-picker, viewport-sizing, edge cases, turbo-flow)
- Release workflow quality gates (fmt, clippy, test, audit, deny, boundaries)

### Distribution

- **crates.io** — same publish order as 0.1.0; all published crates bump to 0.2.0
- **GitHub Releases** — tag `v0.2.0` triggers matrix build (macOS/Windows/Linux)

### Not in 0.2.0

- `--interactive` lifecycle / MCP server — Phase E

## [0.1.0] — 2026-07-14

First public release of Wyvern on the **HTTP dialog host** stack (Phase C delivery rewrite).

### Runtime

- Ephemeral local HTTP host (`wyvern-host`) serves packaged UI and awaits JSON results
- Optional embedded viewer (`wyvern-viewer`) — product default `--viewer embedded`
- Headless / CI path: `WYVERN_VIEWER=none` or `--viewer none`
- Named browser registry (`wyvern browsers list|refresh`) for `--viewer chrome|…`

### Dialog types (packaged `share/wyvern/ui/`)

- `message` — blocking modal with title, body, level, and button combos
- `input` — text, multiline, and file/folder chooser modes
- `markdown` — inline content, file path, and `wyvern file.md` shorthand
- `question` — AskUserQuestion-compatible blocking prompt
- `chrome` — foundation chrome frame / platform safe zones

### Distribution

- **crates.io** — publish order: `wyvern-schema` → `wyvern-wizard` → `wyvern-host` → `wyvern-viewer` → `wyvern` (see `release/publish-artifacts.toml`)
- **cargo install** — `cargo install wyvern-cli wyvern-viewer` (installs `wyvern` + `wyvern-viewer` binaries; UI embedded via `rust-embed`)
- **GitHub Releases** — tag-triggered matrix: macOS aarch64/x86_64, Windows x86_64, Linux x86_64; archives include `wyvern`, `wyvern-viewer`, and full `share/wyvern/ui/**`
- **Homebrew** — `brew install randlee/tap/wyvern` (Apple Silicon tarball from GitHub Releases)
- **winget** — `winget install randlee.wyvern` (Windows zip from GitHub Releases)

### Not in 0.1.0

- `wizard` multi-page flows — Phase D
- `--interactive` lifecycle / MCP server — Phase E

### Known issues

- **Linux crates.io consumers:** the workspace `[patch.crates-io]` for `wayland-scanner` does not apply to `cargo install` from crates.io. Linux users installing via crates.io may need to build from source with the vendored patch or use the GitHub Release tarball.
