# wyvern-cli

**What You View, Engine Renders Natively** — a CLI that opens an OS-native webview for a JSON dialog command and writes a JSON result to stdout.

[![Documentation](https://docs.rs/wyvern-cli/badge.svg)](https://docs.rs/wyvern-cli)
[![crates.io](https://img.shields.io/crates/v/wyvern-cli.svg)](https://crates.io/crates/wyvern-cli)

## Install

```bash
cargo install wyvern-cli --locked
```

Platform archives, Homebrew, and winget are published from [GitHub Releases](https://github.com/randlee/wyvern/releases). Keep `bin/wyvern`, `bin/wyvern-viewer`, and `share/wyvern/ui/` together when extracting an archive.

## Example

```bash
wyvern '{"type":"message","title":"Hello","message":"Wyvern works","level":"info","buttons":"ok"}'
```

Headless and CI runs use `WYVERN_VIEWER=none` or `--viewer none` (no native window). Blocking dialogs must be driven to completion by the caller.

## Documentation

- API docs: <https://docs.rs/wyvern-cli>
- Repository and full guide: <https://github.com/randlee/wyvern>
