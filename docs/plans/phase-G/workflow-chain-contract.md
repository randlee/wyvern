# Wizard → workflow chain contract (Phase G Wave 2)

Short overview. **Sprint docs are sole authority** for deliverables, acceptance criteria, and validation. Behavior: [wizard-workflow-architecture.md](wizard-workflow-architecture.md). Types: [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md).

```
wyvern guide  →  topic page finish with next_wizard (g.5 / g.6 / g.7)
                    ↓
              workflow PRE   →  config_patch merge
                    ↓
              wizard session (HTML/JS, no disk I/O)
                    ↓
              finish JSON (button=finish)
                    ↓
              workflow POST  →  apply / copy / export
                    ↓
              next_wizard    →  CLI loop (max 16)
```

`next_wizard` as a **field** is optional on any given finish JSON (REQ-0126). g.4 **bridge** welcome pages (AskUserQuestion, Template wizard, Agent DAG) emit `wizardNextWizard` on finish to hop to the g.5 / g.6 / g.7 example wizards; g.5–g.7 own those example wizards. Welcome topic pages in g.5–g.7 **must** emit `next_wizard`.

Markers, script flags, and inventories live only in the owning sprint doc.
