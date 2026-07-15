# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
