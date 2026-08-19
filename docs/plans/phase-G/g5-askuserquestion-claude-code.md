---
id: g.5
title: AskUserQuestion replacement + Claude Code hook workflow
status: complete
branch: feature/phase-G-g5-askuserquestion
worktree: ../wyvern-worktrees/feature/phase-G-g5-askuserquestion
target: integrate/phase-G
---

# Sprint g.5 — AskUserQuestion → Wyvern + hook chain

## Goal

Example (a): Claude Code AskUserQuestion hook installer using the g.4 `WorkflowRunner`. Pre queries disk (REQ-0124). The wizard toggles Global / Repo. Post applies finish JSON (REQ-0125). The welcome Questions page finishes with `next_wizard` into this example (REQ-0126). Page JS does not touch the filesystem.

## Hard dependencies

- g.4 merged (`WorkflowRunner`, chain loop, `--workflow-dry-run`, host `next_wizard` copy)
- Phase B `question` type (direct-invoke docs only)

## Deliverables

| Path | Purpose |
|------|---------|
| `scripts/ext/query-askuserquestion-hook.py` | Pre: stdout `config_patch.hook_state` |
| `scripts/ext/apply-askuserquestion-hook.py` | Post (`--dry-run`); `--remove` (test-only); `--invoke` (installed hook command) |
| `crates/wyvern/tests/workflow_askuserquestion_invoke.rs` | `--invoke`: PreToolUse stdin → `question` envelope → answers stdout |
| `crates/wyvern/tests/workflow_welcome_chain_questions.rs` | Welcome Questions finish JSON → CLI resolves next hop |
| `share/wyvern/examples/askuserquestion-hook/wizard.json` | Wizard + `workflow.pre` / `workflow.post` |
| `share/wyvern/examples/askuserquestion-hook/pages/*.html` | Global / Repo toggles + review |
| `share/wyvern/examples/askuserquestion-hook/app.js` | Collect `data.hook_config` |
| `share/wyvern/welcome/pages/questions.html` | Docs + required `next_wizard` into this example |
| `crates/wyvern/tests/workflow_askuserquestion_hook.rs` | Temp-dir hooks: pre merge → mock finish → post |

### Contracts

```json
{
  "type": "wizard",
  "page": { "id": "toggles", "title": "AskUserQuestion hook", "html": "pages/toggles.html" },
  "workflow": {
    "pre": "{wyvern_share}/scripts/ext/query-askuserquestion-hook.py",
    "post": "{wyvern_share}/scripts/ext/apply-askuserquestion-hook.py"
  }
}
```

Pre stdout:

```json
{
  "config_patch": {
    "hook_state": {
      "global": { "enabled": true, "installed": true },
      "repo": { "enabled": false, "installed": false }
    }
  }
}
```

Finish `data.hook_config`:

```json
{
  "global": { "enabled": true },
  "repo": { "enabled": false }
}
```

Welcome Questions finish (required):

```json
{
  "button": "finish",
  "data": {},
  "stack": [],
  "next_wizard": {
    "path": "{wyvern_share}/examples/askuserquestion-hook/wizard.json",
    "input": { "from": "welcome" },
    "ui_root": "{wyvern_share}/examples/askuserquestion-hook"
  }
}
```

### On-disk hook contract (AC 4)

| Scope | Path | Env |
|-------|------|-----|
| Global | `$HOME/.claude/settings.json` | `HOME` (required for global) |
| Repo | `$WYVERN_REPO_ROOT/.claude/settings.local.json` | `WYVERN_REPO_ROOT` from g.4 spawn env (cwd if unset) |

When a scope is enabled and the settings file or parent `.claude/` directory is missing, post **creates** them. Disabled scopes do not create files.

Event / matcher: `PreToolUse` / `AskUserQuestion`.

One complete Wyvern-managed hook entry (merged into `hooks.PreToolUse[]`; do not overwrite unrelated entries):

```json
{
  "matcher": "AskUserQuestion",
  "hooks": [
    {
      "type": "command",
      "command": "python3 /absolute/canonical/path/to/apply-askuserquestion-hook.py --invoke",
      "managed_by": "wyvern:askuserquestion-hook",
      "version": 1
    }
  ]
}
```

