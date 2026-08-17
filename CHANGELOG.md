# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] — 2026-08-16

Phase G — CLI extension agent usability. A cold agent can discover every Phase F skill, recover from near-misses, and inspect one skill using only `wyvern --help`, `wyvern extensions list`, and stderr JSON.

### Added

- First-class `--help` / `-h` and `wyvern help` listing shipped skills with copy-paste examples
- Extension-prefix `--help` skill cards at match time (`wyvern compose render --help`)
- Skill catalog: `wyvern extensions list` (text + `--json`) and `wyvern extensions show <id>`
- Near-miss diagnostics that name the skill and teach the next command (unknown suffix, incomplete prefix, skipped `requires`)
- Structured preexec failure recovery (child stderr in JSON; spawn vs exit vs missing-file)

### Not in 0.2.0

- `--interactive` argv expansion and MCP tool wrappers — Phase E
- User registry (`~/.config/wyvern/extensions.json`)

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
