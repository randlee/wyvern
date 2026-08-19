# Claude Code AskUserQuestion hook — format notes (g.5)

Companion to [g5-askuserquestion-claude-code.md](g5-askuserquestion-claude-code.md). On-disk paths, complete hook entry JSON, AC, and validation live **only** in that sprint doc.

Page JS must not read or write hook files (REQ-0124, REQ-0125, ADR-0023).

Finish `data.hook_config` is the **desired** state. `config.hook_state` from pre is the **current** disk state.

`--remove` is script/test-only (not a wizard field). Both-disabled finish disables both scopes per the sprint-doc contract.

Welcome `next_wizard` JSON is specified in the sprint doc.
