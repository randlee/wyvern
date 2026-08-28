#!/usr/bin/env bash
# Re-vendor publish workflows from ../sc-publish (single source of truth).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
git_common="$(git -C "${repo_root}" rev-parse --git-common-dir)"
main_repo="$(cd "$(dirname "${git_common}")" && pwd)"
sc_publish_root="$(cd "${main_repo}/../sc-publish" && pwd)"
kit="${sc_publish_root}/plugins/sc-publish"
venv="${SC_PUBLISH_VENV:-${repo_root}/.sc-publish-venv}"
input="${repo_root}/release/install.json"

if [[ ! -d "${sc_publish_root}/.git" ]]; then
  echo "Missing sc-publish checkout at ${sc_publish_root}" >&2
  echo "Clone: git clone https://github.com/randlee/sc-publish.git ${sc_publish_root}" >&2
  exit 1
fi

SC_PUBLISH_REF="${SC_PUBLISH_REF:-917ddda191e72bad641c612fd54b87d74017d95b}"
SC_PUBLISH_EXPECTED_SHA="${SC_PUBLISH_EXPECTED_SHA:-917ddda191e72bad641c612fd54b87d74017d95b}"

(
  cd "${sc_publish_root}"
  git fetch origin
  if ! git checkout "${SC_PUBLISH_REF}"; then
    echo "sc-publish ref ${SC_PUBLISH_REF} not found" >&2
    exit 1
  fi
  actual="$(git rev-parse HEAD)"
  expected="$(git rev-parse "${SC_PUBLISH_EXPECTED_SHA}^{commit}" 2>/dev/null || true)"
  if [[ -z "${expected}" || "${actual}" != "${expected}" ]]; then
    echo "sc-publish HEAD ${actual} != expected ${SC_PUBLISH_EXPECTED_SHA}" >&2
    exit 1
  fi
)

if [[ ! -f "${input}" ]]; then
  echo "Missing ${input}" >&2
  exit 1
fi

publish_python="$(
  python3 "${kit}/.github/scripts/bootstrap_sc_compose.py" --venv "${venv}"
)"
"${publish_python}" "${kit}/install.py" --input "${input}" "${repo_root}"
"${publish_python}" "${kit}/install.py" --dry-run --input "${input}" "${repo_root}"
echo "sc-publish kit synced from ${sc_publish_root} @ $(git -C "${sc_publish_root}" rev-parse --short HEAD) (pin ${SC_PUBLISH_REF})"
