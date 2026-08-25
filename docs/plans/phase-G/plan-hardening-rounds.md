# Phase G Plan Hardening — Round Table

## Wave 1 (g.1–g.3)

| Round | Step | Reviewer | Model | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
|-------|------|----------|-------|-----------------|--------|----------|-----------|-------|---------------|------------|------|
| STEP2-R1 | 2 | plan-scope-reviewer | sonnet | 3b61187 | FAIL | 0 | 2 | 3 | psr-r1-3b61187 | | compose preexec ownership, Timeout variant |
| STEP3-R1 | 4 | critical-plan-reviewer | cursor-grok-4.6-high-fast | 3b61187 | FAIL | 1 | 6 | 3 | cpr-G-3b61187-R1 | | see 25e8c05 fix commit |
| STEP2-R2 | 2 | plan-scope-reviewer | sonnet | 25e8c05 | FAIL | 0 | 2 | 3 | psr-G-25e8c05-R2 | STEP2-R1 | cargo test cmd, build_skill_records sig |
| STEP3-R2 | 4 | critical-plan-reviewer | cursor-grok-4.6-high-fast | 25e8c05 | FAIL | 0 | 2 | 4 | cpr-G-25e8c05-R2 | STEP3-R1 | USAGE_ERROR, near-miss kinds |
| — | fix | team-lead | — | be7bdfb | — | — | — | — | — | STEP2-R2,STEP3-R2 | scope+critical R2 fixes applied |
| — | fix | cursor-quality-mgr | — | 15751aa | — | — | — | — | — | plan QA R1 | cross-doc gaps ATM-QA-001–006 |
| STEP6-R2 | 6 | req-qa + arch-qa | — | 15751aa | PASS | 0 | 0 | 0 | qa-pass-15751aa | | 15/15 deliverables; PR #82 CI green |
| — | docs | team-lead | — | pending | — | — | — | — | — | | REQ-0134–0137 + ADR-0022 Phase G principal updates |

Cap: `plan_scope_review_cycle_limit: 2`, `critical_review_cycle_limit: 2` — **Wave 1 plan QA PASS at `15751aa`**

---

## Wave 2 (g.4–g.7)

| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
|-------|------|----------|-----------------|--------|----------|-----------|-------|---------------|------------|------|
| STEP1-R1 | 1 | guidelines pass | e172aaa | PASS | 0 | 0 | 0 | guidelines-e172aaa | | REQ-0124–0127, ADR-0023–0024 |
| SCOPE-R1 | 2 | plan-scope-reviewer | e172aaa | FAIL | 0 | 6 | 2 | SCOPE-R1-e172aaa | | hook contract, validation gates |
| CRIT-R1 | 4 | critical-plan-reviewer | e172aaa | FAIL | 2 | 4 | 2 | e172aaa-CRIT-R1 | | host passthrough, WIZARD_FIELDS |
| SCOPE-R2 | 2 | plan-scope-reviewer | e172aaa | FAIL | 0 | 1 | 1 | SCOPE-R2-e172aaa | SCOPE-R1 | cargo test syntax |
| CRIT-R2 | 4 | critical-plan-reviewer | e172aaa | FAIL | 0 | 6 | 2 | e172aaa-CRIT-R2 | CRIT-R1 | ErrorCode, invoke, sidecar |
| QA-R1 | 6 | req-qa + arch-qa | e172aaa | FAIL | 1+1 | 2+2 | 0 | qa-r1-http | | http-post-schema vs next_wizard |
| QA-R2 | 6 | req-qa + arch-qa | e172aaa | PASS | 0 | 0 | 0 | qa-r2-pass | QA-R1 | HTTP contracts + host REQs |
| SCOPE-VERIFY | 2 | plan-scope-reviewer | e172aaa | PASS | 0 | 0 | 1 | scope-g4-g7-verify | SCOPE-R2 | after QA-2 |
| CRIT-VERIFY | 4 | critical-plan-reviewer | e172aaa | FAIL | 0 | 1 | 1 | CRIT-VERIFY-013 | CRIT-R2 | WYVERN_BIN — fixed in docs |

Cap: 3 reviewer cycles. Post-review fixes: PLAN-CRIT-013 (`WYVERN_BIN`), PLAN-SCOPE-009 (g.7 AC 2 wire-shape). Gate: [`.plan-hardening/gate-summary.md`](.plan-hardening/gate-summary.md).
