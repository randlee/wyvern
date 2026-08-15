# Phase F Plan Hardening — Round Table

| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
|-------|------|----------|-----------------|--------|----------|-----------|-------|---------------|------------|------|
| STEP2-R3-VERIFY | 2 | plan-scope-reviewer | b9cfca8 | PASS | 0 | 0 | 1 | scope-verify-b9cfca8 | STEP2-R3 | verification after final fix |
| STEP4-R2 | 4 | critical-plan-reviewer | 64dd9f2 | FAIL | 0 | 8 | 3 | c2-64dd9f2-015-022 | STEP4-R1 | cycle 2 |
| STEP5-R1 | 5 | arch-ctm | e96c067 | PASS | 0 | 0 | 0 | step5-consistency |  | ADR-0022 cross-doc |
| STEP6-R1 | 6 | quality-mgr | e96c067 | PASS | 0 | 0 | 4 | qa-pass-0b0i4m |  | req-qa + arch-qa plan QA |

Cap behavior reminder:

- default reviewer cap is `3` for both background reviewers unless the vars JSON overrides it explicitly
- every reviewer `FAIL` must still be routed to `arch-ctm`
- if the final allowed reviewer cycle still returns `FAIL`, route that finding set to `arch-ctm`, complete the correction pass, then stop and report `cap-exhausted / not converged`
- do not offer `accept and proceed`
- do not ask the user for a branching decision mid-loop
