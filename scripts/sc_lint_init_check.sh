#!/usr/bin/env bash
# Compare repo-managed consumer integration files to this sc-lint binary's
# `init --just` output. Newlines/BOM are ignored because the Windows release
# binary may embed CRLF templates while the repo (and Unix --check) stay LF.
set -euo pipefail

root="${1:-$PWD}"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/sc-lint-init.XXXXXX")"
cleanup() { rm -rf "${tmp}"; }
trap cleanup EXIT

# `sc-lint init --just` writes into cwd and rejects --root.
(
  cd "${tmp}"
  sc-lint --json init --just >/dev/null
)

normalize() {
  python3 -c '
import sys
from pathlib import Path
raw = Path(sys.argv[1]).read_bytes().replace(b"\r\n", b"\n").replace(b"\r", b"\n")
if raw.startswith(b"\xef\xbb\xbf"):
    raw = raw[3:]
sys.stdout.buffer.write(raw)
' "$1"
}

fail=0
for rel in sc-lint.toml Justfile .sc-lint/bootstrap .sc-lint/bootstrap.ps1; do
  generated="${tmp}/${rel}"
  checked="${root}/${rel}"
  if [ ! -f "${generated}" ]; then
    echo "sc-lint init --just did not create ${rel}" >&2
    fail=1
    continue
  fi
  if [ ! -f "${checked}" ]; then
    echo "repository is missing ${rel}" >&2
    fail=1
    continue
  fi
  if ! diff -u --label "generated/${rel}" --label "repo/${rel}" \
      <(normalize "${generated}") <(normalize "${checked}"); then
    echo "consumer integration drifts from this sc-lint binary: ${rel}" >&2
    fail=1
  fi
done

if [ "${fail}" -ne 0 ]; then
  exit 1
fi
echo "consumer Just integration matches this sc-lint binary (newline-normalized)"
