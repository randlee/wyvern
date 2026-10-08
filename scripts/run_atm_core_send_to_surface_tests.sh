#!/usr/bin/env bash
# Run atm-core Send-To surface tests and AQ5 degradation evidence against a
# pinned atm-core checkout. Optionally probe the real Wyvern binary on PATH.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [[ -z "${ATM_CORE_CHECKOUT:-}" ]]; then
  ATM_CORE_CHECKOUT="$("${ROOT}/scripts/verify_atm_core_send_to_contract.sh" | tail -1)"
  export ATM_CORE_CHECKOUT
fi

PIN="${ROOT}/release/atm-core-send-to-pin.toml"
WYVERN_PIN="$(python3 -c "import tomllib; from pathlib import Path; print(tomllib.loads(Path('${PIN}').read_text())['atm-core']['send-to']['wyvern_pin'])")"
export WYVERN_PIN

echo "run_atm_core_send_to_surface_tests: atm-core checkout ${ATM_CORE_CHECKOUT}"

python3 -m unittest discover -s "${ATM_CORE_CHECKOUT}/.just/tests" -p 'test_send_to_surface.py' -v

python3 "${ATM_CORE_CHECKOUT}/scripts/phase-aq/run_aq5_wyvern_degradation_evidence.py" \
  --host "wyvern-ci" \
  --evidence-dir "${ROOT}/target/atm-send-to-evidence"

if [[ -n "${WYVERN_BIN:-}" ]]; then
  export PATH="$(dirname "${WYVERN_BIN}"):${PATH}"
  export ATM_SEND_TO_WYVERN_BIN="${WYVERN_BIN}"
fi

if command -v wyvern >/dev/null 2>&1 || [[ -n "${WYVERN_BIN:-}" ]]; then
  python3 "${ATM_CORE_CHECKOUT}/scripts/send-to/probe_wyvern.py" \
    --pin "${WYVERN_PIN}" \
    --asset "${ROOT}/examples/wizards/atm-pick-member/pages/pick-member.html"
  echo "run_atm_core_send_to_surface_tests OK: probe_wyvern.py against real Wyvern"
else
  echo "run_atm_core_send_to_surface_tests: skipping probe_wyvern.py (no wyvern on PATH)" >&2
  exit 1
fi

echo "run_atm_core_send_to_surface_tests OK"
