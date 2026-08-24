# Wyvern

**What You View, Engine Renders Natively**

![Wyvern](docs/images/wyvern-banner.png)

> A lightweight CLI tool that opens native webview windows for user interaction and returns structured JSON results — with zero browser dependency, declarative CLI extensions, and an MCP-ready JSON schema (MCP server ships in Phase E).

**Current release:** [v0.4.0](CHANGELOG.md#040--2026-08-24) — Phase H XHTML reporting (view + review) on the extension runtime (wizard flows since v0.2.0).

---

## Quickstart

1. Download the latest release for your platform from [GitHub Releases](https://github.com/randlee/wyvern/releases).
2. Extract the archive. Keep `wyvern`, `wyvern-viewer`, and `share/wyvern/ui/` together (same layout as the tarball).
3. Add the extract directory to your `PATH` (so both binaries resolve as siblings).
4. Try (default viewer is **embedded** — launches `wyvern-viewer`):

```bash
wyvern '{"type":"message","title":"Hello","message":"Wyvern works","level":"info","buttons":"ok"}'
wyvern '{"type":"input","title":"Name","message":"Enter your name","default":""}'
wyvern '{"type":"markdown","content":"# Hello\n\nFrom **Wyvern**."}'
```

**HTTP host notes**

- Dialogs are served by an ephemeral local HTTP host (`wyvern-host`) from packaged `share/wyvern/ui/`.
- Product default: `--viewer embedded` (optional `wyvern-viewer` sibling binary).
- CI / agents / headless: set `WYVERN_VIEWER=none` or pass `--viewer none` (no native window).

```bash
WYVERN_VIEWER=none wyvern '{"type":"message","title":"CI","message":"headless","buttons":"ok"}'
```

Release artifacts (no clone required):

| Platform | Artifact |
|----------|----------|
| macOS Apple Silicon | `wyvern-macos-aarch64.tar.gz` |
| macOS Intel | `wyvern-macos-x86_64.tar.gz` |
| Windows x86_64 | `wyvern-windows.zip` |
| Linux x86_64 | `wyvern-linux.tar.gz` |

Each archive contains `wyvern`, `wyvern-viewer`, and `share/wyvern/ui/` (message, input, markdown, question, chrome).

## Quick examples

```bash
# Discover shipped skills (copy-paste examples)
wyvern help
wyvern --help

# Skill catalog (text, JSON, or detail view)
wyvern extensions list
wyvern extensions list --json
wyvern extensions show csv-suffix

# Extension-specific help at match time
wyvern compose render --help

# Open a markdown file as a dialog
wyvern doc.md

# Open a custom HTML wizard page (auto-infers --ui-root)
wyvern examples/wizards/single-page/pages/only.html

# Load a wizard from wizard.json (auto-infers --ui-root)
wyvern examples/wizards/turbo-flow/wizard.json

# Interactive CSV table (sort / filter / Finish → JSON)
# Requires `python3` on PATH. On Windows, install Python 3 and ensure the
# `python3` command resolves (the Windows `py` launcher is not used).
wyvern fixtures/sample.csv
wyvern table fixtures/sample.csv

# CSV as a markdown pipe table
wyvern md fixtures/sample.csv
```

## Optional: Compose render

If [`sc-compose`](https://crates.io/crates/sc-compose) is installed, wyvern can render Jinja2 templates to HTML previews:

```bash
wyvern compose render --root ./my-template-dir --file page.j2
```

---

## What it does

Wyvern bridges the gap between CLI tools and rich user interaction. Pass it a JSON command, get back a JSON result — or use argv shorthands for common file types and prefix skills. No Electron. No Chrome. Just the OS's built-in webview rendering your HTML.

**v0.3.0** adds declarative CLI extensions and agent-facing discoverability on top of the core dialog API:

- Blocking dialog commands: `message`, `input`, `markdown`, `question`, `chrome`
- Multi-page **`wizard`** flows with browser-history navigation (since v0.2.0)
- **Extensions** — suffix and prefix argv skills (`.html`, `.csv`, `compose render`, `md`, and more via bundled registry)

```bash
# Show a dialog
wyvern '{"type": "message", "title": "Deploy?", "message": "Push to production?", "buttons": "yes_no"}'
# → {"button": "yes"}

# Collect input
wyvern '{"type": "input", "title": "Branch name", "message": "Enter the branch to deploy:"}'
# → {"button": "ok", "input": "feature/my-branch"}

# Render a markdown doc
wyvern my-doc.md
```

---

## Why Wyvern

| | Wyvern | Electron | OS dialogs |
|---|---|---|---|
| Bundle size | ~5MB | ~150MB | 0 |
| HTML/CSS/JS UI | ✅ | ✅ | ❌ |
| No browser required | ✅ | ❌ | ✅ |
| Custom wizards | ✅ | ✅ | ❌ |
| Declarative CLI extensions | ✅ | ❌ | ❌ |
| MCP-compatible | Phase E | ❌ | ❌ |
| JSON I/O | ✅ | custom | ❌ |

---

## Dialog types

- **`message`** — blocking modal with title, body, icon, and standard button combos (`ok`, `yes_no`, `ok_cancel`, `yes_no_cancel`, `retry_cancel`, or custom)
- **`input`** — text entry, multiline, or file/folder chooser
- **`markdown`** — styled markdown viewer (`file`, inline `content`, or `wyvern file.md` shorthand)
- **`question`** — blocking native renderer based on Claude's public `AskUserQuestion` API
- **`chrome`** — foundation chrome frame and platform safe zones (used by other dialog types)
- **`wizard`** — multi-page flows with stack navigation (`POST /api/wizard/navigate`, `finish`, visited-stack JSON on dismiss)

---

## Platform support

| Platform | Engine | Load time | Memory |
|----------|--------|-----------|--------|
| macOS | WebKit (system) | ~instant | ~30–50MB |
| Windows | WebView2 | fast | ~40–60MB |
| Linux | WebKitGTK | moderate | ~100–150MB |

---

## Docs

- [PRD](docs/prd/wyvern-prd.md) — full product requirements and JSON schema reference
- [CHANGELOG](CHANGELOG.md) — release history

## Deferred (post–v0.3.0)

- **`--interactive`** — persistent stdin loop with `show`, `hide`, and `exit` lifecycle actions (Phase E)
- **`wyvern --mcp`** — MCP server; JSON schema is MCP-ready today, binary ships Phase E
- **User extension registry** — `~/.config/wyvern/extensions.json` (post–Phase F)
- **`notification`** — future fire-and-forget path for ephemeral updates; `message` stays blocking

---

*Wyvern: Defy the digital chasm. Unleash native clarity.*
