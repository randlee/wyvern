# Wyvern

**What You View, Engine Renders Natively**

![Wyvern](docs/images/wyvern-banner.png)

> A lightweight CLI tool that opens native webview windows for user interaction and returns structured JSON results — with zero browser dependency and full MCP compatibility.

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

---

## What it does

Wyvern bridges the gap between CLI tools and rich user interaction. Pass it a JSON command, get back a JSON result. No Electron. No Chrome. Just the OS's built-in webview rendering your HTML.

The MVP API stays intentionally small:
- Blocking dialog commands: `message`, `input`, `markdown`, `question`, `wizard`
- `--interactive` lifecycle actions: `show`, `hide`, `exit`

If something feels complicated, it is usually a documentation or scope problem, not a signal to grow the API. Reviews and hardening should attack accidental complexity directly.

```bash
# Show a dialog
wyvern '{"type": "message", "title": "Deploy?", "message": "Push to production?", "buttons": "yes_no"}'
# → {"button": "yes"}

# Collect input
wyvern '{"type": "input", "title": "Branch name", "message": "Enter the branch to deploy:"}'
# → {"button": "ok", "input": "feature/my-branch"}

# Render a markdown doc
wyvern my-doc.md

# Run a multi-page wizard
wyvern '{"type": "wizard", "page": {"id": "start", "title": "Setup", "html": "wizards/setup.html"}, "config": {}}'
```

---

## Why Wyvern

| | Wyvern | Electron | OS dialogs |
|---|---|---|---|
| Bundle size | ~5MB | ~150MB | 0 |
| HTML/CSS/JS UI | ✅ | ✅ | ❌ |
| No browser required | ✅ | ❌ | ✅ |
| Custom wizards | ✅ | ✅ | ❌ |
| MCP-compatible | ✅ | ❌ | ❌ |
| JSON I/O | ✅ | custom | ❌ |

---

## Dialog types

- **`message`** — blocking modal with title, body, icon, and standard button combos (`ok`, `yes_no`, `ok_cancel`, `yes_no_cancel`, `retry_cancel`, or custom)
- **`input`** — text entry, multiline, or file/folder chooser
- **`markdown`** — styled markdown viewer (`file`, inline `content`, or `wyvern file.md` shorthand)
- **`wizard`** — multi-page wizard with browser-history navigation, driven by page descriptors plus your HTML + JSON config
- **`question`** — blocking native renderer based on Claude's public `AskUserQuestion` API

---

## Interactive mode

Wyvern can run as a persistent process, accepting a stream of JSON commands over stdin:

```bash
wyvern --interactive
```

`--interactive` is still a sequential loop. Blocking dialog commands keep their normal modal behavior inside that loop; `show`, `hide`, and `exit` are the only non-dialog commands and are valid only inside that persistent stdin loop.

Claude Code and other agents can drive it from a background shell process with no MCP required.

---

## MCP

Wyvern's JSON schema maps 1:1 to MCP tool parameters. Run it as an MCP server and the same blocking dialog commands become tool calls — with a persistent window that survives across calls. In MVP, lifecycle actions are part of `--interactive`, not public MCP tools.

```bash
wyvern --mcp
```

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

## Phase acceptance criteria (smoke — delivery rewrite c.16)

Phase C delivery rewrite (`c.9`–`c.16`) is complete when:

1. Release tarball includes `wyvern` + `wyvern-viewer` + full `share/wyvern/ui/` (all five dialog types).
2. Tag `v0.1.0` triggers the GitHub Actions release matrix (macOS aarch64/x86_64, Windows, Linux).
3. `integrate/phase-c-web-server` CI is green (build, clippy, sc-lint, Playwright with `--viewer none`).
4. Manual macOS smoke: extract release artifact and run a dialog with the default embedded viewer.

v0.1.0 is authoritative only after this sprint; historical [c5-release](docs/plans/phase-C/c5-release.md) tooling is reused here.

## Deferred

- `notification` is intentionally deferred. It is the future fire-and-forget path for ephemeral updates; MVP keeps `message` strictly blocking.

---

*Wyvern: Defy the digital chasm. Unleash native clarity.*
