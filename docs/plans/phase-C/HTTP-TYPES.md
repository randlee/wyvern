# HTTP host — Rust and wire types (Phase C+)

Authoritative planning types for `wyvern-host`, HTTP routes, and viewer registry. Implementation may live in the cited crates; this doc is the cross-sprint contract index.

**Related:** [http-dialog-contract.md](http-dialog-contract.md), [http-post-schema.md](http-post-schema.md), [http-viewer-contract.md](http-viewer-contract.md), [http-wizard-contract.md](http-wizard-contract.md), [http-interactive-mcp-contract.md](http-interactive-mcp-contract.md).

---

## `wyvern-schema` — command ingress and stdout

Defined in `crates/wyvern-schema/src/command.rs` and `result.rs`. Unchanged wire shapes; host exposes command fields at `GET /api/dialog` and accepts POST bodies that deserialize to these types.

```rust
/// Validated CLI command — one variant per dialog `type`.
pub enum Command {
    Chrome(ChromeCommand),
    Message(MessageCommand),
    Input(InputCommand),
    Markdown(MarkdownCommand),
    Question(QuestionCommand),
    Wizard(WizardCommand), // Phase D — validation in d.1
}

/// Successful stdout JSON — POST /api/result body matches this per active type.
#[serde(untagged)]
pub enum CommandResult {
    Chrome(ChromeResult),
    Message(MessageResult),
    Markdown(MarkdownResult),
    Input(InputResult),
    Question(QuestionResult),
    Wizard(WizardResult), // Phase D — validation in d.1
}

pub struct ChromeResult { pub button: ButtonLabel }
pub struct MessageResult { pub button: ButtonLabel }
pub struct MarkdownResult { pub button: ButtonLabel }

#[serde(untagged)]
pub enum InputValue {
    Text(String),
    Paths(Vec<String>),
}

pub struct InputResult {
    pub button: ButtonLabel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputValue>,
}

pub struct QuestionResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<ButtonLabel>,
    pub questions: Vec<serde_json::Value>,
    pub answers: HashMap<String, String>,
    pub response: String,
}
```

---

## `wyvern-host` — one-shot entry (Phase C)

```rust
/// CLI / one-shot invocation options (from `--bind`, `--ui-root`, `--viewer`).
#[derive(Debug, Clone)]
pub struct HostOptions {
    pub bind: SocketAddr,           // default 127.0.0.1:0
    pub ui_root: PathBuf,           // default share/wyvern/ui/
    pub viewer: ViewerMode,
    pub dialog_url_env: bool,       // set WYVERN_DIALOG_URL when viewer is None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerMode {
    Embedded,   // product default (c.15)
    None,       // CI / headless (c.10+)
    System,
    Named(BrowserId), // Chrome | Safari | Edge | Firefox | …
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserId {
    Chrome,
    Safari,
    Edge,
    Firefox,
    // catalog may add brave, chromium, opera, vivaldi — see http-viewer-contract.md
}

/// One-shot convenience **inside `wyvern-host`** for viewer modes the host owns:
/// `None` (set `WYVERN_DIALOG_URL` only), `System`, and `Named(_)` (host calls
/// `browser_launch` then awaits). **Must not** be used for `ViewerMode::Embedded` —
/// embedded spawn is CLI-owned; use `begin` + CLI `embedded_viewer_spawn`.
pub fn run(command: Command, options: HostOptions) -> Result<CommandResult, HostError>;

/// One-shot bind (c.15+) — host serves dialog; returns handle before external launch.
/// **Required** for `ViewerMode::Embedded` (CLI spawns viewer between `begin` and `await_result`).
/// Also valid for `None` when the CLI wants explicit two-phase control.
pub fn begin(command: Command, options: HostOptions) -> Result<DialogHandle, HostError>;

/// Two-phase handoff for **all** modes that need an external step after bind
/// (required for `Embedded`; also usable when the CLI wants explicit control).
/// Host binds and exposes URL; caller launches viewer; then awaits result.
#[derive(Debug)]
pub struct DialogHandle {
    /// Full dialog URL, e.g. `http://127.0.0.1:PORT/message/`
    pub dialog_url: String,
    /// Optional window hints from command (width/height) for `wyvern-viewer` launch.
    pub viewer_options: ViewerLaunchOptions,
    // await_result() consumes self — not serializable
}

impl DialogHandle {
    /// Block until `POST /api/result` (or wizard finish) or dismiss timeout.
    pub fn await_result(self) -> Result<CommandResult, HostError>;
    /// CLI-only fallback: `wyvern-viewer` child exited without posting a result.
    /// Returns `Ok(CommandResult)` with dismissed semantics for the active type (REQ-0068).
    pub fn viewer_exited_without_result(self) -> Result<CommandResult, HostError>;
}

