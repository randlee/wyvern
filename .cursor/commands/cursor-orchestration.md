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
3b. **Pre-QA-1 RBP sweep** (codex parity): `rust-developer` runs
   `rust-best-practices-agent` on planned QA-1 `review_targets`; fixes all RBP
   findings; reports before first QA spawn.
4. Render + assign QA to **`cursor-quality-mgr` only**:

```bash
sc-compose render \
  --root .cursor/skills/cursor-orchestration \
  --file qa-template.xml.j2 \
  --var-file "$_VARS"
```

5. **Dev–QA loop** (repeat until both gates pass):
   - **QA gate:** PASS only with `reviewer_spawn_gate=pass`, fenced JSON from
     every required reviewer, **0 Blocking + 0 Important + 0 Minor** open
     findings, and 100% deliverable completion.
   - **PR gate:** every QA round posts **all** findings to the PR via
     `findings-report.md.j2` or `quality-report.md.j2` (see skill).
   - **CI gate:** all required PR checks green.
   - **Evidence gate:** parent correlates every `task_id` in PR Machine Status
     to completed Task subagents; `pr_comment_url` present; FAIL rounds cite
     triage `.ttl` paths; orchestration state `qa_rounds[]` updated.
   - On FAIL: `/triaging-findings` → fix **all** finding ids + `.ttl` paths →
     push → re-QA. Do not merge without triage evidence on the fix assignment.
6. On **PASS + green CI + evidence chain complete**: merge; start next sprint.

```bash
gh pr checks <PR> --watch
gh pr view <PR> --json mergeStateStatus,reviewDecision
```

If the user names a sprint/phase/doc, use that as `sprint_doc` authority
(typically under `docs/plans/`).
