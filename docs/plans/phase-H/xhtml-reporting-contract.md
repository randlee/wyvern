# XHTML reporting contract (Phase H)

Normative contract for **`type: "report"`** and the **`report-xhtml`** extension
family. Phase H sprint docs implement this; they do not redefine it.

---

## Motivation

Wizard (`type: "wizard"`) carries stack navigation, history, and wizard-api
semantics. **Report viewing** is a single static document — optionally with one
terminal **review** action. Overloading wizard for XHTML review surfaces confuses
authoring skills and lint profiles.

**Decision:** Add `type: "report"` — static HTML/XHTML under `--ui-root`, optional
review finish, no stack.

**Architecture:** Formal **ADR-0025** and ADR-0022 amendment land in **h.1**
(`docs/architecture.md`, `docs/plans/phase-F/cli-extensions-contract.md`). Phase H
adds the first new `Command` variant since Phase B; extensions still expand to
validated `Command` JSON (ADR-0022 Path A unchanged for MCP).

---

## Command JSON (`type: "report"`)

```json
{
  "type": "report",
  "title": "XHTML review",
  "page": "pages/view.xhtml",
  "mode": "review",
  "panels": [
    { "path": "panels/fail-1.xhtml", "label": "Fail 1", "role": "failure" }
  ],
  "width": 960,
  "height": 720
}
```

| Field | Required | Meaning |
|-------|----------|---------|
| `type` | yes | `"report"` |
| `title` | yes | Window / viewer title |
| `page` | yes | Path relative to `--ui-root` (`.html` or `.xhtml`) |
| `mode` | no | `"view"` (default) or `"review"` |
| `panels` | when `mode: "review"` | Manifest panel entries (path, optional label/role) — host finish validation authority |
| `width`, `height` | no | Viewer hints (same as wizard) |

**Validation:** `page` must resolve to an existing file under `ui_root` after
host canonicalization (same path rules as wizard `page.html`).

**Not allowed on report commands:** `config`, `workflow`, wizard `page.id`,
stack descriptors, `next_wizard`.

Review-mode commands written by preexec **include** manifest `panels` in
`report-command.json` so the host can validate finish POST `panels` against an
authoritative list (see § Review finish JSON).

### Rust types (normative — implemented h.1/h.3)

See sprint h.1 for `ReportCommand`, `ReportMode`, page/title newtypes, and
`ReportResult` / `ReportFinishData` wire shapes. `Command::Report(ReportCommand)` is
the sole new `Command` variant; `CommandResult::Report(ReportResult)` covers both
view dismiss and review finish on stdout.

---

## Host bind (`Command::Report`)

Report sessions use a **dedicated bind arm** (third discriminant — **not** `is_wizard`):

| Step | Behavior |
|------|----------|
| Bind discriminant | `report` — **forbidden:** `is_wizard=true`, `/wizard/` dialog URL, wizard static mount |
| Validate page | `require_report_page(ui_root, page)` — file must exist under `ui_root` |
| Forbidden | `require_type_dir` / `{ui_root}/report/index.html` packaged layout |
| Dialog URL | `/report/{page}` where `{page}` is command `page` (e.g. `pages/view.xhtml`) |
| Static mount | `ServeDir` nest at `/report` from session `ui_root` override |
| GET `/api/dialog` | Rejected (report is static `/report/{page}` only) |
| Viewer OS-close | Uses `/api/result` dismiss path — **not** wizard finish (`/api/wizard/*`) |

Preexec writes generated HTML to `{tmpdir}/pages/view.xhtml`; expand sets
`host.ui_root: "{tmpdir}"` and `page: "pages/view.xhtml"`.

---

## Host HTTP surface

