# `wyvern-schema` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0013 (local): Incremental validation + protocol types

Validation grows with each phase's executable `Command` enum variants. Phase A: `chrome` only.

```rust
pub enum Command {
    Chrome { title: String, status: Option<String> },
}

pub enum ValidationError {
    Validation { field: String, message: String },
    State { field: String, message: String },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum CommandResult {
    Chrome(ChromeResult),
    // Phase B+: Input(InputResult), Wizard(WizardResult), etc.
}

#[derive(Serialize)]
pub struct ChromeResult {
    pub button: String,
}

// Phase A wire: {"button":"dismissed"} via #[serde(untagged)] + ChromeResult
// Serialize-only protocol: overlapping {button} shapes across variants are intentional.

pub fn validate(value: &serde_json::Value) -> Result<Command, ValidationError>;
```

`CommandResult` is an extensible protocol enum in `wyvern-schema`. Phase A adds only `Chrome(ChromeResult)`; later phases add variants without changing the chrome wire shape.

Parse/io errors are **not** `ValidationError` — they are `LoadError` in `crates/wyvern` (see `docs/plans/phase-A/README.md`).

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
