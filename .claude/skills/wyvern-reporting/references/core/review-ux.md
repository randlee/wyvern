# Review UX and finish JSON

Load after [review-manifest.md](review-manifest.md). Contract:
[xhtml-reporting-contract.md](../../../../../docs/plans/phase-H/xhtml-reporting-contract.md)
§ Review finish JSON and § Frame profiles (`review`).

Review mode adds a footer: comments textarea, **Cancel**, **Approve**.
`report-review.js` POSTs once to `/api/report/finish` (not a wizard finish
helper). Buttons disable after the first click.

## Finish JSON (stdout) — cite verbatim

From the contract:

```json
{
  "button": "finish",
  "data": {
    "approved": true,
    "comments": "Panel 2 admissions/s still wrong",
    "panels": [
      { "path": "panels/fail-1.xhtml", "role": "failure", "label": "Fail 1" },
      { "path": "panels/proposed-fix.xhtml", "role": "proposal", "label": "Proposed fix" }
    ]
  }
}
```

| Field | Type | Meaning |
|-------|------|---------|
| `approved` | bool | `true` = Approve; `false` = Cancel |
| `comments` | string | Free text (may be empty) |
| `panels` | array | Echo of manifest panel entries — **host-validated** against `report-command.json` |

Cancel MUST set `approved: false`. Approve MUST set `approved: true`.

## Actionable comments

Ask the reviewer (or write as the agent presenting the report) for comments
that name the panel and the defect, as in the contract example
(`"Panel 2 admissions/s still wrong"`). Empty comments are valid; they are
not a substitute for `approved: false` when the proposal is rejected.

## Terminal actions (mutually exclusive)

| Event | Route | stdout |
|-------|-------|--------|
| Approve or Cancel | `POST /api/report/finish` | `{ "button": "finish", "data": { … } }` |
| OS-close / timeout | `POST /api/result` | `{ "button": "dismissed" }` — **no** `data` |

`dismissed` is not Cancel. Do not treat a dismissed session as
`approved: false`. Re-run `--review` if the window closed without a finish.

## Agent loop

```text
run wyvern report-xhtml --review review.json
read stdout JSON
if button == "dismissed":
    re-open or stop (no approval decision)
if button == "finish" and data.approved:
    stop — proposal accepted
if button == "finish" and not data.approved:
    apply data.comments to the named panels
    revise .xhtml (and proposal role panel)
    re-run --review
```

Parse `data.approved` as a boolean. Do not infer approval from comments
alone.

## Host finish errors (recovery)

| Code | Meaning | Fix |
|------|---------|-----|
| `REPORT_FINISH_UNKNOWN_FIELD` | Extra top-level keys | Send only `approved`, `comments`, `panels` |
| `REPORT_FINISH_PANELS_MISMATCH` | POST panels ≠ command manifest | Echo embedded `#manifest-data` panels |
| `REPORT_FINISH_PANELS_INVALID` | Bad `panels[]` shape | Objects with `path`, optional `label`/`role` |
| `REPORT_FINISH_MANIFEST_REQUIRED` | Finish without review token | Open with `--review` / `mode: "review"` |
| `REPORT_FINISH_COMMENTS_TOO_LONG` | `comments` > 32_768 | Shorten |
| `REPORT_FINISH_INVALID_JSON` | Malformed body | Resubmit valid JSON |
| `REPORT_FINISH_ALREADY_COMPLETE` | Duplicate finish (HTTP 409) | One terminal action per session |

View mode does not register `/api/report/finish` (HTTP 404). Use
`--review` when you need Approve/Cancel.

## Example paths

Interactive review fixtures live under
`crates/wyvern/tests/fixtures/xhtml-review/`. The packaged walkthrough is
`share/wyvern/examples/xhtml-review/` (h.5).
