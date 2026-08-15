# Phase F Plan Hardening — Round Table

| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
|-------|------|----------|-----------------|--------|----------|-----------|-------|---------------|------------|------|
| STEP2-R3-VERIFY | 2 | plan-scope-reviewer | b9cfca8 | PASS | 0 | 0 | 1 | scope-verify-b9cfca8 | STEP2-R3 | verification after final fix |
| STEP4-R2 | 4 | critical-plan-reviewer | 64dd9f2 | FAIL | 0 | 8 | 3 | c2-64dd9f2-015-022 | STEP4-R1 | cycle 2 |
| STEP4-R3 | 4 | critical-plan-reviewer | 8354dc2 | PASS | 0 | 0 | 3 | c3-8354dc23-PASS | STEP4-R2 | ready for QA |

Cap behavior reminder:

- default reviewer cap is `3` for both background reviewers unless the vars JSON overrides it explicitly
- every reviewer `FAIL` must still be routed to `arch-ctm`
- if the final allowed reviewer cycle still returns `FAIL`, route that finding set to `arch-ctm`, complete the correction pass, then stop and report `cap-exhausted / not converged`
- do not offer `accept and proceed`
- do not ask the user for a branching decision mid-loop
