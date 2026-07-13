# Phase D / d.6 — Wizard polish and edge cases

## Status
pending

## Acceptance Criteria

- First page: back button hidden or disabled
- Last page: next button label changes to "Finish"
- Empty `data` on a page handled gracefully (no undefined errors)
- Wizard with a single page (N=1) works correctly
- OS close on any wizard page returns `{"button":"dismissed","stack":[...]}`

## Notes
