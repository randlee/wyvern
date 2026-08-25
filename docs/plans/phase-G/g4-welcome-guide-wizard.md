---
id: g.4
title: Welcome guide wizard (`wyvern guide`)
status: complete (integrate)
branch: feature/phase-G-g4-welcome-guide
worktree: ../wyvern-worktrees/feature/phase-G-g4-welcome-guide
target: integrate/phase-G
---

# Sprint g.4 — Welcome guide wizard

## Goal

Ship `wyvern guide` (REQ-0127) and the CLI workflow foundation used by g.5–g.7: `workflow.pre` / `workflow.post` (REQ-0124, REQ-0125, ADR-0023) and the `next_wizard` chain loop (REQ-0126, ADR-0024).

## Hard dependencies

- Phase F merged (extension registry, `{wyvern_share}`, `preexec` spawn discipline)
- Wave 1 g.1 merged (`--help` already mentions `wyvern guide`)

## Deliverables

| Path | Purpose |
|------|---------|
| `crates/wyvern-schema/src/wizard.rs` | `WorkflowSpec` on `WizardCommand`; `NextWizard` on `WizardFinishRequest` and `WizardResult` |
| `crates/wyvern-schema/src/validate/wizard.rs` | Validate `workflow` shape; populate `WizardCommand.workflow` |
| `crates/wyvern-schema/src/validate/helpers.rs` | Add `workflow` to `WIZARD_FIELDS` (REQ-0053) |
| `crates/wyvern-host/src/routes/wizard.rs` | Copy `next_wizard` from `WizardFinishRequest` onto `WizardResult`; do not resolve or execute |
| `crates/wyvern/src/workflow/mod.rs` | `WorkflowRunner` calls Phase F `extensions/preexec` spawn / timeout / stderr-tail helpers only — no second subprocess stack |
| `crates/wyvern/src/workflow/chain.rs` | `resolve_next_wizard` → `NextInvocation`; depth 16 |
| `crates/wyvern/src/pipeline.rs` | `run_from_loaded`: every `Command::Wizard` one-shot enters `run_wizard_workflow_loop`; other types stay on the existing host path |
| `crates/wyvern/src/cli_args.rs` | `--workflow-dry-run` on `CliArgs` (not `HostOptions`) |
| `crates/wyvern-schema/src/error_code.rs` | Add `ErrorCode::WorkflowError` (`WORKFLOW_ERROR`, slug `workflow`, exit 9) |
| `crates/wyvern/src/error/emit.rs` | Emit stderr via `ErrorCode::WorkflowError` — no hand-built slug (`WorkflowError::subcode()` in the envelope) |
| `crates/wyvern/Cargo.toml` | Incremental on f.1: vendor `share/wyvern/**` + `scripts/ext/**` under `crates/wyvern/embedded/` for rust-embed so `cargo publish --dry-run` can compile the tarball; keep the glob wide |
| `boundaries/wyvern/cli.toml` | `io_owns` += `workflow_script_spawn`, `wizard_chain_loop` |
| `boundaries/wyvern-host/host.toml` | `io_forbidden` += `workflow_script_spawn`, `wizard_chain_loop` |
| `crates/wyvern/tests/workflow_pre_post.rs` | Pre merge; post stdin; dry-run argv; allowlist deny; cancel skips post |
| `crates/wyvern/tests/workflow_chain.rs` | Two-fixture chain; 17th hop fails; stdout omits `next_wizard` |
| `crates/wyvern/tests/guide_extension.rs` | `wyvern guide` expands welcome `wizard.json` |
| `crates/wyvern-host/tests/wizard_next_wizard_passthrough.rs` | Finish request with `next_wizard` is copied onto `WizardResult` |
| `share/wyvern/welcome/` | Hub `wizard.json`, home page, four topic pages — Overview is a **terminal** page with Back/Finish chrome (`data-wizard-terminal="true"`); AskUserQuestion, Template wizard, and Agent DAG are **bridge** pages with full copy, Back/Finish chrome, and `wizardNextWizard` hops to the g.5 / g.6 / g.7 example wizards |
| `share/wyvern/extensions.json` | `guide` argv-prefix entry |

### Boundary contracts

