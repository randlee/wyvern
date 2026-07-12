# /cursor-orchestration

Run Wyvern sprint/phase orchestration **inside this Cursor session**.

## Mandatory

1. Read and follow `.cursor/skills/cursor-orchestration/SKILL.md`.
2. Quality-mgr role → **only** Task `subagent_type: cursor-quality-mgr`.
3. **Never** spawn `quality-mgr` (ATM/Claude agent) while this command is active.
4. If any prompt or skill says `quality-mgr`, remap to `cursor-quality-mgr` and
   launch once — do not dual-dispatch.
5. Developer work → Task `rust-developer` with the planned model.
6. Do not edit `.claude/skills/codex-orchestration/` or
   `.claude/agents/quality-mgr.md` as part of this flow.
7. Keep all authored paths repo-root-relative (see skill **Path portability**).
   Do not paste host-absolute paths into skill/agent/template text.

## Quick flow

1. Ensure sprint worktree via `/sc-git-worktree` from `develop`.
2. Assign dev with `.cursor/skills/cursor-orchestration/dev-template.xml.j2`.
3. On push (branch + SHA): open/update PR.
4. Assign QA with `.cursor/skills/cursor-orchestration/qa-template.xml.j2` to
   **`cursor-quality-mgr` only**.
5. On FAIL: triage → fix Task → re-QA via `cursor-quality-mgr`.
6. On PASS + green CI: merge to sprint `pr_target`.

If the user names a sprint/phase/doc, use that as `sprint_doc` authority.
