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
   **Never** spawn a second QA Task for the same `task_id` / `qa_pass` while the
   first is still running.
5. Do not follow `codex-orchestration`, ATM team-lead QA handoffs, or any path
   that assigns ATM `quality-mgr` in parallel with this skill.
6. Parent does **not** launch reviewers directly **unless** QA-SPAWN-001 applies
   (nested coordinator cannot spawn Tasks — see below). Default: coordinator
   owns reviewer spawn.
7. Parent does **not** merge on narrative QA PASS alone — see **Reviewer spawn
   merge gate** below.

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

## QA-SPAWN-001 — nested coordinator cannot spawn reviewers

When `cursor-quality-mgr` runs as a **nested Task subagent**, it may lack Task
spawn capability (observed on c.11). Do **not** let the coordinator substitute
foreground cargo/grep review. Use **parent delegation**:

1. `cursor-quality-mgr` (or parent before spawn) renders reviewer assignments via
   the fenced `sc-compose` recipes.
2. **Parent orchestrator** spawns every required reviewer Task in parallel with
   models from `.cursor/orchestration-agent-models.yaml`.
3. Parent records each reviewer `agent` + `task_id` in the manifest.
4. Parent awaits completion and forwards each reviewer's fenced JSON to
   `cursor-quality-mgr` for aggregation, TODO scan, CI check, and PR publish —
   **or** parent performs steps 4–14 of `cursor-quality-mgr.md` inline while
   adopting that agent prompt (coordinator role unchanged).
5. Machine Status JSON must include `"spawn_actor": "parent-orchestrator"` on
   each `spawned_reviewers[]` entry when the parent spawned that reviewer.
6. Parent merge gate still correlates every `task_id` to completed Tasks in
   **this** session — spawn actor does not relax evidence requirements.

**Preferred when possible:** run QA coordination from the **top-level**
orchestrator session (not nested) so `cursor-quality-mgr` can spawn reviewers
directly and omit `spawn_actor: parent-orchestrator`.


Try in order; stop at the first success. Never fall through to `quality-mgr`.

1. **Preferred:** Cursor Task with `subagent_type: cursor-quality-mgr` and the
   rendered QA XML as the Task prompt (plus planned model).
2. **If the Task enum rejects `cursor-quality-mgr`:** spawn a Task whose prompt
   begins with: read and adopt `.cursor/agents/cursor-quality-mgr.md`, then
   execute the rendered QA XML assignment. Do **not** use
   `subagent_type: quality-mgr`.
3. **Custom subagent:** if the product exposes project agents by name, invoke
   the `cursor-quality-mgr` agent with the same QA XML payload.

## Agent model defaults

Read `.cursor/orchestration-agent-models.yaml` before launching any subagent.
Override only when the user names a model for a role. Always pass `model:`
explicitly on Task spawns (do not rely on agent frontmatter defaults).

- Parent orchestrator: current session model (not in the table).
- `inherit` entries: use the agent frontmatter default.
- `alternates` on an agent: honor only when the user explicitly prefers that model.

### Phase-ending review model override

For phase-ending QA (`review_mode` / assignment indicating phase-end), use the
YAML defaults **except**:

- Prefer **`gpt-5.6-terra-medium`** (GPT-5.6 Terra) for **`rust-qa-agent`**
  when available in the Cursor Task model list; otherwise fall back to the YAML
  default for `rust-qa-agent`.
- Do not replace `arch-qa`'s Sonnet assignment; keep the Claude precision gate
  and the Terra comprehensive reviewer as a deliberate mix.

Other phase-end reviewers stay on their YAML defaults.

## Preconditions

1. Sprint/phase target defined in `docs/requirements.md`,
   `docs/architecture.md`, and `docs/plans/project-plan.md` (or linked phase plan).
