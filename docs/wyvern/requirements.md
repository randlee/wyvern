# `wyvern` (CLI) — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## CLI Invocation

**REQ-0001** — Accept a JSON command as an inline string argument: `wyvern '{"type":"message",...}'`

**REQ-0002** — Accept a `.json` file path and load it as the command: `wyvern input.json`

**REQ-0003** — Accept a `.md` file path and open it as a markdown viewer: `wyvern my-doc.md`

**REQ-0004** — Accept JSON via stdin when no argument is provided: `echo '{...}' | wyvern`

**REQ-0005** — Support `--interactive` (alias `--persistent`) to enter a readline loop on stdin, processing one JSON command per line until `{"action":"exit"}` or window close.

---

## Interactive Mode

**REQ-0070** — In `--interactive` mode, display commands (`message`, `markdown`, `image`) shall be fire-and-forget — the loop immediately awaits the next command.

**REQ-0071** — In `--interactive` mode, `question` commands shall block the loop until the user answers, then write the result to stdout before continuing.

**REQ-0072** — `{"action":"show"}` and `{"action":"hide"}` shall toggle window visibility without terminating the process.

**REQ-0073** — `{"action":"exit"}` shall close the window and terminate the process cleanly.
