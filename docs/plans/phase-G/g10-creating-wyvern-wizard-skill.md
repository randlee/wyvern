---
id: g.10
title: creating-wyvern-wizard Layer 0 skill router
status: complete (integrate)
branch: feature/phase-G-g10-wizard-skill
worktree: ../wyvern-worktrees/feature/phase-G-g10-wizard-skill
target: integrate/phase-G
---

# Sprint g.10 — creating-wyvern-wizard skill (Layer 0)

## Goal

Ship the **Layer 0 router** for Wave 3 wizard authoring: a thin
`creating-wyvern-wizard` SKILL.md (progressive disclosure, gates G1–G6, stack
picker, wizard type picker) plus the remaining core refs
(`platform-contract.md`, `validation-and-lint.md`). Page-author work delegates
to `wyvern-wizard-js` and `wyvern-dag-wizard-js` **only**.

This sprint does **not** use `rust-developer`. No `crates/**` changes. No
agent bodies, no templates, no lint-rule implementation.

## Hard dependencies

- g.8 foundation on this branch (stack registry, `author-workflow.md`,
  `dataflow-contracts.md`, `references/stacks/*`)
- Wave 2 examples on `integrate/phase-G` (goldens cited, not rewritten)
- `wyvern wizard lint` nav rules WIZARD-LINT-001–004 exist on
  `feature/phase-G-wizard-lint` (g.9 extends them; this sprint does not merge
  that branch)

## Exact targets

- `docs/plans/phase-G/g10-creating-wyvern-wizard-skill.md`
- `.claude/skills/creating-wyvern-wizard/SKILL.md`
- `.claude/skills/creating-wyvern-wizard/references/core/platform-contract.md`
- `.claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md`
- `.cursor/skills/creating-wyvern-wizard/SKILL.md`

## Deliverables

