# Phase F Plan Hardening — Round Table

| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
|-------|------|----------|-----------------|--------|----------|-----------|-------|---------------|------------|------|
| STEP2-R1 | 2 | plan-scope-reviewer | 266c92a | FAIL | 1 | 8 | 3 | scope-r1-f1b0i3-f2b0i0-f3b0i2-f4b1i2 |  | initial scope review |

Cap behavior reminder:

- default reviewer cap is `3` for both background reviewers unless the vars JSON overrides it explicitly
- every reviewer `FAIL` must still be routed to `arch-ctm`
- if the final allowed reviewer cycle still returns `FAIL`, route that finding set to `arch-ctm`, complete the correction pass, then stop and report `cap-exhausted / not converged`
- do not offer `accept and proceed`
- do not ask the user for a branching decision mid-loop
