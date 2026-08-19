# Wyvern workflow hooks & wizard chaining (Phase G architecture)

Normative behavior for Wave 2. **g.4** lands the Rust types and runner; **g.5–g.7** consume them. All four sprints ship.

**Requirements:** REQ-0124, REQ-0125, REQ-0126, REQ-0127

**ADRs:** ADR-0023, ADR-0024

**Type and spawn signatures** live in [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md) (Boundary contracts). This file does not add deliverables, acceptance criteria, or validation commands.

**Repo paths:** source trees are `share/wyvern/**` and `scripts/ext/**` ([cli-extensions-contract.md](../phase-F/cli-extensions-contract.md)). Runtime prefix is `{wyvern_share}` (so `{wyvern_share}/scripts/ext/…`). Do not invent `crates/wyvern/share/`.

---

## 1. Workflow envelope

Optional `workflow: { "pre", "post" }` on wizard **command** JSON only (not on extension `expand`). CLI runs scripts; host ignores the field; page JS has no disk access.

| Phase | When | I/O |
|-------|------|-----|
| Pre (REQ-0124) | After validate, before host | stdout `{ "config_patch": { … } }`; deep-merge into `config` |
| Post (REQ-0125) | After `button: "finish"`, before `next_wizard` | finish JSON on stdin |

`cancel` / `dismissed` skip post. `--workflow-dry-run` appends `--dry-run` to argv. Timeout 30s. Allowlist: `{wyvern_share}`, cwd, current wizard.json directory. `.py` → `python3 <path>`. Failure → `WORKFLOW_ERROR` exit 9.

Merge order: `wizard.json` `config` ← `next_wizard.input` ← pre `config_patch`. Object keys deep-merge; arrays/scalars replace. Non-object `input` or `config_patch` is `WorkflowError::Merge`.

---

## 2. `next_wizard`

Optional sibling on finish JSON (REQ-0126 / ADR-0024). Host **copies** the field and does not resolve it. CLI honors it only when `button` is `finish`. Max 16 sessions. Final stdout omits `next_wizard`. `--emit-all` is out of Wave 2.

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

`path` required; `input` defaults `{}`; `ui_root` optional (else wizard-root inference). Relative `path` tries `{wyvern_share}`, then cwd, then current wizard.json directory.

---

## 3. Sprint consumption

g.4 ships the loop and fixture tests. g.5, g.6, and g.7 each wire their welcome topic page to `next_wizard` (required in those sprint docs). What ships per sprint is only in that sprint doc.

---

## Authority

- REQ-0124–REQ-0127, ADR-0023, ADR-0024, ADR-0006
- [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md)
- Phase F `preexec` spawn discipline