| Route | Method | Purpose |
|-------|--------|---------|
| `/report/{page_path}` | GET | Static page (via `ServeDir` under `ui_root`) |
| `/shared/*` | GET | Packaged shared assets (`report-base.css`; `report-review.js` in review mode). Same mount pattern as wizard/dialog shared UI — available whenever the report host session is active (view **and** review). |
| `/api/report/finish` | POST | Review mode terminal action → `CommandResult` (registered only when `mode: "review"`) |

View mode (`mode: "view"`): user closes window → `{"button":"dismissed"}` (same
semantics as chrome dismiss unless review controls present).

Review mode (`mode: "review"`): page POSTs finish payload; host emits result JSON.

---

## Review finish JSON (stdout)

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

**OS-close / timeout (review mode):** viewer OS-close and session timeout use the
shared `POST /api/result` path and emit `{ "button": "dismissed" }` with **no** `data`
(semantically distinct from Approve/Cancel finish). Finish and dismiss are mutually
exclusive terminal actions per session.

---

## Frame profiles

Three preexec-built HTML shells (see `scripts/ext/xhtml_report.py`):

### 1. `basic-single`

Wrap one XHTML fragment (or full document) in:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{title}</title>
  <link rel="stylesheet" href="/shared/report-base.css" />
</head>
<body class="report report--single">
  <main class="report-body">{fragment_or_document_body}</main>
</body>
</html>
```

### 2. `basic-array`

Same head; body contains ordered panes:

```html
<section class="pane" data-role="failure" data-path="panels/fail-1.xhtml">
  <header class="pane-label">Fail 1</header>
  …fragment…
</section>
```

### 3. `review`

Extends `basic-array` with footer:

```html
<footer class="report-review" data-testid="report-review">
  <label for="review-comments">Comments</label>
  <textarea id="review-comments" data-testid="review-comments"></textarea>
  <nav>
    <button type="button" data-report-cancel data-testid="report-cancel">Cancel</button>
    <button type="button" data-report-approve data-testid="report-approve">Approve</button>
  </nav>
