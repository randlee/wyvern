# Phase D / d.1 — Wizard host: HTML load and config injection

## Status
pending

## Acceptance Criteria

- `{"type":"wizard","page":{"id":"start","title":"Start","html":"path/to/wizard.html"},"config":{}}` opens the initial HTML file
- `config` object injected into the page as `window.wyvern.config` on load
- Wizard window uses explicit `width`/`height` when provided
- Minimize enabled for wizard windows

## Notes
