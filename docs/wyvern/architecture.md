# `wyvern` (CLI) — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0008: Interactive mode uses stdin readline loop

**Status:** Accepted

**Context:**
A persistent Wyvern window needs to receive updates over time. Options: named pipe/Unix socket, local HTTP server, or stdin readline loop.

**Decision:**
`--interactive` flag puts Wyvern into a readline loop on stdin. Each newline-delimited JSON object is a command. Blocking dialog commands retain their normal modal behavior inside the loop; `show`, `hide`, and `exit` are lifecycle actions for that loop. Results go to stdout on completion. Process exits on `{"action":"exit"}` or window close. `--persistent` is an alias.

**Consequences:**
- No socket setup or port conflicts
- Works identically in CLI and MCP modes
- Any agent or script can drive it by holding stdin/stdout handles open (background shell pattern)
- Sequential — commands processed one at a time (sufficient for UI interaction cadence)