Where the file allows comments, write `# wyvern:askuserquestion-hook v1` immediately above that entry.

`--invoke` (installed command; **in scope** for g.5):

1. Post **bakes** `python3 <canonical-absolute-apply-script> --invoke` (no `${WYVERN_SHARE}`).
2. Reads `WYVERN_BIN` from the g.4 spawn env (canonical wyvern executable). Bakes that **absolute path** into the hook `command` as the first token: `"<WYVERN_BIN-or-error>"` is not used as argv0 of the hook; the hook remains `python3 <absolute-apply-script> --invoke` and `--invoke` execs `WYVERN_BIN`. If `WYVERN_BIN` is unset, resolve `wyvern` via PATH (`shutil.which`) and bake that absolute path into a sidecar next to the settings file or into the script's `--invoke` default. **Never** bake `sys.executable` (that is Python). Windows: same absolute `python3`/`py` + script path tokens (no Unix-only `env VAR=` prefix).
3. stdin is Claude Code PreToolUse JSON:

```json
{
  "hook_event_name": "PreToolUse",
  "tool_name": "AskUserQuestion",
  "tool_input": {
    "questions": [
      { "question": "Pick one", "header": "Pick", "options": [{ "label": "A" }, { "label": "B" }], "multiSelect": false }
    ]
  }
}
```

4. Map `tool_input.questions` to `{ "type": "question", "questions": [...] }` and run `$WYVERN_BIN` (else `wyvern` on PATH).
5. stdout is REQ-0067 (`questions`, `answers`, `response`). Exit 0 on success.

This is **not** the wizard finish path. Injecting that stdout into Claude Code `hookSpecificOutput` / native tool-result across CC versions is **non-closure**.

`--remove` is **script/test-only** (not a wizard finish field). It strips entries with `managed_by: wyvern:askuserquestion-hook` from the two files above.

Both-disabled finish (`global.enabled` and `repo.enabled` both false) is **valid**. Post disables those scopes by removing Wyvern-managed entries for each disabled scope. There is no uninstall button.

## Acceptance criteria

1. Pre runs via `WorkflowRunner` before host bind; toggles initialize from `config.hook_state` (REQ-0124).
2. Wizard has Global and Repo enable/disable rows. Both-disabled finish is valid and means disable both scopes (see on-disk contract).
3. Post runs on `button: "finish"`; `--workflow-dry-run` reaches the script as `--dry-run` and writes no hook files (REQ-0125).
4. Written entries match the on-disk contract (absolute `--invoke` path, markers); `--remove` (script/test-only) strips only marked entries.
5. `workflow_askuserquestion_hook` covers pre → merged state → mock finish → post against a temp hook dir (`HOME` + `WYVERN_REPO_ROOT` isolated).
6. `workflow_askuserquestion_invoke` feeds sample PreToolUse stdin and asserts the question-envelope → answers stdout contract.
7. Welcome Questions page emits the `next_wizard` object above; `workflow_welcome_chain_questions` asserts the CLI resolves that hop (REQ-0126).
8. This sprint does not change `wyvern-host` beyond what g.4 already landed (`next_wizard` copy only). No host script execution.

## Required validation

```bash
cargo test -p wyvern-cli --test workflow_askuserquestion_hook
cargo test -p wyvern-cli --test workflow_askuserquestion_invoke
cargo test -p wyvern-cli --test workflow_welcome_chain_questions
```

```bash
# manual (not a QA gate):
# wyvern share/wyvern/examples/askuserquestion-hook/wizard.json \
#   --ui-root share/wyvern/examples/askuserquestion-hook --viewer none
```

## Non-closure

- Claude Code hook schema auto-discovery across versions
- Mapping `--invoke` stdout into Claude Code `hookSpecificOutput` / native tool-result (version-specific)
- `--interactive` / MCP stdin pipe of wizard → script (Phase E)
- MCP `question` tool
- Repo skill `.claude/skills/wyvern-question/` (walkthrough R6)

## Authority

- REQ-0124, REQ-0125, REQ-0126
- ADR-0023, ADR-0024
- [claude-code-hook-workflow.md](claude-code-hook-workflow.md)
- [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md)
