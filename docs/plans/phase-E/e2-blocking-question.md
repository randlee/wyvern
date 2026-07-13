# Phase E / e.2 — Blocking dialogs and `exit` in interactive mode

## Status
pending

## Acceptance Criteria

- Blocking dialog commands return their normal JSON result on stdout; loop resumes afterward
- `{"action":"exit"}` closes window and terminates process cleanly
- Window close by user also terminates process and loop
- `--persistent` accepted as alias for `--interactive`

## Notes
