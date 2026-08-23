---
id: g.13
title: Wizard type refs (template, hook, welcome-bridge) + sc-compose snippets
status: complete (integrate)
branch: feature/phase-G-g13-wizard-refs
worktree: ../wyvern-worktrees/feature/phase-G-g13-wizard-refs
target: integrate/phase-G
---

# Sprint g.13 — Type refs and sc-compose snippets

## Goal

Ship Layer 3 type recipes for **vanilla-chrome** wizard kinds (template picker,
AskUserQuestion hook, welcome hub bridge) plus copy-safe **sc-compose J2**
snippets authors can use to render wizard HTML. Docs/skill only — no
`crates/**` changes.

## Hard dependencies

- g.10 SKILL.md router on `integrate/phase-G`
- g.11 `wyvern-wizard-js` + vanilla-chrome templates
- g.12 `wyvern-dag-wizard-js` + `dag-wizards.md`
- g.9 dataflow lint (optional declare on new wizards)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g13-wizard-type-refs-and-templates.md` | This sprint doc |
| `.claude/skills/creating-wyvern-wizard/references/wizard-types/template.md` | Template picker type recipe |
| `.claude/skills/creating-wyvern-wizard/references/wizard-types/hook.md` | Hook / settings toggle type recipe |
| `.claude/skills/creating-wyvern-wizard/references/wizard-types/welcome-bridge.md` | Hub card → `wizardNextWizard` bridge |
| `.claude/skills/creating-wyvern-wizard/templates/sc-compose/page.j2` | Minimal J2 snippet for rendered pages |
| `.claude/skills/creating-wyvern-wizard/templates/sc-compose/vars.json` | Sample vars for the J2 snippet |
| `.claude/skills/creating-wyvern-wizard/templates/sc-compose/README.md` | How to run `sc-compose` + `wyvern compose render` |

Update `dag-wizards.md` cross-links to the new type doc basenames (no
`template-wizards` alias in SKILL paths).

## Acceptance criteria

1. `template.md`, `hook.md`, and `welcome-bridge.md` exist under
   `references/wizard-types/` and name stack `vanilla-chrome`, agent
   `wyvern-wizard-js`, golden examples, and lint/dataflow notes.
2. Each type doc lists when **not** to use it (cross-type table like
   `dag-wizards.md`).
3. `templates/sc-compose/` ships J2 + vars + README with the exact
   `wyvern compose render` smoke command from Phase F.
4. Layer 0 `SKILL.md` wizard type picker paths resolve (no broken links).
5. No `crates/**` delta vs merge base.

## Required validation

```bash
test -f docs/plans/phase-G/g13-wizard-type-refs-and-templates.md
test -f .claude/skills/creating-wyvern-wizard/references/wizard-types/template.md
test -f .claude/skills/creating-wyvern-wizard/references/wizard-types/hook.md
test -f .claude/skills/creating-wyvern-wizard/references/wizard-types/welcome-bridge.md
test -f .claude/skills/creating-wyvern-wizard/templates/sc-compose/page.j2
test -f .claude/skills/creating-wyvern-wizard/templates/sc-compose/vars.json
test -f .claude/skills/creating-wyvern-wizard/templates/sc-compose/README.md

rg -n "vanilla-chrome" .claude/skills/creating-wyvern-wizard/references/wizard-types/template.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/hook.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/welcome-bridge.md
rg -n "wyvern-wizard-js" .claude/skills/creating-wyvern-wizard/references/wizard-types/template.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/hook.md \
  .claude/skills/creating-wyvern-wizard/references/wizard-types/welcome-bridge.md
rg -n "share/wyvern/examples/template-picker" .claude/skills/creating-wyvern-wizard/references/wizard-types/template.md
rg -n "share/wyvern/examples/askuserquestion-hook" .claude/skills/creating-wyvern-wizard/references/wizard-types/hook.md
rg -n "share/wyvern/welcome" .claude/skills/creating-wyvern-wizard/references/wizard-types/welcome-bridge.md
rg -n "wizardNextWizard" .claude/skills/creating-wyvern-wizard/references/wizard-types/welcome-bridge.md
rg -n "sc-compose|compose render" .claude/skills/creating-wyvern-wizard/templates/sc-compose/README.md

git diff --name-only origin/integrate/phase-G...HEAD | python3 -c "
import sys
p = [l.strip() for l in sys.stdin if l.startswith('crates/')]
assert not p, 'crates/ must not change: ' + ', '.join(p)
"
git diff --check
```

## Non-closure

- CI lint gate — **g.14**
- Declaring `config.dataflow` on all golden examples
- Live sc-compose in CI (manual smoke when `sc-compose` on PATH)

## Authority

- [g8-wizard-authoring-foundation.md](g8-wizard-authoring-foundation.md)
- [g11-wyvern-wizard-js-agent.md](g11-wyvern-wizard-js-agent.md)
- [wave-3-wizard-authoring/README.md](wave-3-wizard-authoring/README.md)
- [f3-compose-extension.md](../phase-F/f3-compose-extension.md)
