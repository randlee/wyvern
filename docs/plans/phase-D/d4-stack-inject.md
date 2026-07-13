# Phase D / d.4 — Stack injection and data restoration

## Status
pending

## Acceptance Criteria

- `stack` array in host→page message contains all prior `{page, data}` entries
- `page_data` populated with this page's previously collected data on restore
- JS on any page can access `window.wyvern.stack` to read prior answers
- Data round-trips correctly through JSON serialization

## Notes
