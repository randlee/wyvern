# Review manifest and CLI

Load after [panel-authoring.md](panel-authoring.md). Contract:
[xhtml-reporting-contract.md](../../../../../docs/plans/phase-H/xhtml-reporting-contract.md)
§ Manifest and § Extension registry. Schema:
[review-manifest.schema.json](../../../../../docs/plans/phase-H/review-manifest.schema.json).

## Manifest shape

Starter: [templates/review.json](../../templates/review.json).

```json
{
  "title": "Failed benchmark panels",
  "mode": "review",
  "panels": [
    { "path": "panels/fail-1.xhtml", "label": "Fail 1", "role": "failure" },
    { "path": "panels/fail-2.xhtml", "label": "Fail 2", "role": "failure" },
    { "path": "panels/fail-3.xhtml", "label": "Fail 3", "role": "failure" },
    { "path": "panels/proposed-fix.xhtml", "label": "Proposed fix", "role": "proposal" }
  ]
}
```

| Field | Required | Meaning |
|-------|----------|---------|
| `title` | yes | Report window title |
| `mode` | no | `"view"` (default) or `"review"`. CLI `--review` overrides to `"review"`. |
| `panels` | yes | ≥1 and ≤32 entries |
| `panels[].path` | yes | `.xhtml` path relative to the manifest directory |
| `panels[].label` | no | Pane heading (defaults to basename) |
| `panels[].role` | no | `failure` \| `proposal` \| `info` — CSS class only |

Unknown top-level keys are rejected. Stitched HTML must stay under 4 MiB.

Set `role: "proposal"` on the fix panel so the frame applies
`.pane--proposal`. Order is document order: failures first, proposal last
is the usual review layout.

## Validate

Same check the preexec script uses (schema plus panel files relative to
the manifest). The starter includes `templates/panels/*.xhtml` so this
command exits 0:

```bash
python3 scripts/ext/xhtml_report.py --validate-manifest .claude/skills/wyvern-reporting/templates/review.json
```

For a real tree, the same command against your `review.json` plus a
`--viewer none` expand:

```bash
WYVERN_VIEWER=none wyvern report-xhtml path/to/review.json
```

Missing panel paths fail preexec (non-zero) and name the missing file on
stderr.

## CLI

```bash
wyvern report-xhtml path/to/review.json
wyvern report-xhtml --review path/to/review.json
```

| Invocation | Extension id | Frame |
|------------|--------------|--------|
| `wyvern report-xhtml <manifest.json>` | `report-xhtml` | Array (or review if `mode` is `"review"`) |
| `wyvern report-xhtml --review <manifest.json>` | `report-xhtml-review` | Review footer + finish POST |
| `wyvern panel.xhtml` | `xhtml-suffix` | Single wrapped panel |

`--review` is a longer argv prefix (`report-xhtml` then `--review`) so it
wins over the view command. Preexec writes `{tmpdir}/pages/view.xhtml` and
`{tmpdir}/report-command.json` (`type: "report"`, `page: "pages/view.xhtml"`).

## Catalog (cross-link)

`wyvern extensions list` is the skill index. Confirm the three report
entries and their examples:

```bash
wyvern extensions list
wyvern extensions list --json
wyvern extensions show xhtml-suffix
wyvern extensions show report-xhtml
wyvern extensions show report-xhtml-review
```

Expected catalog examples (REQ-0137 parity with `share/wyvern/extensions.json`):

- `xhtml-suffix` — `wyvern panel.xhtml`
- `report-xhtml` — `wyvern report-xhtml path/to/review.json`
- `report-xhtml-review` — `wyvern report-xhtml --review path/to/review.json`

`expands_to` for all three is `"report"`. If a suffix or prefix is unknown,
stderr near-miss recovery points at `wyvern extensions list`.

## Example paths

| Path | Notes |
|------|-------|
| `.claude/skills/wyvern-reporting/templates/review.json` | Schema-valid starter |
| `crates/wyvern/tests/fixtures/xhtml-review/view.json` | View-mode fixture |
| `crates/wyvern/tests/fixtures/xhtml-review/review.json` | Review-mode fixture |
| `share/wyvern/examples/xhtml-review/review-view.json` | Packaged view example (h.5) |
| `share/wyvern/examples/xhtml-review/review-review.json` | Packaged review example (h.5) |
