# Phase F Plan Hardening — Round Table

| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
|-------|------|----------|-----------------|--------|----------|-----------|-------|---------------|------------|------|

Cap behavior reminder:

- default reviewer cap is `3` for both background reviewers unless the vars JSON overrides it explicitly
- every reviewer `FAIL` must still be routed to `arch-ctm`
- if the final allowed reviewer cycle still returns `FAIL`, route that finding set to `arch-ctm`, complete the correction pass, then stop and report `cap-exhausted / not converged`
- do not offer `accept and proceed`
- do not ask the user for a branching decision mid-loop
