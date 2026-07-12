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

| Role | Agent | Invocation |
|------|-------|------------|
| Orchestrator | this parent session | — |
| Developer | `rust-developer` | Task `subagent_type: rust-developer` |
| Quality manager | **`cursor-quality-mgr` only** | see **Spawning cursor-quality-mgr** |
| Reviewers | shared `.claude` agents | Task `subagent_type:` `req-qa`, `arch-qa`, `rust-qa-agent`, `rust-best-practices-agent`, `rust-service-hardening-agent`, `flaky-test-qa` |

### Quality-mgr binding (critical)

While this skill governs the session:

1. The quality-mgr **role** is always fulfilled by **`cursor-quality-mgr`**.
2. **Never** spawn Task `subagent_type: quality-mgr` (the Claude/ATM agent).
3. If any template, sibling skill, triage note, or prompt says `quality-mgr` /
   `assignee="quality-mgr"`, **rewrite** to `cursor-quality-mgr` and launch
   once. Do not dual-dispatch.
4. Per QA round: **at most one** QA coordinator Task, and it must be
   `cursor-quality-mgr` (or the fallback that loads that prompt only).
5. Do not follow `codex-orchestration`, ATM team-lead QA handoffs, or any path
   that assigns ATM `quality-mgr` in parallel with this skill.
6. Parent does **not** launch reviewers directly in the same round as
   `cursor-quality-mgr` (coordinator owns reviewer spawn).

## Parent constraints

