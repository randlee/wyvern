# Sprint j.5 — sc-lint extended analyzers (line-counts, identity-literals)

**Status:** In progress on stack #160 (`feature/sc-lint-0.5.0-portability`).

## Goal

Use sc-lint **0.5.0** Python analyzers shipped in the release source bundle (`.just/lint_*.py`) for Wyvern policy beyond boundary/portability/runtime.

## Deliverables

| Item | Owner file | CI |
|------|------------|-----|
| Analyzer policy | [`sc-lint-analyzers.toml`](../../../sc-lint-analyzers.toml) | `scripts/sc_lint_python_gate.sh` |
| line-counts | `.just/lint_line_counts.py` (ephemeral) | Boundary lint job |
| identity-literals | `.just/lint_identity_literals.py` | Boundary lint job |
| Release binaries | Future `sc-lint-line-counts` in bundle | Replace Python gate when published |

## Notes

- Top-level `sc-lint lint line-counts` expects a sibling binary not yet in the GitHub release tarball; Wyvern invokes the Python adapter until the kit publishes those backends.
- Identity literal scanning requires Rust string literals to avoid `\U` false positives in Windows path fixtures (use `C:/Users/...` in tests).

## Acceptance

- `bash scripts/sc_lint_python_gate.sh line-counts` → pass
- `bash scripts/sc_lint_python_gate.sh identity-literals` → pass
- Policy changes require updating `sc-lint-analyzers.toml`, not `.sc-lint.toml`