Every listed deliverable lands at a production-ready level for this sprint.
No deliverable may be silently dropped or partially deferred.

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g10-creating-wyvern-wizard-skill.md` | Sole authority for g.10 AC and validation |
| `.claude/skills/creating-wyvern-wizard/SKILL.md` | Layer 0 router (~150 lines max) |
| `.claude/skills/creating-wyvern-wizard/references/core/platform-contract.md` | Host / CLI / page-JS seams + schema command |
| `.claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md` | `wyvern` schema validate + `wyvern wizard lint` |
| `.cursor/skills/creating-wyvern-wizard/SKILL.md` | Cursor stub pointing at the `.claude` copy |

Inherited from g.8 (not re-authored here): `references/core/author-workflow.md`,
`references/core/dataflow-contracts.md`, `references/stacks/*`.

**Do not** spawn or assign `rust-developer`. **Do not** edit `crates/**`.

### Contracts

#### Layer 0 router (SKILL.md)

SKILL.md is a table of contents (skills/agents guidelines v0.7). It must
contain, and must not expand into agent or template bodies:

1. **Progressive disclosure load map** — Layers 0–4 (this file → `references/core/`
   → one stack doc → one type doc → templates).
2. **Gates G1–G6** — fail-fast table pointing at `author-workflow.md`.
3. **Stack picker** — `vanilla-chrome` default, `workspace-canvas` supported;
   each row names its stack doc and agent.
4. **Wizard type picker** — template / hook / welcome-bridge / dag; type docs
   may be absent until g.12–g.13.
5. **Agent delegation** — `wyvern-wizard-js` and `wyvern-dag-wizard-js` only.

```
Layer 0  SKILL.md
Layer 1  references/core/{author-workflow,platform-contract,validation-and-lint,dataflow-contracts}.md
Layer 2  references/stacks/<one-stack>.md
Layer 3  references/wizard-types/<one-type>.md
Layer 4  templates/<stack>/
```

#### Schema validate (G1) — exact command

No second schema. Same loader the CLI uses. Exit **4** = schema / `validation`.

```bash
wyvern path/to/wizard.json --viewer none
```

#### Wizard lint (G4) — exact command

```bash
wyvern wizard lint path/to/wizard-dir
```

Exit 0 = clean, 1 = findings, 2 = usage. Nav codes 001–004 are implemented on
the lint branch; dataflow 005–008 wait for g.9.

#### Agent pairing (page authors only)

| Stack | Agent |
|-------|-------|
| `vanilla-chrome` | `wyvern-wizard-js` |
| `workspace-canvas` | `wyvern-dag-wizard-js` |

## Required work

- Author SKILL.md as a Layer 0 router (≤150 lines) with the five contract
  sections above. Repo-relative paths only.
- Document host / CLI / page-JS ownership and the G1 command in
  `platform-contract.md`.
- Document schema validate + `wyvern wizard lint` (codes, exits, profiles) in
  `validation-and-lint.md`.
- Add a Cursor stub that points at the `.claude` SKILL.md (do not duplicate
  the router).
- Index g.10 on the Wave 3 map is already done in g.8
  (`wave-3-wizard-authoring/README.md`). Do not rewrite Wave 1–2 AC.

## This sprint does not close

- WIZARD-LINT-005–008 implementation — **g.9**
- `.claude/agents/wyvern-wizard-js.md` + `templates/vanilla-chrome/` — **g.11**
- `.claude/agents/wyvern-dag-wizard-js.md` + `wizard-types/dag-wizards.md` — **g.12**
- Type refs (template, hook, welcome-bridge) and sc-compose J2 — **g.13**
- CI lint gate and known nav-lint HTML fixes — **g.14**
- Registering the JS agents in `.claude/agents/registry.yaml` (g.11 / g.12)
- Rust / `crates/**` / `rust-developer` implementation
- New host routes, schema fields, or dialog types
- Replacing `share/wyvern/templates/wizard/two-step/`

## Acceptance criteria

1. `.claude/skills/creating-wyvern-wizard/SKILL.md` exists with YAML
   `name: creating-wyvern-wizard` and is **at most 150 lines**. It contains a
   progressive disclosure load map, gates G1–G6, a stack picker
   (`vanilla-chrome` default, `workspace-canvas`), a wizard type picker, and
   agent delegation to `wyvern-wizard-js` and `wyvern-dag-wizard-js` only.
2. SKILL.md does **not** name `rust-developer` as an allowed delegate (it may
   forbid it). No other rust-* authoring agent is listed.
3. `references/core/platform-contract.md` documents CLI vs host vs page JS,
   `wizard.json` required fields, and the G1 command
   `wyvern path/to/wizard.json --viewer none`.
4. `references/core/validation-and-lint.md` documents
   `wyvern wizard lint path/to/wizard-dir`, exit 0/1/2, nav codes 001–004, and
   dataflow codes 005–008 (implemented in g.9).
5. `.cursor/skills/creating-wyvern-wizard/SKILL.md` exists and points at
   `.claude/skills/creating-wyvern-wizard/SKILL.md` (stub, not a second router).
6. No file under `crates/` is added, modified, or deleted on this branch vs
   `origin/integrate/phase-G` except what g.8 already shipped (g.8 also has no
   `crates/` delta). This sprint adds no `crates/` paths.

## Required validation

```bash
test -f docs/plans/phase-G/g10-creating-wyvern-wizard-skill.md
test -f .claude/skills/creating-wyvern-wizard/SKILL.md
test -f .claude/skills/creating-wyvern-wizard/references/core/platform-contract.md
test -f .claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md
test -f .cursor/skills/creating-wyvern-wizard/SKILL.md
```

```bash
# Layer 0 router contracts
rg -q "Layer 0" .claude/skills/creating-wyvern-wizard/SKILL.md
rg -q "G1" .claude/skills/creating-wyvern-wizard/SKILL.md
rg -q "G6" .claude/skills/creating-wyvern-wizard/SKILL.md
rg -q "vanilla-chrome" .claude/skills/creating-wyvern-wizard/SKILL.md
rg -q "workspace-canvas" .claude/skills/creating-wyvern-wizard/SKILL.md
rg -q "wyvern-wizard-js" .claude/skills/creating-wyvern-wizard/SKILL.md
rg -q "wyvern-dag-wizard-js" .claude/skills/creating-wyvern-wizard/SKILL.md
rg -q "wizard-types" .claude/skills/creating-wyvern-wizard/SKILL.md
```

```bash
# Forbidden delegate
python3 - <<'PY'
from pathlib import Path
text = Path(".claude/skills/creating-wyvern-wizard/SKILL.md").read_text()
assert "rust-developer" in text, "must explicitly forbid rust-developer"
# Allowed mention is forbid-only: must not appear in an Agent | table cell as the delegate
for line in text.splitlines():
    if "rust-developer" in line and "|" in line and "Forbidden" not in line and "never" not in line.lower() and "not" not in line.lower() and "Do not" not in line:
        raise SystemExit(f"rust-developer looks like a delegate: {line}")
print("rust-developer forbid-only ok")
PY
```

```bash
# SKILL.md line cap
python3 - <<'PY'
from pathlib import Path
n = len(Path(".claude/skills/creating-wyvern-wizard/SKILL.md").read_text().splitlines())
assert n <= 150, f"SKILL.md is {n} lines (max 150)"
print("SKILL.md lines", n)
PY
```

```bash
rg -q "wyvern path/to/wizard.json --viewer none" \
  .claude/skills/creating-wyvern-wizard/references/core/platform-contract.md \
  .claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md
rg -q "wyvern wizard lint" \
  .claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md
rg -q "WIZARD-LINT-001" \
  .claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md
rg -q "WIZARD-LINT-008" \
  .claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md
rg -q ".claude/skills/creating-wyvern-wizard/SKILL.md" \
  .cursor/skills/creating-wyvern-wizard/SKILL.md
```

```bash
# Repo-relative paths only in authored g.10 files
python3 - <<'PY'
from pathlib import Path
needles = ("/" + "Users/", "/" + "Volumes/")
paths = [
    Path("docs/plans/phase-G/g10-creating-wyvern-wizard-skill.md"),
    Path(".claude/skills/creating-wyvern-wizard/SKILL.md"),
    Path(".claude/skills/creating-wyvern-wizard/references/core/platform-contract.md"),
    Path(".claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md"),
    Path(".cursor/skills/creating-wyvern-wizard/SKILL.md"),
]
bad = []
for p in paths:
    text = p.read_text()
    for i, line in enumerate(text.splitlines(), 1):
        if any(n in line for n in needles):
            bad.append(f"{p}:{i}:{line}")
assert not bad, "absolute paths:\n" + "\n".join(bad)
print("repo-relative paths ok")
PY
```

```bash
git diff --name-only origin/integrate/phase-G...HEAD | python3 -c "
import sys
p = [l.strip() for l in sys.stdin if l.startswith('crates/')]
assert not p, 'crates/ must not change: ' + ', '.join(p)
"
git diff --check
```

## Authority

- [g8-wizard-authoring-foundation.md](g8-wizard-authoring-foundation.md)
- [wave-3-wizard-authoring/README.md](wave-3-wizard-authoring/README.md)
- [wizard-workflow-architecture.md](wizard-workflow-architecture.md)
- REQ-0124, REQ-0125, REQ-0126; ADR-0006, ADR-0023, ADR-0024
- Skills/agents guidelines v0.7 (progressive disclosure; SKILL.md as TOC;
  fenced JSON from delegated agents)
