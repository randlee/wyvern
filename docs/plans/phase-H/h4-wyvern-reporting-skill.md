---
id: h.4
title: wyvern-reporting skill
status: complete
branch: feature/phase-H-h4-wyvern-reporting-skill
worktree: ../wyvern-worktrees/feature/phase-H-h4-wyvern-reporting-skill
target: integrate/phase-H
---

# Sprint h.4 — `wyvern-reporting` skill

## Goal

Ship a **reporting** skill (separate from `creating-wyvern-wizard`) documenting
how agents author sc-compose XHTML panels, build review manifests, and run
`report-xhtml` with `--review`.

## Hard dependencies

- h.3 merged (finish JSON contract stable)
- h.2 manifest schema on integrate

## Deliverables

| Path | Purpose |
|------|---------|
| `.claude/skills/wyvern-reporting/SKILL.md` | Layer 0 router |
| `.claude/skills/wyvern-reporting/references/core/panel-authoring.md` | sc-compose XHTML fragment rules |
| `.claude/skills/wyvern-reporting/references/core/review-manifest.md` | Manifest + CLI |
| `.claude/skills/wyvern-reporting/references/core/review-ux.md` | Actionable feedback, Approve/Cancel semantics |
| `.claude/skills/wyvern-reporting/references/core/installation.md` | `wyvern`, `sc-compose`, `python3` |
| `.claude/skills/wyvern-reporting/templates/panel.xhtml.j2` | Minimal fragment starter |
| `.claude/skills/wyvern-reporting/templates/review.json` | Manifest starter |
| `.cursor/skills/wyvern-reporting/SKILL.md` | Cursor stub → canonical skill |
| `docs/plans/phase-H/h4-wyvern-reporting-skill.md` | This sprint doc |

### Skill boundaries (normative)

| In scope | Out of scope |
|----------|--------------|
| XHTML panel fragments for report viewing | Wizard pages, `wizard-nav.js` |
| `wyvern panel.xhtml`, `wyvern report-xhtml` | `creating-wyvern-wizard` |
| `--review` agent loops parsing finish JSON | Published aggregate HTML reports |

### Agent workflow (document in SKILL.md)

1. Author/fix `.xhtml` panel(s) via sc-compose template.
2. Write `review.json` listing panels; set `role: "proposal"` on fix panel.
3. Run `wyvern report-xhtml --review review.json`.
4. Parse finish JSON; if `!approved`, revise panels and re-run.

## Acceptance criteria

1. SKILL.md progressive disclosure: panel → manifest → review → example paths.
2. References cite [xhtml-reporting-contract.md](xhtml-reporting-contract.md)
   finish shape verbatim.
3. No references to `rust-developer` or wizard lint (WIZARD-LINT-*).
4. Templates validate against h.2 manifest schema (documented check command).
5. `wyvern extensions list` examples cross-linked from `review-manifest.md`.

## Required validation

```bash
test -f .claude/skills/wyvern-reporting/SKILL.md
test -f .claude/skills/wyvern-reporting/references/core/panel-authoring.md
python3 scripts/ext/xhtml_report.py --validate-manifest .claude/skills/wyvern-reporting/templates/review.json
rg -n 'creating-wyvern-wizard' .claude/skills/wyvern-reporting/  # expect zero or explicit "use instead" negation
rg -n 'wizard-nav' .claude/skills/wyvern-reporting/  # expect zero
```

## Non-closure

- atm-core-specific template paths (wyvern docs stay repo-agnostic; may mention
  atm-core as example only)
- Skill catalog JSON schema changes — only if g.3 catalog requires new fields

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md)
- Synaptic-canvas skill guidelines v0.7 (authoring-platform QA profile)
