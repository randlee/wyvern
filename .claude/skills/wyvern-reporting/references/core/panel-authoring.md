# Panel authoring (sc-compose XHTML fragments)

Load from the Layer 0 router when authoring or fixing `.xhtml` panels. Report
panels are static fragments (or full documents) that preexec stitches into a
`type: "report"` frame. Contract:
[xhtml-reporting-contract.md](../../../../../docs/plans/phase-H/xhtml-reporting-contract.md)
§ XHTML panel authoring.

## Prefer a fragment

Emit a single XHTML section. This is the atm-core benchmark-run shape and
the review-scroll default:

```xhtml
<section xmlns="http://www.w3.org/1999/xhtml" data-testid="report-panel">
  <h2>Panel heading</h2>
  <p>Describe the finding.</p>
</section>
```

Starter template: [templates/panel.xhtml.j2](../../templates/panel.xhtml.j2).

Render with sc-compose (or the wyvern compose extension):

```bash
wyvern compose render --root .claude/skills/wyvern-reporting --file templates/panel.xhtml.j2 \
  --var heading="Fail 1" --var body="admissions/s out of band." --var testid="fail-1"
```

Write the result next to the manifest, typically `panels/<name>.xhtml`.

## Full documents are allowed

Smoke panes may include `<!DOCTYPE html>` and a full document. Preexec
accepts both:

- If a `<body>` is present, the host uses the inner HTML.
- If fragment extraction fails, preexec may embed via `<iframe srcdoc>` —
  prefer an inline `<section>` so review mode can scroll one document.

## Rules

- File suffix **must** be `.xhtml` (manifest `path` pattern and
  `xhtml-suffix` match).
- Paths in the manifest are relative to the manifest directory; no `..`,
  no absolute paths.
- Keep each panel self-contained: headings, numbers, and status text the
  reviewer can act on. Do not assume wizard page JS or shared app state.
- `role` is a CSS hint only (`failure` | `proposal` | `info`). Put the
  human-readable difference in the fragment (for example a proposed-fix
  panel must look different from a failure panel).
- atm-core may emit `class="benchmark-run"` on the section; wyvern docs
  stay repo-agnostic — mention that class only as an example.

## Single-panel shortcut

Open one fragment without a manifest:

```bash
wyvern panel.xhtml
```

That is the `xhtml-suffix` extension (`type: "report"`, `mode: "view"`).
For two or more panes, or for review, write a manifest
([review-manifest.md](review-manifest.md)).