```rust
pub const WORKFLOW_SCRIPT_TIMEOUT: Duration = Duration::from_secs(30);
pub const NEXT_WIZARD_MAX_DEPTH: u32 = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre: Option<WorkflowPath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<WorkflowPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextWizard {
    pub path: WorkflowPath,
    #[serde(default)]
    pub input: serde_json::Value, // default {}
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_root: Option<WorkflowPath>,
}

pub struct Allowlist {
    pub share_root: PathBuf,
    pub cwd: PathBuf,
    pub wizard_dir: PathBuf,
}

impl Allowlist {
    /// Expand `{wyvern_share}`, canonicalize, reject `..` and symlink escape.
    pub fn resolve_allowed(&self, raw: &str) -> Result<PathBuf, WorkflowError>;
}

pub struct WorkflowRunner {
    pub allowlist: Allowlist,
    pub timeout: Duration,
}

/// Spawn env for every pre and post script (inherit + these keys):
/// `WYVERN_SHARE` = allowlist.share_root
/// `WYVERN_REPO_ROOT` = existing env or allowlist.cwd
/// `WYVERN_BIN` = canonical `std::env::current_exe()` of this wyvern process
///   (PATH lookup only if current_exe fails). Never the Python interpreter.
/// `HOME` = inherited
/// cwd = allowlist.cwd
/// Spawn MUST go through `extensions/preexec` helpers (ADR-0023).
impl WorkflowRunner {
    pub fn run_pre(
        &self,
        spec: &WorkflowSpec,
        config: &mut serde_json::Value,
        dry_run: bool,
    ) -> Result<(), WorkflowError>;

    pub fn run_post(
        &self,
        spec: &WorkflowSpec,
        finish: &serde_json::Value,
        dry_run: bool,
    ) -> Result<(), WorkflowError>;
}

pub struct NextInvocation {
    pub command: serde_json::Value,
    pub ui_root: PathBuf,
    pub wizard_dir: PathBuf,
    pub input: serde_json::Value,
}

pub fn resolve_next_wizard(
    finish: &serde_json::Value,
    allowlist: &Allowlist,
) -> Result<Option<NextInvocation>, WorkflowError>;

/// base ← input ← config_patch. Object keys deep-merge; arrays/scalars replace.
/// Non-object `input` or `config_patch` → WorkflowError::Merge.
pub fn merge_wizard_config(
    base: serde_json::Value,
    input: serde_json::Value,
    config_patch: Option<serde_json::Value>,
) -> Result<serde_json::Value, WorkflowError>;

#[derive(Debug)]
pub enum WorkflowError {
    PathDenied { path: PathBuf },
    Timeout { stderr_tail: String },
    NonZero { status: i32, stderr_tail: String },
    InvalidStdout { cause: String },
    ChainDepth { max: u32 },
    Resolve { path: String, cause: String },
    MissingPython3,
    Merge { cause: String },
}

// ErrorCode::WorkflowError { code: WORKFLOW_ERROR, slug: workflow, exit: 9 }
// crates/wyvern/src/error/emit.rs emits via that variant only and includes
// WorkflowError::subcode() in the stderr envelope.

| Variant | Cause | Recovery |
|---------|-------|----------|
| `PathDenied` | Path escaped allowlist | Use a path under `{wyvern_share}`, cwd, or current wizard.json directory |
| `Timeout` | Script exceeded 30s | Shorten script or raise only via a later ADR; `stderr_tail` is in the JSON `cause` |
| `NonZero` | Script exit ≠ 0 | Fix script; stderr_tail is in the JSON `cause` |
| `InvalidStdout` | Pre stdout not one JSON object with object `config_patch` | Print `{ "config_patch": { ... } }` only |
| `ChainDepth` | 17th hop | Keep chains ≤ 16 |
| `Resolve` | `{wyvern_share}` / relative path failed | Fix the path string |
| `MissingPython3` | `.py` script and `python3` not on PATH | Install Python 3 |
| `Merge` | `input` or `config_patch` not an object | Pass JSON objects |

pub struct CliArgs {
    pub host: HostOptions,
    pub positionals: Vec<String>,
    pub workflow_dry_run: bool, // --workflow-dry-run; never a HostOptions field
}

// argv construction only — spawn/timeout/stderr-tail MUST call
// crates/wyvern/src/extensions/preexec.rs helpers (extract/re-export if needed).
// Forbid a second Command::new stack in workflow/.
fn script_argv(canonical: &Path) -> Result<Vec<OsString>, WorkflowError> {
    if canonical.extension() == Some(OsStr::new("py")) {
        Ok(vec![OsString::from("python3"), canonical.as_os_str().to_os_string()])
    } else {
        Ok(vec![canonical.as_os_str().to_os_string()])
    }
}

// crates/wyvern-host/src/routes/wizard.rs
// After wyvern-wizard stack validation, copy next_wizard from the HTTP
// request onto WizardResult. Do not pass next_wizard into WizardSession.
fn finish_to_result(req: WizardFinishRequest, validated: WizardResult) -> WizardResult {
    WizardResult { next_wizard: req.next_wizard, ..validated }
}

// crates/wyvern/src/pipeline.rs
// run_from_loaded: Command::Wizard => run_wizard_workflow_loop(...); _ => existing path
pub fn run_wizard_workflow_loop(
    first: serde_json::Value,
    host: HostOptions,
    runner: &WorkflowRunner,
    dry_run: bool,
) -> Result<String, PipelineError>;
```

