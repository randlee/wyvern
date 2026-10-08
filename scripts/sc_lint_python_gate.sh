#!/usr/bin/env bash
# Gate sc-lint Python analyzers materialized under .just/ (adapter_schema sc-lint-python-v1).
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: sc_lint_python_gate.sh <line-counts|identity-literals>" >&2
  exit 2
fi

analyzer="$1"
case "${analyzer}" in
  line-counts) script=".just/lint_line_counts.py" ;;
  identity-literals) script=".just/lint_identity_literals.py" ;;
  *)
    echo "unknown analyzer: ${analyzer}" >&2
    exit 2
    ;;
esac

if [ ! -f "${script}" ]; then
  echo "missing ${script}; run setup-sc-lint (materializes .just/*.py)" >&2
  exit 2
fi

config_args=()
if [ -f sc-lint-analyzers.toml ]; then
  config_args=(--config sc-lint-analyzers.toml)
fi

report="$(python3 "${script}" --json --root "${PWD}" "${config_args[@]}")"
printf '%s\n' "${report}"

jq -e '.ok == true and .data.status == "pass"' <<<"${report}" >/dev/null
