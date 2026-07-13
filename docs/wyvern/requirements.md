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

---

## Interactive Mode

**REQ-0100** — In `--interactive` mode, commands are processed sequentially. Blocking dialog commands retain normal modal behavior inside the loop.

**REQ-0101** — In `--interactive` mode, a blocking dialog command writes its normal JSON result to stdout on completion, then the loop continues.

**REQ-0102** — `{"action":"show"}` and `{"action":"hide"}` toggle window visibility; return `{"action":"show|hide","ok":true}`.

**REQ-0103** — `{"action":"exit"}` closes the window and terminates cleanly, returning `{"action":"exit","ok":true}` before shutdown.
