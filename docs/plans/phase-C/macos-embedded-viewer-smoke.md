# macOS embedded-viewer smoke checklist (c.16 AC4)

Template for recording manual smoke evidence after a release artifact is available.
**ATM-QA-004:** fill this checklist when running smoke on a packaged macOS binary
(post-merge / after `v0.1.0` tag assets exist).

## Preconditions

- [ ] Release artifact downloaded (e.g. `wyvern-macos-aarch64.tar.gz` or local `cargo build --release`)
- [ ] Archive extracted; layout contains:
  - [ ] `wyvern`
  - [ ] `wyvern-viewer`
  - [ ] `share/wyvern/ui/{message,input,markdown,question,chrome}/index.html`

## Smoke steps

```bash
# From the extracted release root:
./wyvern '{"type":"message","title":"Smoke","message":"c.16 embedded viewer","buttons":["ok"]}'
```

Expected:

- [ ] Embedded `wyvern-viewer` window opens (default `--viewer` / product default)
- [ ] Message dialog renders title + body
- [ ] Clicking **ok** closes the window
- [ ] Process exits 0 and prints JSON result on stdout with `"button":"ok"` (or equivalent label)

Optional dismiss path:

```bash
./wyvern '{"type":"message","title":"Dismiss","message":"close via window","buttons":["ok"]}'
# Close via traffic-light / OS close without clicking ok
```

- [ ] Dismiss yields dismissed / fail-safe result per contract

## Record

| Field | Value |
|-------|-------|
| Date (UTC) | |
| Operator | |
| Artifact / SHA | |
| Platform | macOS (arch: ) |
| Result | PASS / FAIL |
| Notes | |

## Evidence links

- CI green (AC3): see triage note for run id / PR checks
- Release workflow run URL (after tag):
- Screenshot / log path (optional):
