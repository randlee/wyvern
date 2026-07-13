# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-07-13

First public release of Wyvern: native webview dialogs over JSON I/O.

### Phase A — Core dialog types

- `message` — blocking modal with title, body, level icons, and standard button combos
- `input` — text, multiline, and file/folder chooser modes
- `markdown` — inline content, file path, and `wyvern file.md` shorthand
- `question` — AskUserQuestion-compatible blocking prompt

### Phase B — Wizard

- `wizard` — multi-page wizard with browser-history navigation
- Page descriptors plus HTML + JSON config driving the flow

### Phase C — Icons, chrome, and NFR pass

- Production icon asset bundle and full icon field resolution
- Windows and Linux platform chrome
- Cross-platform NFR validation and auto-size bounds

### Release tooling

- Tag-triggered GitHub Actions release matrix (macOS aarch64/x86_64, Windows x86_64, Linux x86_64)
- README quickstart for install from GitHub Releases without cloning the repo