</footer>
<script src="/shared/report-review.js"></script>
```

`report-review.js` posts to `/api/report/finish` (minimal — **not** wizard-nav).

---

## Manifest (`review.json`)

JSON Schema: [review-manifest.schema.json](review-manifest.schema.json).

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
| `title` | yes | Report title |
| `mode` | no | `"view"` or `"review"` (CLI `--review` overrides to `"review"`) |
| `panels` | yes (array) | ≥1 entry |
| `panels[].path` | yes | `.xhtml` path relative to manifest directory |
| `panels[].label` | no | Pane heading (defaults to basename) |
| `panels[].role` | no | `failure` \| `proposal` \| `info` — affects CSS class only |

---

## Extension registry entries

### `xhtml-suffix` (h.1)

Same expand shape as `html-suffix` but match `.xhtml` and **always** run preexec
frame wrapper → `type: "report"`, `mode: "view"`.

### `report-xhtml` (h.2–h.3)

Preexec reads the manifest, stitches panes, and writes **`{tmpdir}/report-command.json`**
(validated `type: "report"` command). Expand uses Phase F **`command_from_file`**
(no custom template placeholders).

```json
{
  "id": "report-xhtml",
  "description": "Open an ordered array of XHTML panels as one report view.",
  "examples": ["wyvern report-xhtml path/to/review.json"],
  "match": { "argv_prefix": ["report-xhtml"], "arg_suffix": ".json" },
  "preexec": {
    "cmd": "python3",
    "args": [
      "{wyvern_share}/scripts/ext/xhtml_report.py",
      "--manifest", "{path}",
      "--out", "{tmpdir}/pages/view.xhtml",
      "--command-out", "{tmpdir}/report-command.json"
    ],
    "requires": ["python3"]
  },
  "expand": {
    "command_from_file": "{tmpdir}/report-command.json",
    "host": { "ui_root": "{tmpdir}" }
  }
}
```

**Review override (h.3):** register a **second** extension with a longer prefix so
no new `{arg:*:flag}` template syntax is required:

```json
{
  "id": "report-xhtml-review",
  "description": "Open XHTML panels in review mode (comments + Approve/Cancel).",
  "examples": ["wyvern report-xhtml --review path/to/review.json"],
  "match": { "argv_prefix": ["report-xhtml", "--review"], "arg_suffix": ".json" },
  "extends": "report-xhtml",
  "preexec": {
    "args": [
      "{wyvern_share}/scripts/ext/xhtml_report.py",
      "--manifest", "{path}",
      "--out", "{tmpdir}/pages/view.xhtml",
      "--command-out", "{tmpdir}/report-command.json",
      "--force-mode", "review"
    ]
  }
}
```

Registry ordering: **`report-xhtml-review` before `report-xhtml`** so the longer
prefix wins. Preexec sets `mode` from manifest `mode` (default `"view"`) unless
`--force-mode review` is present. `report-command.json` always includes `title`
from manifest and `page: "pages/view.xhtml"`.

CLI:

```bash
wyvern report-xhtml path/to/review.json
wyvern report-xhtml --review path/to/review.json
```

---

## XHTML panel authoring (atm-core alignment)

sc-compose templates may emit:

- **Fragments:** `<section xmlns="http://www.w3.org/1999/xhtml" …>` (benchmark-run)
- **Full documents:** smoke panes with `<!DOCTYPE html>` (optional)

Preexec MUST accept both: if input is a full document, extract `<body>` inner
HTML or embed via `<iframe srcdoc>` only when fragment extraction fails (prefer
inline section for review scroll).

Panel authoring guidance ships in **`wyvern-reporting`** skill refs — not here.

---

## Boundaries

- Report surfaces MUST NOT register as wizard lint targets (WIZARD-LINT-*).
- Report pages MUST NOT require `data-wizard-nav` or wizard finish helpers.
- Phase H does not add `config.dataflow` or wizard schema fields.

---

## Error inventory (normative — h.1/h.3)

| Stage | Condition | Exit / code | Recovery |
|-------|-----------|-------------|----------|
| Extension match | Unknown suffix / prefix | `ParseError` near-miss (REQ-0136) | `wyvern extensions list` |
| Preexec | Missing manifest panel path | preexec non-zero → `ExtensionError::Preexec { kind, message }` | stderr names missing file |
| Preexec | Invalid manifest JSON / schema | preexec non-zero | fix manifest; schema in `review-manifest.schema.json` |
| Preexec | Panel count > 32 or stitched HTML > 4 MiB | preexec non-zero | reduce panels or panel size per schema |
| Validate | `report-command.json` missing after preexec | `ExtensionError::InvalidCommand` | preexec must write `{tmpdir}/report-command.json` |
| Host | `page` not under `ui_root` | validation before bind | fix preexec output path |
| Host | `POST /api/report/finish` in view mode | HTTP **404** (route unregistered) | use review mode or dismiss window |
| Host | Finish unknown top-level keys | HTTP 400 `REPORT_FINISH_UNKNOWN_FIELD` | remove extra keys |
| Host | Finish `panels` mismatch authoritative command | HTTP 400 `REPORT_FINISH_PANELS_MISMATCH` | resubmit embedded manifest panels |
| Host | Finish `comments` > 32_768 chars | HTTP 400 `REPORT_FINISH_COMMENTS_TOO_LONG` | shorten comments |
| Host | Malformed finish JSON body | HTTP 400 `REPORT_FINISH_INVALID_JSON` | resubmit valid JSON |
| Host | Duplicate finish POST | HTTP 409 `REPORT_FINISH_ALREADY_COMPLETE` | single terminal action per session |
| Host | Duplicate `/api/result` after terminal action | HTTP 409 (inherit `SessionState::complete`) | session already complete |

Finish request schema (h.3): `approved` (required bool), `comments` (string, max
32_768 chars), `panels` (required array, echo manifest entries). Unknown top-level
keys rejected.
