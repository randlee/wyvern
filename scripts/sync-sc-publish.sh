#!/usr/bin/env bash
# Materialize sc-publish kit into this consumer repo from a pinned upstream SHA.
# Does NOT mutate a shared sibling ../sc-publish checkout (other repos may use it).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pin_file="${repo_root}/release/sc-publish-pin.toml"
input="${repo_root}/release/install.json"
kit_cache="${SC_PUBLISH_CACHE:-${repo_root}/.sc-publish-kit}"
venv="${SC_PUBLISH_VENV:-${repo_root}/.sc-publish-venv}"

if [[ ! -f "${pin_file}" ]]; then
  echo "Missing ${pin_file}" >&2
  exit 1
fi

readarray -t pin_values < <(
  python3 - "${pin_file}" <<'PY'
import sys
import tomllib
from pathlib import Path

data = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
repo = data.get("repository")
rev = data.get("revision")
if not isinstance(repo, str) or not repo.strip():
    raise SystemExit("release/sc-publish-pin.toml: repository is required")
if not isinstance(rev, str) or len(rev) != 40:
    raise SystemExit("release/sc-publish-pin.toml: revision must be a 40-char commit SHA")
print(repo.strip())
print(rev.strip())
PY
)
sc_publish_repo="${pin_values[0]}"
expected_sha="${pin_values[1]}"

SC_PUBLISH_REF="${SC_PUBLISH_REF:-${expected_sha}}"
SC_PUBLISH_EXPECTED_SHA="${SC_PUBLISH_EXPECTED_SHA:-${expected_sha}}"

if [[ "${SC_PUBLISH_EXPECTED_SHA}" != "${expected_sha}" ]]; then
  echo "SC_PUBLISH_EXPECTED_SHA ${SC_PUBLISH_EXPECTED_SHA} != pin file ${expected_sha}" >&2
  exit 1
fi

if [[ ! -d "${kit_cache}/.git" ]]; then
  git clone --quiet "${sc_publish_repo}" "${kit_cache}"
fi

(
  cd "${kit_cache}"
  git fetch origin --quiet
  if ! git checkout --quiet "${SC_PUBLISH_REF}"; then
    echo "sc-publish ref ${SC_PUBLISH_REF} not found" >&2
    exit 1
  fi
  actual="$(git rev-parse HEAD)"
  if [[ "${actual}" != "${SC_PUBLISH_EXPECTED_SHA}" ]]; then
    echo "sc-publish HEAD ${actual} != expected ${SC_PUBLISH_EXPECTED_SHA}" >&2
    exit 1
  fi
)

kit="${kit_cache}/plugins/sc-publish"

if [[ ! -f "${input}" ]]; then
  echo "Missing ${input}" >&2
  exit 1
fi

publish_python="$(
  python3 "${kit}/.github/scripts/bootstrap_sc_compose.py" --venv "${venv}"
)"
"${publish_python}" "${kit}/install.py" --input "${input}" "${repo_root}"
"${publish_python}" "${kit}/install.py" --dry-run --input "${input}" "${repo_root}"
echo "sc-publish kit synced from ${kit_cache} @ $(git -C "${kit_cache}" rev-parse --short HEAD) (pin ${SC_PUBLISH_REF})"
