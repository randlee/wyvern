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

### Release bundle

- Tag-triggered GitHub Actions matrix: macOS aarch64/x86_64, Windows x86_64, Linux x86_64
- Artifacts include `wyvern`, sibling `wyvern-viewer`, and full `share/wyvern/ui/**`
- README quickstart for install from GitHub Releases without cloning

### Not in 0.1.0

- `wizard` multi-page flows — Phase D
- `--interactive` lifecycle / MCP server — Phase E