#[derive(Debug, Clone, Default)]
pub struct ViewerLaunchOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Completed one-shot run — convenience when viewer is `none`/`system`/`named`
/// (no CLI embedded-spawn step). Not used for `Embedded`.
#[derive(Debug)]
pub struct HostRunOutcome {
    pub result: CommandResult,
    pub dialog_url: String,
    pub viewer_options: ViewerLaunchOptions,
}
```

---

## `wyvern-host` — persistent session (Phase E)

```rust
pub struct HostSession {
    // HTTP listener, ui_root, session state — NO wyvern-viewer child (CLI-owned)
}

impl HostSession {
    pub fn new(options: HostOptions) -> Result<Self, HostError>;
    /// Bind route, return URL. When `options.viewer` is `System`/`Named`, host
    /// opens the URL via `browser_launch` before returning. When `Embedded`/`None`,
    /// no host-side launch — CLI spawns/navigates viewer or harness attaches.
    pub fn run_dialog(&mut self, command: Command) -> Result<DialogHandle, HostError>;
    pub fn shutdown(self) -> Result<(), HostError>;
}
```

**Not on `HostSession`:** `show`, `hide`, or embedded viewer spawn — owned by **`wyvern` CLI** → **`wyvern-viewer`** subprocess. See [http-viewer-contract.md](http-viewer-contract.md).

**One-shot orchestration (by viewer mode):**

| `ViewerMode` | Who launches | API |
|--------------|--------------|-----|
| `None` | nobody | `wyvern-host::run` (or `DialogHandle` + await) |
| `System` / `Named` | **`wyvern-host`** `browser_launch` | `wyvern-host::run` (one-shot) or `HostSession::run_dialog` (persistent) — host opens URL as part of the call |
| `Embedded` | **`wyvern` CLI** subprocess | CLI: `begin` → `DialogHandle` → `embedded_viewer_spawn` → `await_result` — **not** `host::run` |

---

## `HostError` (planning — finalize in c.10)

```rust
#[derive(Debug)]
pub enum HostError {
    /// TCP bind failed (maps to stderr `host_bind` / exit 7).
    Bind { message: String },
    /// UI root missing or static file not found (maps to `host_error` / exit 6).
    UiNotFound { path: PathBuf },
    /// Active command type not implemented on host matrix yet (c.10–c.14).
    /// Returned at run time after validation passes — not a validation-time phase gate.
    UnsupportedType { type_name: String },
    /// POST /api/result JSON invalid for active type.
    InvalidResult { message: String },
    /// Named browser not installed (`HOST_VIEWER_ERROR`).
    ViewerNotFound { id: String, hint: String },
    /// Internal server fault.
    Internal { message: String },
}
```

CLI maps `HostError` via `emit_host_error` in `crates/wyvern/src/error.rs` (replaces `RunError` / `wyvern-window` after c.9).

---

## HTTP route payloads

### `GET /api/dialog`

Response: JSON object — serialized fields from the active `Command` variant (no HTML rendering).

```rust
/// Example: Message — other types expose their command fields similarly.
#[derive(Serialize)]
pub struct DialogPayloadMessage {
    #[serde(rename = "type")]
    pub type_name: &'static str, // "message"
    pub title: String,
    pub message: String,
    pub buttons: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    // markdown: content_html added server-side (c.12)
}

