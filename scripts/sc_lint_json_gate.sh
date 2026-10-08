#!/usr/bin/env bash
# Gate sc-lint --json commands that exit 0 when data.status is not pass/current.
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$2" != "--" ]; then
  echo "usage: sc_lint_json_gate.sh <expected_status> -- <sc-lint-args...>" >&2
  echo "example: sc_lint_json_gate.sh pass -- lint sc-boundary --config .sc-lint.toml" >&2
  exit 2
fi

expected_status="$1"
shift 2

report="$(sc-lint --json "$@")"
printf '%s\n' "${report}"

jq -e --arg expected "${expected_status}" \
  '.ok == true and .data.status == $expected' \
  <<<"${report}" >/dev/null
