# Phase E / e.1 — `--interactive` stdin loop and lifecycle actions

## Status
pending

## Acceptance Criteria

- `wyvern --interactive` opens window and enters read loop on stdin
- `{"action":"hide"}` and `{"action":"show"}` toggle window visibility
- Lifecycle actions return `{"action":"...","ok":true}`
- Loop remains alive after lifecycle actions and continues waiting for the next command

## Notes
