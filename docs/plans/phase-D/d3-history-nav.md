# Phase D / d.3 — Browser-history navigation model

## Status
pending

## Acceptance Criteria

- Forward navigation pushes page + data, advances cursor
- Back moves cursor back without truncating forward history
- Forward on same next-page restores cached page data
- Forward on different next-page truncates forward history and pushes new page
- History state verified by unit tests covering all four cases

## Notes

Implements the cursor-over-array history (ADR-0005).