/// Markdown — host adds sanitized `content_html` (pulldown-cmark + ammonia).
#[derive(Serialize)]
pub struct DialogPayloadMarkdown {
    #[serde(rename = "type")]
    pub type_name: &'static str, // "markdown"
    pub title: String,
    pub content: String,          // raw markdown from command
    pub content_html: String,     // pulldown-cmark → ammonia sanitize → JSON field
    pub buttons: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Question option — `preview` markdown rendered server-side to `preview_html` (c.13).
#[derive(Serialize)]
pub struct QuestionOptionPayload {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,       // raw markdown from command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_html: Option<String>,  // ammonia-sanitized HTML
}

/// Question dialog payload — options use `QuestionOptionPayload` when serialized.
#[derive(Serialize)]
pub struct DialogPayloadQuestion {
    #[serde(rename = "type")]
    pub type_name: &'static str, // "question"
    pub title: String,
    pub questions: Vec<serde_json::Value>, // wire: QuestionOptionPayload per option
    pub buttons: String,
}
```

Session model: **one in-process session per CLI invocation**; no session id in URL for one-shot. Path `/{type}/` selects the template tree only.

### `POST /api/result` — ack

```rust
#[derive(Serialize, Deserialize)]
pub struct ResultAck {
    pub ok: bool,
}
```

Body **request** equals `CommandResult` wire JSON per [http-post-schema.md](http-post-schema.md).

### Picker helpers (c.11 — `input` only)

```rust
#[derive(Deserialize)]
pub struct PickerFileRequest {
    pub filter: Option<Vec<String>>,
    pub multiple: Option<bool>,
    pub start_path: Option<String>,
}

#[derive(Deserialize)]
pub struct PickerFolderRequest {
    pub start_path: Option<String>,
}

#[derive(Serialize)]
pub struct PickerResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
}
```

---

## Browser registry JSON (c.15)

On-disk cache — not serde types in `wyvern-schema`. Owned by `wyvern-host/src/browser_registry.rs`.

```rust
#[derive(Serialize, Deserialize)]
pub struct BrowserRegistryFile {
    pub version: u32,
    pub updated_at: String,      // RFC3339
    pub platform: String,        // e.g. "macos-aarch64"
    pub entries: Vec<BrowserRegistryEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct BrowserRegistryEntry {
    pub id: String,              // catalog id: "chrome", "firefox", …
    pub name: String,
    pub executable: PathBuf,
}
```

---

## Wizard HTTP types (Phase D)

See [http-wizard-contract.md](http-wizard-contract.md).

**Crate ownership:**

| Type | Crate |
|------|-------|
| `WizardCommand`, `WizardResult`, `WizardPageDescriptor`, `WizardPageLayout`, `WizardStackEntry` | `wyvern-schema` |
| `WizardSession`, `WizardSnapshot`, `NavigateOutcome`, `WizardError` | `wyvern-wizard` |
| `WizardStateResponse`, `WizardNavigateRequest`, `WizardFinishRequest` | `wyvern-schema` (wire DTOs built from `snapshot()` / `NavigateOutcome`) |

```rust
/// Minimal page descriptor — REQ-0026.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WizardPageDescriptor {
    pub id: String,
    pub title: String,
    pub html: String,
    /// Per-page layout: `dialog` (default) or `workspace` (large-canvas HTML pages). Phase D d.5–d.6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<WizardPageLayout>,
}

/// `dialog` = typical form step; `workspace` = HTML page requesting viewport-sized canvas (example: graph editor).
pub enum WizardPageLayout { Dialog, Workspace }

/// One stack entry — REQ-0024.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WizardStackEntry {
    pub page: WizardPageDescriptor,
    pub data: serde_json::Value,
}

/// Wizard command ingress — validated in d.1.
/// Static HTML paths resolve from `page.html` relative to `--ui-root` (no separate `page_html` field).
pub struct WizardCommand {
    #[serde(rename = "type")]
    pub type_name: &'static str, // "wizard"
    pub page: WizardPageDescriptor,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

/// GET /api/wizard/state shape — `stack` = prior entries only (REQ-0024).
pub struct WizardSnapshot {
    pub config: serde_json::Value,
    pub page: WizardPageDescriptor,
    pub page_data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>,
}

/// Host uses this after navigate to build response URL + state refresh.
pub struct NavigateOutcome {
    pub page: WizardPageDescriptor,
    pub page_data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>,
}

pub enum WizardError {
    AtFirstPage,
    InvalidCommand(String),
    StackMismatch,
}

/// Wizard stdout — POST /api/wizard/finish body matches this.
pub struct WizardResult {
    pub button: ButtonLabel, // finish | cancel | dismissed
    pub data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>,
}

#[derive(Serialize)]
pub struct WizardStateResponse {
    #[serde(rename = "type")]
    pub type_name: &'static str,
    pub config: serde_json::Value,
    pub page: WizardPageDescriptor,
    pub page_data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

/// Wire: `"next"` | `"back"` only. Cancel/finish/dismissed use `POST /api/wizard/finish`.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardNavAction {
    Next,
    Back,
}

#[derive(Deserialize)]
pub struct WizardNavigateRequest {
    pub action: WizardNavAction,
    #[serde(default)]
    pub data: serde_json::Value,
    pub page_id: Option<String>,
    pub next: Option<WizardPageDescriptor>,
}

#[derive(Deserialize)]
pub struct WizardFinishRequest {
    pub button: ButtonLabel,
    pub data: serde_json::Value,
    pub stack: Vec<WizardStackEntry>,
}
```

---

## Sprint ownership (types land in code)

| Types / routes | Sprint |
|----------------|--------|
| `HostOptions`, `HostError`, `run()`, `GET /api/dialog`, `POST /api/result` (`message`) | c.10 |
| Picker request/response, `input` payload | c.11 |
| `content_html` in dialog payload | c.12 |
| `DialogPayloadMarkdown`, `DialogPayloadQuestion` (`preview_html`) | c.12–c.13 |
| `question` result validation | c.13 |
| `chrome` payload | c.14 |
| `ViewerMode`, `BrowserRegistryFile`, `DialogHandle`, `ViewerLaunchOptions` | c.15 |
| `HostSession` | Phase E (e.1) |
| `WizardCommand`, `WizardResult`, validators | d.1 |
| Wizard state/navigate/finish routes | d.1–d.2 |
