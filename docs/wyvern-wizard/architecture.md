# `wyvern-wizard` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0005: Wizard navigation uses browser-history model

**Status:** Accepted

**Context:**
A simple push/pop stack loses forward history on back-navigation, forcing users to re-enter data if they go back then forward on the same path.

**Decision:**
Cursor-over-array browser-history model:
- Back moves cursor back without discarding forward entries
- Forward to the same explicit next-page restores cached data
- Forward to a different explicit next-page truncates forward history and pushes the new page

**Consequences:**
- Users explore back/forward freely without losing entered data
- Branching correctly clears stale forward history
- Host owns the history array and cursor; pages direct navigation by returning their own descriptor plus an explicit next-page descriptor when advancing
- Slightly more complex than a simple stack but well-understood (browser model)

---

## ADR-0006: Host is domain-agnostic — wizard data is opaque

**Status:** Accepted

**Context:**
Wyvern could interpret wizard page data (validate fields, understand DAG structure). This would couple the host to specific use-cases.

**Decision:**
Host stores and passes through page-specific `data` blobs without inspection. All domain logic lives in HTML/JS. Host only manages page descriptors, navigation signals (`next`, `back`, `finish`, `cancel`), and the history stack.

**Consequences:**
- Any wizard can be built without changing Wyvern
- Pages inspect the full stack via JS (`window.wyvern.stack`) for context-aware decisions
- Wyvern ships no wizard-specific business logic
- Validation of wizard data is the caller's responsibility