- Coordinator only: no product code, no cargo/clippy/test QA analysis.
- Persist state to disk (sprint docs, PR #, SHA, triage records, QA verdicts).
- Prefer short Task completion summaries over pasting full agent transcripts.
- Worktrees via `/sc-git-worktree` (never switch main repo off `develop`).
- Use ambient `git` / `gh` only — no account flags, no `gh auth login`, no
  `--hostname` / `--user` overrides unless the user explicitly requires them.

## Path portability (required)

All authored paths in this skill, its templates, the `/cursor-orchestration`
command, and `.cursor/agents/cursor-quality-mgr.md` must be portable:

- Use **repo-root-relative** paths (e.g. `.cursor/skills/...`,
  `.claude/agents/...`, `docs/plans/...`).
- Skill-local template names may be basename-only when the skill directory is
  already implied (e.g. `dev-template.xml.j2`).
- **Never** hardcode host-absolute paths (`/Users/...`, `/Volumes/...`,
  `/home/...`, `C:\...`, `D:\...`, `\\server\...`).
- Do not embed machine-specific roots in examples or defaults.
- Worktree layout is a **sibling of the primary repo root** named
  `wyvern-worktrees/<branch>`. Always create/resolve via `/sc-git-worktree`
  (or git common-dir → primary root → sibling `wyvern-worktrees`). **Never**
  assume `../wyvern-worktrees` is valid from the current cwd (it fails when
  already inside a worktree).
- Runtime assignment fields (`worktree_path`, etc.) may be absolute **only**
  when resolved dynamically for the current machine; authored skill/prompt
  content must still use relative forms and placeholders like
  `{{ worktree_path }}`.

## Spawning cursor-quality-mgr

Try in order; stop at the first success. Never fall through to `quality-mgr`.

1. **Preferred:** Cursor Task with `subagent_type: cursor-quality-mgr` and the
   rendered QA XML as the Task prompt (plus planned model).
2. **If the Task enum rejects `cursor-quality-mgr`:** spawn a Task whose prompt
   begins with: read and adopt `.cursor/agents/cursor-quality-mgr.md`, then
   execute the rendered QA XML assignment. Do **not** use
   `subagent_type: quality-mgr`.
3. **Custom subagent:** if the product exposes project agents by name, invoke
   the `cursor-quality-mgr` agent with the same QA XML payload.

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
2. Sprint worktree exists (create via `/sc-git-worktree` from `develop` if missing).
3. These exist and are readable (repo-root-relative):
   - `.cursor/agents/cursor-quality-mgr.md`
   - `.claude/agents/{req-qa,arch-qa,flaky-test-qa,rust-qa-agent,rust-best-practices-agent,rust-service-hardening-agent,rust-developer}.md`
   - `.claude/skills/quality-management-gh/SKILL.md`
   - `.claude/assets/sc-rust/quality-mgr/quality-mgr.rust.md`
4. `sc-compose` is on `PATH` and usable (see **Tool recipes**).

## Sprint flow

1. Render a dev assignment with `sc-compose` from
   `.cursor/skills/cursor-orchestration/dev-template.xml.j2`.
   Always include `sprint_doc` as authoritative scope.
2. Spawn Task `rust-developer` with the rendered assignment and planned model.
3. On push report (branch + SHA): open or update the PR targeting
   `integrate/phase-N` (or the sprint's `pr_target`).
4. Render QA assignment with `sc-compose` from
   `.cursor/skills/cursor-orchestration/qa-template.xml.j2`
   with coordinator = **`cursor-quality-mgr`**.
5. Spawn **one** `cursor-quality-mgr` coordinator (see spawn rules above).
6. `cursor-quality-mgr` launches the reviewer set (see that agent prompt).
7. QA-2+: omit RBP and service-hardening reviewers; merge gate remains
   0B+0I+0m with no backlog deferral.
8. On FAIL: run `/triaging-findings`, then fix via
   `.cursor/skills/cursor-orchestration/fix-assignment.xml.j2` →
   `rust-developer`, then re-QA via `cursor-quality-mgr` only.
   Fix assignments must include:
   - authoritative `sprint_doc` = owning/promoted branch sprint plan (plus
     additional sprint docs when findings span multiple sprint origins)
   - specific requirement ids and ADR ids the fix must address (not whole
     requirements/architecture dumps; planning already embeds those in the
     sprint plan)
   - triage `.ttl` paths and concrete occurrences
   Fresh `rust-developer` Tasks have no prior sprint memory — never omit
   these fields.
9. On PASS + green CI: merge may proceed.

## Plan review flow

1. Complete `/plan-hardening` steps 1–5 when applicable.
2. Assign plan QA to **`cursor-quality-mgr`** with `review_mode: plan` and the
   phase-plan path as `sprint_doc`.
3. Never also assign ATM `quality-mgr` for the same plan review.

## QA coverage rule

- Extract every deliverable, AC, deletion target, validation item, and artifact
  from `sprint_doc` before reviewers run.
- PASS requires 100% deliverable completion.
- `req-qa` owns completion percentage; `arch-qa` owns structural gates.

## Tool recipes (fenced)

All CLI examples use ambient auth. Do not add account, hostname, or login flags.

Skill template root (repo-root-relative):

`.cursor/skills/cursor-orchestration`

### Preconditions check

```bash
command -v sc-compose
command -v gh
command -v git
```

### Render dev assignment

```bash
_VARS=$(mktemp)
cat > "$_VARS" <<'JSON'
{
  "task_id": "dev-1",
  "sprint": "1a",
  "sprint_doc": "docs/plans/<sprint>.md",
  "description": "<summary>",
  "worktree_path": "<resolved-worktree-path>",
  "branch": "<branch>",
  "pr_target": "integrate/phase-N",
  "deliverables": "- <item>",
  "acceptance_criteria": "- <item>",
  "references": "- docs/requirements.md"
}
JSON
sc-compose render \
  --root .cursor/skills/cursor-orchestration \
  --file dev-template.xml.j2 \
  --var-file "$_VARS"
rm -f "$_VARS"
```

### Render QA assignment

```bash
_VARS=$(mktemp)
cat > "$_VARS" <<'JSON'
{
  "task_id": "qa-1",
  "sprint": "1a",
  "sprint_doc": "docs/plans/<sprint>.md",
  "review_mode": "sprint_review",
  "description": "<summary>",
  "pr_number": "<pr>",
  "branch": "<branch>",
  "worktree_path": "<resolved-worktree-path>",
  "commits": "<sha>",
  "review_targets": "- <path>",
  "references": "- docs/requirements.md",
  "changed_files": "",
  "triage_records": ""
}
JSON
sc-compose render \
  --root .cursor/skills/cursor-orchestration \
  --file qa-template.xml.j2 \
  --var-file "$_VARS"
rm -f "$_VARS"
```

### Render reviewer JSON (req-qa example)

```bash
_VARS=$(mktemp)
cat > "$_VARS" <<'JSON'
{
  "reference_docs": ["docs/requirements.md", "docs/architecture.md"],
  "sprint_doc": "docs/plans/<sprint>.md",
  "phase": "1",
  "sprint": "1a",
  "worktree_path": "<resolved-worktree-path>",
  "branch": "<branch>",
  "commit": "<sha>",
  "review_targets": ["<path>"],
  "round_limit": false,
  "changed_files": [],
  "carry_forward_findings_json": "[]",
  "triage_records": [],
  "notes": ""
}
JSON
sc-compose render \
  --root .cursor/skills/cursor-orchestration \
  --file req-qa-assignment.json.j2 \
  --var-file "$_VARS"
rm -f "$_VARS"
```

Render `arch-qa-assignment.json.j2` and `flaky-test-qa-assignment.json.j2` the
same way (see each template's `required_variables`). Rust reviewer templates:

```bash
sc-compose render \
  --root .claude/assets/sc-rust/quality-mgr/templates \
  --file rust-qa-assignment.json.j2 \
  --var-file "$_VARS"
```

(Use the var shapes in `.claude/assets/sc-rust/quality-mgr/quality-mgr.rust.md`.)

### CI status

```bash
gh pr checks <PR> --watch
gh pr view <PR> --json mergeStateStatus,reviewDecision
```

### PR findings / closeout (shared templates)

Required variable names come from the template frontmatter. Flat string map for
`sc-compose`. Blocking review example:

```bash
_VARS=$(mktemp)
# Fill every required_variables entry from
# .claude/skills/quality-management-gh/findings-report.md.j2
sc-compose render \
  --root .claude/skills/quality-management-gh \
  --file findings-report.md.j2 \
  --var-file "$_VARS" \
  | gh pr review <PR> --request-changes --body-file -
rm -f "$_VARS"
```

In-flight comment:

```bash
sc-compose render \
  --root .claude/skills/quality-management-gh \
  --file findings-report.md.j2 \
  --var-file "$_VARS" \
  | gh pr comment <PR> --body-file -
```

PASS closeout:

```bash
sc-compose render \
  --root .claude/skills/quality-management-gh \
  --file quality-report.md.j2 \
  --var-file "$_VARS" \
  | gh pr review <PR> --approve --body-file -
```

(Or `gh pr comment` if approval is not appropriate.)

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
