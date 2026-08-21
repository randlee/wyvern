---
id: g.6
title: Template catalog wizard + apply workflow
status: planning
branch: feature/phase-G-g6-template-wizard
target: integrate/phase-G
---

# Sprint g.6 — Template catalog + workflow apply

## Goal

Example (b): bundled template catalog, picker wizard, and g.4 post apply via `apply-template.py` (REQ-0125). The welcome Templates page finishes with `next_wizard` into the picker (REQ-0126).

## Hard dependencies

- g.4 merged (`WorkflowRunner`, chain loop, `--workflow-dry-run`)

## Deliverables

| Path | Purpose |
|------|---------|
| `share/wyvern/templates/pytest/` | Content template + `template.manifest.json` |
| `share/wyvern/templates/github-workflow/` | Content template + manifest |
| `share/wyvern/templates/nunit/` | Content template + manifest |
| `share/wyvern/templates/xunit/` | Content template + manifest |
| `share/wyvern/templates/benchmark-dotnet/` | Content template + manifest |
| `share/wyvern/templates/wizard/minimal/` | Authoring skeleton (N=1) + `template.manifest.json` |
| `share/wyvern/templates/wizard/two-step/` | Authoring skeleton (N=2) + `template.manifest.json` |
| `share/wyvern/examples/template-picker/wizard.json` | Picker + `workflow.post` |
| `share/wyvern/examples/template-picker/pages/*.html` | Grid + form + review |
| `scripts/ext/apply-template.py` | Copy + `{var}` substitution; `--dry-run`; `--force` (see overwrite matrix) |
| `share/wyvern/welcome/pages/templates.html` | Catalog + required `next_wizard` into picker |
| `crates/wyvern/tests/workflow_apply_template.rs` | Dry-run plan; apply; re-apply markers |
| `crates/wyvern/tests/workflow_welcome_chain_templates.rs` | Welcome Templates finish JSON → CLI resolves next hop |

Five **content** templates (`pytest`, `github-workflow`, `nunit`, `xunit`, `benchmark-dotnet`) plus two **authoring** skeletons. Typical outputs: `tests/test_example.py`, `.github/workflows/<name>.yml`, `*Tests.cs`, `*Benchmarks.cs`.

### Contracts

Page JS reads `config.templates` only (no directory scan). All seven ids are picker rows, including both authoring skeletons.

```json
{
  "type": "wizard",
  "page": { "id": "pick", "title": "Templates", "html": "pages/pick.html" },
  "config": {
    "templates": [
      { "id": "pytest", "label": "pytest", "default_output_path": "tests/test_example.py", "variables": [{ "name": "module_name", "default": "example" }] },
      { "id": "github-workflow", "label": "GitHub workflow", "default_output_path": ".github/workflows/ci.yml", "variables": [{ "name": "name", "default": "ci" }] },
      { "id": "nunit", "label": "NUnit", "default_output_path": "ExampleTests.cs", "variables": [{ "name": "class_name", "default": "ExampleTests" }] },
      { "id": "xunit", "label": "xUnit", "default_output_path": "ExampleTests.cs", "variables": [{ "name": "class_name", "default": "ExampleTests" }] },
      { "id": "benchmark-dotnet", "label": "Benchmark .NET", "default_output_path": "ExampleBenchmarks.cs", "variables": [{ "name": "class_name", "default": "ExampleBenchmarks" }] },
      { "id": "wizard/minimal", "label": "Wizard skeleton (1 page)", "default_output_path": "wizard-minimal/", "variables": [] },
      { "id": "wizard/two-step", "label": "Wizard skeleton (2 pages)", "default_output_path": "wizard-two-step/", "variables": [] }
    ]
  },
  "workflow": {
    "post": "{wyvern_share}/scripts/ext/apply-template.py"
  }
}
```

```json
{
  "id": "pytest",
  "label": "pytest",
  "description": "Python pytest starter",
  "default_output_path": "tests/test_example.py",
  "files": ["test_example.py"],
  "variables": [{ "name": "module_name", "default": "example" }]
}
```

