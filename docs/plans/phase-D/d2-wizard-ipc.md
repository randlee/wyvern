# Phase D / d.2 — Wizard IPC contract

## Status
pending

## Acceptance Criteria

- Page can send: `{"action":"next","page":{...},"data":{},"next":{...}}` → host advances
- Page can send: `{"action":"back","page":{...},"data":{}}` → host navigates back
- Page can send: `{"action":"finish","page":{...},"data":{}}` → host closes + returns result
- Page can send: `{"action":"cancel"}` → host closes + returns `{"button":"cancel"}`
- Host sends on page load: `{"page":{},"page_data":{},"stack":[]}`

## Notes
