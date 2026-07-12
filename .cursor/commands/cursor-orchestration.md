# /cursor-orchestration

Run Wyvern sprint/phase orchestration **inside this Cursor session**.

## Mandatory

1. Read and follow `.cursor/skills/cursor-orchestration/SKILL.md`.
2. Quality-mgr role → **only** `cursor-quality-mgr` (see skill spawn rules;
   never `quality-mgr`).
3. **Never** spawn `quality-mgr` (ATM/Claude agent) while this command is active.
4. If any prompt or skill says `quality-mgr`, remap to `cursor-quality-mgr` and
   launch once — do not dual-dispatch.
5. Developer work → Task `rust-developer` with model from
   `.cursor/orchestration-agent-models.yaml` (user override wins).
6. Do not edit `.claude/skills/codex-orchestration/` or
   `.claude/agents/quality-mgr.md` as part of this flow.
7. Keep all authored paths repo-root-relative (see skill **Path portability**).
8. Use ambient `git` / `gh` / `sc-compose` only — no account or login flags.
9. Render assignments with the fenced `sc-compose` recipes in the skill.
10. Pass reviewer models from `.cursor/orchestration-agent-models.yaml`; for
    phase-ending QA prefer `gpt-5.6-terra-medium` on `rust-qa-agent` when
    available (else YAML default).

## Quick flow

Build `$_VARS` JSON maps with the fenced `sc-compose` recipes in
`.cursor/skills/cursor-orchestration/SKILL.md` (dev, QA, fix, and reviewer
assignments). Read `.cursor/orchestration-agent-models.yaml` before each Task
spawn.

1. Ensure sprint worktree via `/sc-git-worktree` from `develop`.
2. Render + assign dev:

```bash
sc-compose render \
  --root .cursor/skills/cursor-orchestration \
  --file dev-template.xml.j2 \
  --var-file "$_VARS"
```

3. On push (branch + SHA): open/update PR with ambient `gh`.
4. Render + assign QA to **`cursor-quality-mgr` only**:

```bash
sc-compose render \
  --root .cursor/skills/cursor-orchestration \
  --file qa-template.xml.j2 \
  --var-file "$_VARS"
```

5. On FAIL: triage → fix Task (with owning-branch `sprint_doc`, REQ/ADR ids,
   triage `.ttl` paths) → re-QA via `cursor-quality-mgr`.
6. On PASS + green CI: merge to sprint `pr_target`.

```bash
gh pr checks <PR> --watch
gh pr view <PR> --json mergeStateStatus,reviewDecision
```

If the user names a sprint/phase/doc, use that as `sprint_doc` authority
(typically under `docs/plans/`).