2. Sprint worktree exists (create via `/sc-git-worktree` from `develop` if missing).
3. These exist and are readable (repo-root-relative):
   - `.cursor/orchestration-agent-models.yaml`
   - `.cursor/agents/cursor-quality-mgr.md`
   - `.claude/agents/{req-qa,arch-qa,flaky-test-qa,rust-qa-agent,rust-best-practices-agent,rust-service-hardening-agent,rust-developer}.md`
   - `.claude/skills/quality-management-gh/SKILL.md`
   - `.claude/assets/sc-rust/quality-mgr/quality-mgr.rust.md`
4. `sc-compose` is on `PATH` and usable (see **Tool recipes**).

## Dev–QA loop (mandatory per sprint)

Every sprint runs the same closed loop until **both** gates pass:

```
dev → push → PR → QA → [FAIL → triage → fix → push → re-QA]* → PASS + green CI → merge
```

| Gate | Requirement |
|------|-------------|
| **QA** | `cursor-quality-mgr` declares **PASS** only when deliverables are 100% complete, **`reviewer_spawn_gate: pass`**, every required reviewer returned **fenced JSON**, every required reviewer PASSes, **TODO scan clean or TODO findings included in open counts**, and **zero open findings** at any severity (**0 Blocking + 0 Important + 0 Minor**) from **reviewer JSON ∪ TODO scan**. |
| **CI** | All required PR checks green (`gh pr checks <PR> --watch`). Merge is blocked while any check fails or is pending. |

### Reviewer spawn merge gate (parent — non-negotiable)

Do **not** merge a sprint PR unless **all** of the following are true:

1. Latest PR QA comment Machine Status JSON includes `"reviewer_spawn_gate": "pass"`.
2. `reviewer_manifest` lists every required reviewer with non-empty `task_id`
   correlatable to **completed Task subagents in this parent session**.
3. `evidence_chain_json` (or equivalent fields) includes `pr_comment_url` for
   this `qa_pass` and triage `.ttl` paths when the round was FAIL→fix.
4. Parent QA verdict duplicates the PR Machine Status JSON (dual publish).
5. `.cursor/<phase>-orchestration.json` has a `qa_rounds[]` entry for this pass.
6. Finding counts match **reviewer fenced JSON ∪ TODO-scan** union.

If the PR report lacks `reviewer_spawn_gate` / `reviewer_manifest`, treat QA as
**INCOMPLETE** — re-run `cursor-quality-mgr` for that `qa_pass`; do not merge.

If two conflicting QA comments exist on the same PR (e.g. PASS then FAIL),
**FAIL wins**; require fix + QA-2 before merge.

**Fix scope on FAIL:** route **every** finding id (Blocking, Important, and Minor) back to `rust-developer` via `fix-assignment.xml.j2`. Do not merge after fixing only Important/Blocking findings while Minors remain open.

**QA rounds:** QA-1 runs the full reviewer set (incl. RBP + service-hardening). QA-2+ omits RBP and service-hardening but the merge gate stays **0B+0I+0m** — prior-round findings must be fixed before re-QA.

