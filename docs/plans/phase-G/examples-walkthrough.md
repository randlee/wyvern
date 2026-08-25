# Phase G — Example walkthrough (sequential review)

Review **one sprint doc at a time**. This file is review order only — not a deliverables, AC, or validation list.

| Step | Sprint | Doc |
|------|--------|-----|
| 0 | g.4 welcome hub + workflow foundation | [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md) |
| 1 | g.5 AskUserQuestion hooks | [g5-askuserquestion-claude-code.md](g5-askuserquestion-claude-code.md) |
| 2 | g.6 template catalog | [g6-template-wizard.md](g6-template-wizard.md) |
| 3 | g.7 DAG demo + export | [g7-dag-agent-execution.md](g7-dag-agent-execution.md) |

All four sprints must complete, in that order. Architecture: [wizard-workflow-architecture.md](wizard-workflow-architecture.md).

Wave 2 adds CLI workflow Rust (REQ-0124–0127). The Phase D HTML-only rule still applies to **page `data`** (no Rust DAG interpreter). It does not forbid the g.4 runner.

## Other recommendations (not Wave 2 closure)

| Id | Recommendation | Notes |
|----|----------------|-------|
| R1 | Dialog gallery on welcome hub | Post-wave; not a g.4 gate |
| R2 | Bundle more `examples/` under share | Post-wave |
| R4 | Interactive/MCP preview page | After Phase E |
| R5 | L2 Playwright smoke | Post-wave; not attached to g.4–g.7 |
| R6 | `.claude/skills/wyvern-question/` | Post-wave |
| R7 | CSV/markdown extension showcase | Phase F already shipped the extensions |
