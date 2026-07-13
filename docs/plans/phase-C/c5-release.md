---
id: c.5
title: Release tooling and v0.1.0
status: complete
branch: feature/phase-C-c5-release
worktree: /Volumes/Extreme Pro/github/wyvern-worktrees/feature/phase-C-c5-release
target: integrate/phase-C
---

# Sprint c.5 — Release tooling and v0.1.0

## Goal

- Ship Wyvern v0.1.0: GitHub Actions release matrix, README quickstart, `CHANGELOG.md`, tag `v0.1.0`.
- Phase C phase acceptance: installable binary + all Phase B dialog types from release artifact.

## Hard Dependencies

- c.4 NFR pass complete — do not tag if NFR-0003 fails without documented waiver
- `integrate/phase-C` merged to `develop` (or release branch policy per team)

## Exact Targets

- `.github/workflows/release.yml` — new workflow (tag-triggered)
- `README.md` — quickstart section
- `CHANGELOG.md` — v0.1.0 entry
- `Cargo.toml` — workspace version `0.1.0` (root + published crates as applicable)
- Optional: `brew/` formula stub or install script — **not required** for sprint closure if README documents GitHub release download

## Deliverables

- Push tag `v0.1.0` triggers matrix build: **macOS aarch64 + x86_64** (dual matrix jobs), **Windows x86_64**, **Linux x86_64**
- Release artifacts attached to GitHub Release automatically (`.tar.gz` / `.zip` with `wyvern` binary)
- README quickstart: install from release + **3 example commands** runnable in < 5 minutes:
  1. `message` with level icon
  2. `input` text mode
  3. `markdown` inline or file
- `CHANGELOG.md` entry summarizing Phase A–C scope
- CI unchanged for PRs: existing `.github/workflows/ci.yml` remains PR gate; release workflow is tag-only

## Required Work — release workflow (authoritative)

### Trigger

```yaml
on:
  push:
    tags:
      - 'v*'
```

### Matrix (minimum)

**macOS approach:** dual matrix jobs (one per architecture) — not a universal binary via `lipo`. Produces separate artifacts consumers download for their arch.

| OS | Target | Artifact |
|----|--------|----------|
| `macos-latest` | `aarch64-apple-darwin` | `wyvern-macos-aarch64.tar.gz` |
| `macos-latest` | `x86_64-apple-darwin` | `wyvern-macos-x86_64.tar.gz` |
| `windows-latest` | `x86_64-pc-windows-msvc` | `wyvern-windows.zip` |
| `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `wyvern-linux.tar.gz` |

Build: `cargo build --release -p wyvern --target ${{ matrix.target }}`

Linux release build uses same webview deps as CI (`libwebkit2gtk-4.1-dev`).

### README quickstart template

```markdown
## Quickstart

1. Download the latest release for your platform from GitHub Releases.
2. Extract and add `wyvern` to your PATH.
3. Try:
   wyvern '{"type":"message","title":"Hello","message":"Wyvern works","level":"info","buttons":"ok"}'
   wyvern '{"type":"input","title":"Name","message":"Enter your name","default":""}'
   wyvern '{"type":"markdown","content":"# Hello\n\nFrom **Wyvern**."}'
```

### Version bump

- Workspace `version = "0.1.0"` in root `Cargo.toml`
- Binary `--version` reflects crate version

## Explicit Code Samples

```yaml
# .github/workflows/release.yml (skeleton)
jobs:
  release:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: wyvern-macos-aarch64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: wyvern-macos-x86_64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: wyvern-windows
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: wyvern-linux
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      # Linux: install webkit deps (same as ci.yml)
      - run: cargo build --release -p wyvern --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v2
        with:
          files: packaged artifact path
```

## This Sprint Does Not Close

- `brew install wyvern` official Homebrew core (stretch goal — phase acceptance says "or equivalent")
- crates.io publish — post-v0.1.0 if desired
- Code signing / notarization — post-MVP polish

## Acceptance Criteria

- Tag `v0.1.0` produces GitHub Release with **four** platform artifacts (macOS aarch64, macOS x86_64, Windows, Linux)
- Fresh download + quickstart commands succeed on macOS (authoritative); Win/Linux via CI-built artifact smoke
- `CHANGELOG.md` v0.1.0 section lists Phase B dialog types + Phase C icons/chrome
- README install path documented without requiring repo clone
- PR CI (`ci.yml`) still passes on release branch

## Required Validation

- Dry-run release workflow on pre-release tag (`v0.1.0-rc.1`) optional but recommended
- `cargo build --release -p wyvern` on all three platforms (or matrix job green)
- Manual quickstart walkthrough on macOS
- `cargo test --workspace -- --test-threads=1` on `integrate/phase-C` head before tag
- `cargo clippy --workspace -- -D warnings`
