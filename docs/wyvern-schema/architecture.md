# `wyvern-schema` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0007: Base `question` on Claude's public AskUserQuestion API

**Status:** Accepted

**Context:**
Wyvern needs a question dialog type. Claude's public `AskUserQuestion` API already defines the core fields and behavior for this problem. Options: define a new Wyvern-specific schema, or adopt the Claude API inside Wyvern's standard command envelope.

**Decision:**
Adopt the public Claude `AskUserQuestion` fields and behavior for Wyvern's `question` command, while keeping Wyvern's normal top-level `type: "question"` envelope.

**Consequences:**
- Best-effort compatibility with Claude Code hooks without reinventing a second question schema
- Can be used standalone with no Claude dependency
- Future extensions must remain backward-compatible with the public Claude API semantics where possible
- Question semantics follow the public AskUserQuestion contract rather than a Wyvern-specific redesign. Multi-step questionnaires remain wizard territory.
