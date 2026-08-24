# sc-compose snippets for wizard HTML

Copy this folder beside your wizard package when you want **Jinja2-rendered**
page bodies instead of hand-written HTML.

## Prerequisites

- `sc-compose` on `PATH` (Phase F compose extension)
- `wyvern` with `compose render` registered in `share/wyvern/extensions.json`

## Render one page

From the wizard root (parent of `pages/`):

```bash
sc-compose render --root . --file templates/sc-compose/page.j2 \
  --var-file templates/sc-compose/vars.json \
  --output pages/preview.html
```

Or via Wyvern (runs preexec + opens wizard):

```bash
wyvern compose render --root . --file templates/sc-compose/page.j2 \
  --var-file templates/sc-compose/vars.json
```

## Wire `wizard.json`

Point `page.html` at the rendered file:

```json
{
  "page": {
    "id": "compose-preview",
    "title": "Compose preview",
    "html": "pages/preview.html"
  }
}
```

## Gates

After render: G1 schema validate, G4 `wyvern wizard lint`, declare
`config.dataflow` if the page exports stack keys.

## Authority

- [f3-compose-extension.md](../../../../../docs/plans/phase-F/f3-compose-extension.md)
- Fixture: `fixtures/compose-minimal/` in the Wyvern repo
