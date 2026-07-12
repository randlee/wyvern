---
name: cursor-orchestration
description: >-
  Orchestrate Wyvern sprint/phase work inside a single Cursor session.
  Parent coordinates; Task(rust-developer) implements; Task(cursor-quality-mgr)
  owns the QA gate and spawns shared reviewers. Use when the user asks for
  /cursor-orchestration, Cursor-session phase/sprint orchestration, or
  same-session QA via cursor-quality-mgr. Never use ATM quality-mgr or
  codex-orchestration while this skill governs the session.
disable-model-invocation: true
---

# Cursor Orchestration

Same-session Cursor adaptation of the repo's codex-orchestration gate contract.
Does **not** modify `.claude/skills/codex-orchestration/` or
`.claude/agents/quality-mgr.md`.

## Hard role map (non-negotiable)

| Role | Agent | Task `subagent_type` |
|------|-------|----------------------|
| Orchestrator | this parent session | — |
| Developer | `rust-developer` | `rust-developer` |
| Quality manager | **`cursor-quality-mgr` only** | **`cursor-quality-mgr`** |
| Reviewers | shared `.claude` agents | `req-qa`, `arch-qa`, `rust-qa-agent`, `rust-best-practices-agent`, `rust-service-hardening-agent`, `flaky-test-qa` |

### Quality-mgr binding (critical)

While this skill governs the session:

1. The quality-mgr **role** is always fulfilled by **`cursor-quality-mgr`**.
2. **Never** spawn Task `subagent_type: quality-mgr` (the Claude/ATM agent).
3. If any template, sibling skill, triage note, or prompt says `quality-mgr` /
   `assignee="quality-mgr"`, **rewrite** to `cursor-quality-mgr` and launch
   once. Do not dual-dispatch.
4. Per QA round: **at most one** QA coordinator Task, and it must be
   `cursor-quality-mgr`.
5. Do not follow `codex-orchestration`, ATM team-lead QA handoffs, or any path
   that assigns ATM `quality-mgr` in parallel with this skill.
6. Parent does **not** launch reviewers directly in the same round as
   `cursor-quality-mgr` (coordinator owns reviewer spawn).

## Parent constraints

- Coordinator only: no product code, no cargo/clippy/test QA analysis.
- Persist state to disk (sprint docs, PR #, SHA, triage records, QA verdicts).
- Prefer short Task completion summaries over pasting full agent transcripts.
- Worktrees via `/sc-git-worktree` (never switch main repo off `develop`).

## Default model matrix

Override only when the user names a model for a role.

| Role | Default |
|------|---------|
| Parent orchestrator | current session model |
| `rust-developer` | user-planned / `claude-4.6-sonnet-medium-thinking` if unspecified |
| `cursor-quality-mgr` | `claude-4.6-sonnet-medium-thinking` |
| Reviewers | leave agent default unless user overrides |

## Preconditions

1. Sprint/phase target defined in `docs/requirements.md`,
   `docs/architecture.md`, and `docs/plans/project-plan.md` (or linked phase plan).
2. Sprint worktree exists under `../wyvern-worktrees/<branch>` (create via
   `/sc-git-worktree` if missing).
3. These exist and are readable:
   - `.cursor/agents/cursor-quality-mgr.md`
   - `.claude/agents/{req-qa,arch-qa,flaky-test-qa,rust-qa-agent,rust-best-practices-agent,rust-service-hardening-agent,rust-developer}.md`
   - `.claude/skills/quality-management-gh/SKILL.md`
   - `.claude/assets/sc-rust/quality-mgr/quality-mgr.rust.md`
4. `sc-compose` available for template rendering.

## Sprint flow

1. Render a dev assignment from
   `.cursor/skills/cursor-orchestration/dev-template.xml.j2`.
   Always include `sprint_doc` as authoritative scope.
2. Spawn `Task` `rust-developer` with the rendered assignment and planned model.
3. On push report (branch + SHA): open or update the PR targeting
   `integrate/phase-N` (or the sprint's `pr_target`).
4. Render QA assignment from
   `.cursor/skills/cursor-orchestration/qa-template.xml.j2`
   with `assignee` / coordinator = **`cursor-quality-mgr`**.
5. Spawn **one** `Task` with `subagent_type: cursor-quality-mgr`.
6. `cursor-quality-mgr` launches the reviewer set (see that agent prompt).
7. QA-2+: omit RBP and service-hardening reviewers; merge gate remains
   0B+0I+0m with no backlog deferral.
8. On FAIL: run `/triaging-findings`, then fix via
   `fix-assignment.xml.j2` → `rust-developer`, then re-QA via
   `cursor-quality-mgr` only.
9. On PASS + green CI: merge may proceed.

## Plan review flow

1. Complete `/plan-hardening` steps 1–5 when applicable.
2. Assign plan QA to **`cursor-quality-mgr`** with `review_mode: plan` and the
   phase-plan path as `sprint_doc`.
3. Never also assign ATM `quality-mgr` for the same plan review.

## QA coverage rule

Same gate as codex-orchestration:

- Extract every deliverable, AC, deletion target, validation item, and artifact
  from `sprint_doc` before reviewers run.
- PASS requires 100% deliverable completion.
- `req-qa` owns completion percentage; `arch-qa` owns structural gates.

## CI

```bash
gh pr checks <PR> --watch
gh pr view <PR> --json mergeStateStatus,reviewDecision
```

Do not use ATM-only `atm gh` monitors unless they are confirmed available;
prefer `gh`.

## Templates (this skill)

- `dev-template.xml.j2`
- `fix-assignment.xml.j2`
- `qa-template.xml.j2`
- `review-template.xml.j2`
- `req-qa-assignment.json.j2`
- `arch-qa-assignment.json.j2`
- `flaky-test-qa-assignment.json.j2`
- `sprint-plan.md.j2`

Rust reviewer JSON templates remain shared at:

- `.claude/assets/sc-rust/quality-mgr/templates/`

## Isolation from Codex path

| Do | Do not |
|----|--------|
| Edit only under `.cursor/` for this skill | Edit `codex-orchestration` or `quality-mgr.md` |
| Spawn `cursor-quality-mgr` | Spawn `quality-mgr` while this skill is active |
| Reuse shared reviewers by Task type | Duplicate reviewer prompts under `.cursor/` |
| Remap stale `quality-mgr` mentions | Dual-launch both coordinators |
