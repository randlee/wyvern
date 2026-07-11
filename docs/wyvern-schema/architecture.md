# `wyvern-schema` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0007: Adopt Claude AskUserQuestion schema verbatim for `question` type

**Status:** Accepted

**Context:**
Wyvern needs a question/multiple-choice dialog type. Claude's `AskUserQuestion` tool already defines a well-specified schema. Options: define a custom Wyvern schema, or adopt Claude's verbatim.

**Decision:**
Adopt the Claude `AskUserQuestion` JSON schema exactly — same input, same output. Wyvern becomes a drop-in native renderer for Claude's own tool calls.

**Consequences:**
- Zero translation layer when intercepting Claude's `AskUserQuestion` calls
- Can be used standalone with no Claude dependency
- Future extensions must remain backward-compatible with the Claude API
- Limited to 1–4 questions, 2–4 options each (current Claude API constraints)
