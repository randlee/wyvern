# `wyvern-schema` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0013 (local): Incremental validation + protocol types

Validation grows with each phase's executable `Command` enum variants. Phase A: `chrome` only. Phase B adds `message`, `input`, `markdown`, and `question` incrementally per sprint (see `docs/plans/phase-B/README.md`).

```rust
pub enum Command {
    Chrome { title: ChromeTitle, status: Option<ChromeStatus> },
    // Phase B — unlocked per sprint b.1–b.8
    Message { /* title, message, buttons, ... */ },
    Input { /* title, message, mode, ... */ },
    Markdown { /* file or content, title, buttons, ... */ },
    Question { /* questions: Vec<QuestionCard> */ },
}

pub enum ValidationError {
    Validation { field: String, message: String },
    State { field: String, message: String },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum CommandResult {
    Chrome(ChromeResult),
    Message(MessageResult),
    Input(InputResult),
    Markdown(MarkdownResult),
    Question(QuestionResult),
    // Phase D+: Wizard(WizardResult), etc.
}

#[derive(Serialize)]
pub struct ChromeResult {
    pub button: ButtonLabel,
}

#[derive(Serialize)]
pub struct MessageResult {
    pub button: ButtonLabel,
}

#[derive(Serialize)]
pub struct InputResult {
    pub button: ButtonLabel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputValue>, // string or path array for multiple
}

#[derive(Serialize)]
pub struct MarkdownResult {
    pub button: ButtonLabel,
}

#[derive(Serialize)]
pub struct QuestionResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<ButtonLabel>, // present only on force close (REQ-0068)
    pub questions: Vec<serde_json::Value>,
    pub answers: std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub response: String,
}

// Phase A wire: {"button":"dismissed"} via #[serde(untagged)] + ChromeResult
// message/markdown wire: {"button":"<label>"} — overlapping {button} shapes intentional.
// Serialize-only protocol.

pub fn validate(value: &serde_json::Value) -> Result<Command, ValidationError>;
```

**Newtypes from Phase A** (`ButtonLabel`, `ChromeTitle`, `ChromeStatus`, `FieldName`, etc.) apply to button and chrome fields on all Phase B variants. Construct validated values only at the `validate()` boundary; downstream `Command` and `CommandResult` types carry newtypes, not raw `String`, for protocol fields already introduced in Phase A.

`CommandResult` is an extensible protocol enum in `wyvern-schema`. Phase A adds `Chrome(ChromeResult)`; Phase B adds dialog variants without changing existing wire shapes.

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