`WizardFinishRequest.button` stays `WizardTerminalButton` (`finish` \| `cancel` \| `dismissed`).

Welcome hub `wizard.json`:

```json
{
  "type": "wizard",
  "page": {
    "id": "home",
    "title": "Wyvern Guide",
    "html": "pages/home.html"
  },
  "config": {
    "topics": [
      { "id": "overview", "label": "Overview", "html": "pages/overview.html" },
      { "id": "questions", "label": "AskUserQuestion", "html": "pages/questions.html" },
      { "id": "templates", "label": "Template wizard", "html": "pages/templates.html" },
      { "id": "agent-dag", "label": "Agent DAG", "html": "pages/agent-dag.html" }
    ],
    "theme": "light"
  },
  "width": 800,
  "height": 560
}
```

`guide` extension:

```json
{
  "id": "guide",
  "description": "Visual feature guide (multi-page wizard).",
  "examples": ["wyvern guide"],
  "match": { "argv_prefix": ["guide"] },
  "expand": {
    "command_from_file": "{wyvern_share}/welcome/wizard.json",
    "host": { "ui_root": "{wyvern_share}/welcome" }
  }
}
```

Chain-test fixture finish (testdata only — not welcome pages):

```json
{
  "button": "finish",
  "data": {},
  "stack": [],
  "next_wizard": {
    "path": "{wyvern_share}/testdata/workflow/b/wizard.json",
    "input": { "from": "a" }
  }
}
```

## Acceptance criteria

1. `wyvern guide` expands the `guide` argv-prefix extension and opens the welcome hub with four topic cards from `config.topics` (REQ-0127). `wyvern help` / `--help` stay stdout.
2. Fixture `workflow.pre` runs after validate and before host bind; `config_patch` is on the first `GET /api/wizard/state` (REQ-0124).
3. Fixture A `button: "finish"` with `next_wizard` runs fixture B in the same process; stdout is B's finish JSON with `next_wizard` omitted (REQ-0126).
4. Paths that escape `{wyvern_share}`, cwd, or the current wizard.json directory fail with `WORKFLOW_ERROR` exit 9; pre failure does not start the host.
5. `button: "cancel"` and `button: "dismissed"` skip post and skip chain (REQ-0125, REQ-0126).
6. A 17th hop fails with `WorkflowError::ChainDepth` (`NEXT_WIZARD_MAX_DEPTH = 16`).
7. `--workflow-dry-run` is parsed on `CliArgs` and appends `--dry-run` to pre and post argv.
8. Host copies `next_wizard` on finish after stack validation and does not spawn workflow scripts (ADR-0023, ADR-0024). `cli.toml` owns spawn/chain; `host.toml` forbids spawn.
9. Schema accepts wizard JSON that includes `workflow` (`WIZARD_FIELDS`); a missing `workflow` key remains valid.
10. `run_from_loaded` sends every `Command::Wizard` through `run_wizard_workflow_loop`. `WorkflowRunner` spawn goes through `extensions/preexec` helpers only.

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-schema
cargo test -p wyvern-cli --test workflow_pre_post
cargo test -p wyvern-cli --test workflow_chain
cargo test -p wyvern-cli --test guide_extension
cargo test -p wyvern-host --test wizard_next_wizard_passthrough
rg -n "workflow_script_spawn|wizard_chain_loop" boundaries/wyvern/cli.toml boundaries/wyvern-host/host.toml
# Requires wyvern-schema and wyvern-host already published to crates.io
# (QA-002). Local path/workspace deps do not satisfy crates.io verify.
cargo publish --dry-run -p wyvern-cli --locked
```

## Non-closure

- g.5 / g.6 / g.7 example wizards (hook installer, template picker, Agent DAG demo) — welcome bridge pages already link via `wizardNextWizard`
- L2 Playwright tour
- `--emit-all`
- `wyvern chain` subcommand
- Phase E `--interactive` / MCP auto-chain
- Dialog type gallery (walkthrough R1)
- `probe-destination.py` and example workflow scripts
- `cargo publish --dry-run -p wyvern-cli` until `wyvern-schema` and `wyvern-host` are published to crates.io. Packaging of rust-embed assets is in-crate (`embedded/`); the remaining dry-run failure is the unpublished sibling crates, not a g.4 code defect (QA-002).

## Authority

- REQ-0124, REQ-0125, REQ-0126, REQ-0127
- ADR-0023, ADR-0024, ADR-0006
- [wizard-workflow-architecture.md](wizard-workflow-architecture.md)
- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