Finish `data`:

```json
{
  "template_id": "pytest",
  "variables": { "module_name": "example" },
  "output_path": "tests/test_example.py"
}
```

### `apply-template.py` contract (AC 3–4)

| Item | Rule |
|------|------|
| stdin | Full finish JSON (g.4 post). `--finish-file PATH` is test-only |
| `--dry-run` | Print copy plan; write nothing. CLI `--workflow-dry-run` appends this flag |
| `--force` | Script/test-only. Shipped `WorkflowRunner` post uses the Default column only |
| Template root | `$WYVERN_SHARE/templates/<template_id>/` (g.4 spawn sets `WYVERN_SHARE`) |
| `template_id` | Must be one of the seven `config.templates[].id` values. Resolve only under the templates root after canonicalize. Reject `..` and symlink escape. Slash is allowed only for `wizard/minimal` and `wizard/two-step`. |
| Output root | `output_path` is relative to `$WYVERN_REPO_ROOT` if set, else process cwd |
| `{var}` | Substitute `{name}` from finish `data.variables` in copied file contents |
| Marker | Sidecar `<dest>.wyvern.json` next to every written file (not in-file JSON). Tagged = sidecar exists and `managed_by` is `wyvern:template`. |

Overwrite matrix:

| Situation | Default | `--force` |
|-----------|---------|-----------|
| Destination file missing | create | create |
| Destination has sidecar `<dest>.wyvern.json` with `managed_by = wyvern:template` | overwrite | overwrite |
| Destination exists and is untagged | **fail** (non-zero; write nothing for that file) | overwrite |

Shipped apply path is **automatic post only**. Do not document `cp -R` as an install path.

Sidecar bytes (same for `.py`, `.yml`, `.cs` — e.g. `tests/test_example.py.wyvern.json`):

```json
{ "managed_by": "wyvern:template", "template_id": "pytest", "version": 1 }
```

Welcome Templates finish (required):

```json
{
  "button": "finish",
  "data": {},
  "stack": [],
  "next_wizard": {
    "path": "{wyvern_share}/examples/template-picker/wizard.json",
    "input": { "from": "welcome" },
    "ui_root": "{wyvern_share}/examples/template-picker"
  }
}
```

## Acceptance criteria

1. All five content templates and both wizard skeletons exist with a valid `template.manifest.json` and at least one source file each.
2. Picker grid is driven by `config.templates` (seven rows, including both skeletons). Finish `data` includes `template_id`, `variables`, and `output_path`.
3. Post runs via g.4 `WorkflowRunner` (REQ-0125). `--workflow-dry-run` and `apply-template.py --dry-run` print a copy plan and write nothing (see contract).
4. Apply follows the overwrite matrix in the `apply-template.py` contract.
5. Welcome Templates page emits the `next_wizard` object above; `workflow_welcome_chain_templates` asserts the CLI resolves that hop (REQ-0126).
6. `workflow_apply_template` covers dry-run, apply, tagged re-apply, untagged collision fail, and `--force` as a **script/test-only** flag.

## Required validation

```bash
cargo test -p wyvern-cli --test workflow_apply_template
cargo test -p wyvern-cli --test workflow_welcome_chain_templates
cargo publish --dry-run -p wyvern-cli --locked
```

```bash
# manual (not a QA gate):
# wyvern share/wyvern/examples/template-picker/wizard.json \
#   --ui-root share/wyvern/examples/template-picker --viewer none
```

## Non-closure

- `scripts/ext/probe-destination.py` destination-exists pre
- In-browser full-file editor (customize step is form fields only)
- User template registry outside `share/wyvern/templates/`
- sc-compose generation

## Authority

- REQ-0125, REQ-0126
- ADR-0023, ADR-0024
- [template-catalog-workflow.md](template-catalog-workflow.md)
- [g4-welcome-guide-wizard.md](g4-welcome-guide-wizard.md)