**Sequence:** one sprint at a time. Do not start sprint N+1 until sprint N is merged to `integrate/phase-N` (or the sprint's `pr_target`).

## Chain of evidence (non-negotiable)

Merge is blocked unless an auditable evidence chain exists for the **latest**
`qa_pass` on that PR. Self-attested prose without correlatable artifacts is
invalid (c.9-class failure).

### Evidence layers

| Layer | What proves it | Required artifact |
|-------|----------------|-------------------|
| **Reviewer spawn** | Sub-agents actually ran | `reviewer_manifest[].task_id` per required reviewer — ids from Cursor Task tool returns, not invented |
| **Reviewer output** | Findings came from reviewers | Parseable fenced JSON per agent; aggregated counts match **reviewer JSON ∪ TODO scan** |
| **Triage** | Findings were correlated before fix | `.triage/<phase_id>/findings/<finding_id>.ttl` paths; `qa-triage` Task ids when triage ran |
| **Fix** | Dev addressed triaged scope | Fix assignment lists every finding id + triage `.ttl` paths; push SHA after fix |
| **PR publish** | Stakeholders can audit QA | `gh pr comment` or review URL for **each** QA round; `detailed_findings_md` lists **all** severities |
| **Coordinator handoff** | Parent/ATM received same facts | Parent verdict + persisted orchestration state contain **identical** Machine Status JSON as PR post |

### Parent correlation (fail-closed)

Before merge, the parent orchestrator must:

1. Read the latest PR QA comment Machine Status JSON for this `qa_pass`.
2. Confirm every `reviewer_manifest[].task_id` matches a **completed** Task
   subagent from this session (notification id or transcript correlation).
3. If any required `task_id` is missing, empty, or uncorrelatable → **QA
   INCOMPLETE** — do not merge; re-run `cursor-quality-mgr`.
4. On FAIL rounds, confirm `/triaging-findings` produced `.ttl` records before
   fix dispatch; fix assignment must cite those paths.
5. Append a `qa_rounds[]` entry to `.cursor/<phase>-orchestration.json` (or
   sprint-local orchestration state) with: `qa_pass`, `commit`, `verdict`,
   `pr_comment_url`, `reviewer_manifest`, `triage_ttl_paths`, `finding_ids`,
   `coordinator_task_id`.

### Dual publish (PR + coordinator)

`cursor-quality-mgr` must publish the **same** rendered report body to:

1. **PR** — `gh pr comment` or `gh pr review` (every QA round)
2. **Parent** — final verdict message includes the same Machine Status JSON block

Codex `quality-mgr` equivalent: PR + ATM message per `quality-management-gh`.

### `evidence_chain_json` (mandatory on Cursor QA PR posts)

`cursor-quality-mgr` must populate on every Cursor QA report (FAIL, IN-FLIGHT,
PASS). Missing or empty `evidence_chain_json` / `reviewer_spawn_gate` /
`reviewer_manifest_json` → spawn-gate **fail** / QA **INCOMPLETE**.

```json
{
  "qa_pass": "qa-c10-1",
  "commit": "<sha>",
  "pr_number": 39,
  "pr_comment_url": "<url from gh after post>",
  "coordinator_task_id": "<cursor-quality-mgr Task id>",
  "reviewer_tasks": [
    {"agent": "req-qa", "task_id": "<id>", "fenced_json_received": true, "verdict": "PASS"}
  ],
  "triage": {
    "phase_id": "phase-C-web-server",
    "ttl_paths": [".triage/phase-C-web-server/findings/REQ-QA-001.ttl"],
    "qa_triage_task_ids": ["<id>"]
  },
  "parent_correlation_required": true
}
```

## Required message sequence

Every orchestration handoff follows codex ACK → Work → Completion → receiver ACK:

| Hop | Parties | Required messages |
|-----|---------|-------------------|
| Dev | parent → `rust-developer` | assignment → ACK → push+SHA → validation PASS/FAIL |
| Pre-QA RBP | parent → `rust-developer` | RBP sweep assignment → report with finding ids fixed or none |
| QA | parent → `cursor-quality-mgr` | QA XML → ACK → (optional IN-FLIGHT) → verdict + PR URL |
| Triage | parent → `qa-triage` (per finding) | triage JSON → fenced JSON → `.ttl` path |
| Fix | parent → `rust-developer` | fix XML with finding ids + `.ttl` paths → push+SHA |

Parent must not spawn the next hop until the prior completion message exists.
Silent skips invalidate the evidence chain.

## Sprint flow

1. Render a dev assignment with `sc-compose` from
   `.cursor/skills/cursor-orchestration/dev-template.xml.j2`.
   Always include `sprint_doc` as authoritative scope.
2. Spawn Task `rust-developer` with the rendered assignment and planned model.
3. On push report (branch + SHA): open or update the PR targeting
   `integrate/phase-N` (or the sprint's `pr_target`).
4. **Before QA-1** (codex parity): `rust-developer` runs a self-directed
   `rust-best-practices-agent` sweep on the same `review_targets` planned for
   QA-1 and fixes all RBP findings before the first QA assignment. This is dev
   cleanup, not a substitute for QA-1 RBP review.
5. Render QA assignment with `sc-compose` from
   `.cursor/skills/cursor-orchestration/qa-template.xml.j2`
   with coordinator = **`cursor-quality-mgr`**.
6. Spawn **one** `cursor-quality-mgr` coordinator (see spawn rules above).
7. If nested spawn fails (QA-SPAWN-001), parent spawns reviewers per
   **QA-SPAWN-001**; coordinator still owns aggregation and PR publish.
   Otherwise `cursor-quality-mgr` launches the reviewer set (see that agent
   prompt).
8. On **FAIL** (any open finding or deliverable &lt; 100%): run
   `/triaging-findings`, then fix **all** findings via
   `.cursor/skills/cursor-orchestration/fix-assignment.xml.j2` →
   `rust-developer`, push, then re-QA via `cursor-quality-mgr` only.
   Repeat until QA PASS **and** CI green.
   Fix assignments must include:
   - authoritative `sprint_doc` = owning/promoted branch sprint plan (plus
     additional sprint docs when findings span multiple sprint origins)
   - specific requirement ids and ADR ids the fix must address (not whole
     requirements/architecture dumps; planning already embeds those in the
     sprint plan)
   - triage `.ttl` paths and concrete occurrences
   - **every** finding id from the FAIL round (all severities)
   Fresh `rust-developer` Tasks have no prior sprint memory — never omit
   these fields.
9. On **PASS + green CI + reviewer_spawn_gate pass**: merge to `pr_target`; then start the next sprint.

## Parity with codex-orchestration

Cursor orchestration is a **transport adapter** (parent + Task vs team-lead +
ATM). It must preserve codex gate semantics:

| Codex | Cursor equivalent |
|-------|-------------------|
| `team-lead` | parent session |
| `cwy` | `rust-developer` |
| `quality-mgr` | **`cursor-quality-mgr` only** (never ATM `quality-mgr`) |
| `codex-orchestration/*.j2` | `.cursor/skills/cursor-orchestration/*.j2` |
| Shared reviewers | same `.claude/agents/*` Task types |
| `quality-management-gh` reports | same templates + SKILL |
| Pre-QA-1 RBP dev sweep | `rust-developer` before first QA |
| Reviewer spawn in background | `cursor-quality-mgr` Task spawns only |
| Multi-pass QA on PR | every round posts all findings to PR |
| 0B+0I+0m merge gate | unchanged |

**Cursor-only additions** (do not weaken codex):

- `reviewer_spawn_gate` + `reviewer_manifest_json` in PR Machine Status when
  Cursor QA runs (proves reviewers spawned and returned fenced JSON)
- Parent merge blocked without manifest proof (prevents coordinator self-review)
- Single QA coordinator per round (no duplicate `qa-c9-1` Tasks)

**Do not edit** `.claude/skills/codex-orchestration/` or
`.claude/agents/quality-mgr.md` from this skill. Shared report templates may
gain **optional** Cursor fields; codex callers omit them.

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

### Render fix assignment

```bash
_VARS=$(mktemp)
cat > "$_VARS" <<'JSON'
{
  "task_id": "fix-1",
  "phase": "1",
  "sprint_doc": "docs/plans/<sprint>.md",
  "branch": "<branch>",
  "worktree_path": "<resolved-worktree-path>",
  "pr_target": "integrate/phase-N",
  "description": "<summary>",
  "finding_ids": "- <id>",
  "triage_records": "- .triage/<phase>/<finding>.ttl",
  "required_fixes": "- <fix>",
  "acceptance_criteria": "- <criterion>",
  "references": "- docs/requirements.md",
  "requirement_ids": "- REQ-0001",
  "adr_ids": "- ADR-0001"
}
JSON
sc-compose render \
  --root .cursor/skills/cursor-orchestration \
  --file fix-assignment.xml.j2 \
  --var-file "$_VARS"
rm -f "$_VARS"
```

### Render reviewer JSON (req-qa example)

```bash
_VARS=$(mktemp)
cat > "$_VARS" <<'JSON'
{
  "reference_docs": ["docs/requirements.md", "docs/architecture.md", "docs/plans/project-plan.md"],
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
