# `wyvern` (CLI) — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## CLI Invocation

**REQ-0001** — Accept a JSON command as an inline string argument: `wyvern '{"type":"message",...}'`

**REQ-0002** — Accept a `.json` file path and load it as the command: `wyvern input.json`

**REQ-0003** — Accept a `.md` file path and open it as a markdown viewer: `wyvern my-doc.md`

**REQ-0004** — Accept JSON via stdin when no argument is provided: `echo '{...}' | wyvern`

**REQ-0005** — Support `--interactive` (alias `--persistent`) to enter a readline loop on stdin, processing one JSON command per line until `{"action":"exit"}` or window close.

**REQ-0006** — Support `--mcp` to start Wyvern as an MCP server using stdio transport.

**REQ-0007** — `show`, `hide`, and `exit` are valid only inside the `--interactive` command loop. They are not valid as single-shot CLI commands.

## Host options (c.10+)

**REQ-0115** — `--bind <ADDR:PORT>` sets the dialog HTTP bind address (default `127.0.0.1:0`). Passed to `wyvern-host`.

**REQ-0116** — `--ui-root <PATH>` sets the static UI directory (default: packaged `share/wyvern/ui/`). Passed to `wyvern-host`.

**REQ-0117** — `--viewer <embedded|none|system|chrome|safari|edge|firefox>`. **Default: `embedded`** (c.15). Env `WYVERN_VIEWER` overrides. CI uses `none`. c.10: parse enum, implement `none` only. Registry: [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md).

**REQ-0118** — `wyvern browsers list` / `wyvern browsers refresh` (c.15).

---

## Interactive Mode (Phase E)

**REQ-0120** — In `--interactive` mode, commands are processed sequentially. Blocking dialog commands retain normal modal behavior inside the loop.

**REQ-0121** — In `--interactive` mode, a blocking dialog command writes its normal JSON result to stdout on completion, then the loop continues.

**REQ-0122** — `{"action":"show"}` and `{"action":"hide"}` toggle **`wyvern-viewer`** visibility via **`wyvern` CLI** (when embedded); return `{"action":"show|hide","ok":true}`. Host HTTP server stays up. Not `HostSession` methods.

**REQ-0123** — `{"action":"exit"}` shuts down `HostSession` (host) and CLI-owned viewer, returning `{"action":"exit","ok":true}` before shutdown.
