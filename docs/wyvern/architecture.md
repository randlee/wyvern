# `wyvern` (CLI) — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0013 (local): CLI obeys direct dispatch

`wyvern` binary is a thin entry point. It does not embed routing logic beyond:

1. `load_input(argv, stdin) -> Value`
2. `wyvern_schema::validate(value) -> Command`
3. `wyvern_window::run(command) -> CommandResult`
4. `emit_stdout(result)`

Library crates own behavior. The binary wires I/O only.

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
