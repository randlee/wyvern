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
- **`wyvern-wizard`** owns history logic (cursor, truncate, restore) inside private modules
- **`wyvern-host`** owns HTTP session storage of `Box<dyn WizardNavigator>` and serializes `snapshot()` only
- Pages direct navigation by returning their own descriptor plus an explicit next-page descriptor when advancing
- Slightly more complex than a simple stack but well-understood (browser model)

---

## ADR-0007: Wizard logic exposed only through traits

**Status:** Accepted (planning)

**Context:**
Exposing `BrowserHistory` internals to `wyvern-host` would couple HTTP routes to navigation implementation details and block future refactors (e.g. alternative history models for tests).

**Decision:**
- Public API of `wyvern-wizard` is trait-based: `WizardEngine`, `WizardNavigator`
- Concrete history types (`BrowserHistory`, `HistoryEntry`) live in private modules
- `wyvern-host` may depend on `wyvern-wizard` but imports **only** `lib.rs` re-exports
- Integration tests outside `wyvern-wizard` use HTTP or trait mocks — never `browser_history` internals

**Consequences:**
- Host route handlers stay thin serializers around trait calls
- d.3 can replace stub history without host changes
- Boundaries enforce `wizard_engine_trait` ownership in `boundaries/wyvern-wizard/wizard.toml`

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
