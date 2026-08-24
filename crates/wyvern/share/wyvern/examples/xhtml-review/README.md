# Synthetic XHTML review example

Fabricated atm-core-style `benchmark-run` panels for `wyvern report-xhtml`.
Numbers and run ids are invented — this tree is not a live report directory.

Report pages are **not** wizard packages. Do not run `wyvern wizard lint` here
and do not add wizard finish or stack-navigation helpers.

## Single-panel shortcut

Open one fragment without a manifest (`xhtml-suffix`, view mode):

```bash
wyvern share/wyvern/examples/xhtml-review/panels/fail-1.xhtml
```

Headless / CI:

```bash
WYVERN_VIEWER=none wyvern share/wyvern/examples/xhtml-review/panels/fail-1.xhtml
```

## View mode (3 failures + proposal)

```bash
wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json
```

```bash
WYVERN_VIEWER=none wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json
```

Validate the manifest without opening a host:

```bash
python3 scripts/ext/xhtml_report.py --validate-manifest share/wyvern/examples/xhtml-review/review-view.json
```

## Review mode (comments + Approve/Cancel)

```bash
wyvern report-xhtml --review share/wyvern/examples/xhtml-review/review-review.json
```

`review-review.json` already sets `"mode": "review"`, so the same tree also
expands through `wyvern report-xhtml share/wyvern/examples/xhtml-review/review-review.json`.

Finish stdout is `{ "button": "finish", "data": { "approved", "comments", "panels" } }`.
OS-close / timeout emits `{ "button": "dismissed" }` with no `data`.

## What to look for

| Panel | Status | Distinct fabricated signal |
|-------|--------|----------------------------|
| `panels/fail-1.xhtml` | FAIL | 12480 admissions/s below 18000 floor |
| `panels/fail-2.xhtml` | FAIL | 47.2 ms p99 vs 12.0 ms budget |
| `panels/fail-3.xhtml` | FAIL | 2/5 shards, 38 s replay lag |
| `panels/proposed-fix.xhtml` | **PASS** | 22140 admissions/s, 8.1 ms p99, 5/5 shards |

The proposal pane uses `role: "proposal"` so the frame applies `.pane--proposal`.
