# Wyvern

**What You View, Engine Renders Natively**

![Wyvern](docs/images/wyvern-banner.png)

> A lightweight CLI tool that opens native webview windows for user interaction and returns structured JSON results — with zero browser dependency and full MCP compatibility.

---

## What it does

Wyvern bridges the gap between CLI tools and rich user interaction. Pass it a JSON command, get back a JSON result. No Electron. No Chrome. Just the OS's built-in webview rendering your HTML.

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
wyvern '{"type": "wizard", "html": "wizards/setup.html", "config": {}}'
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

- **`message`** — modal with title, body, icon, and standard button combos (`ok`, `yes_no`, `ok_cancel`, `yes_no_cancel`, `retry_cancel`, or custom)
- **`input`** — text entry, multiline, or file/folder chooser
- **`markdown`** — styled markdown viewer (`wyvern file.md` shorthand)
- **`wizard`** — multi-page wizard with browser-history navigation, driven entirely by your HTML + JSON config
- **`question`** — drop-in native renderer for Claude's `AskUserQuestion` API

---

## Interactive mode

Wyvern can run as a persistent process, accepting a stream of JSON commands over stdin:

```bash
wyvern --interactive
```

Feed it content over time, ask questions, display status updates — then exit when done. Perfect for AI agent status dashboards, live progress views, or anywhere you'd otherwise use an artifact panel.

Claude Code and other agents can drive it from a background shell process with no MCP required.

---

## MCP

Wyvern's JSON schema maps 1:1 to MCP tool parameters. Run it as an MCP server and the same commands become tool calls — with a persistent window that survives across calls.

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

---

*Wyvern: Defy the digital chasm. Unleash native clarity.*
