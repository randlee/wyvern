#!/usr/bin/env bash
# CI gate: every publish=true crate in release/publish-artifacts.toml packages a
# non-empty README and sets documentation = https://docs.rs/<package>.
# Uses `cargo package --allow-dirty --no-verify` so the check inspects the
# crates.io tarball without a second compile (workspace CI already builds).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="${ROOT}/release/publish-artifacts.toml"
TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT}/target}"

if [[ ! -f "$MANIFEST" ]]; then
  echo "check-crates-publish-manifest: missing ${MANIFEST}" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "check-crates-publish-manifest: python3 is required" >&2
  exit 1
fi

ROWS=()
while IFS= read -r row; do
  ROWS+=("$row")
done < <(python3 - "$MANIFEST" <<'PY'
import re
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()
blocks = re.split(r"(?m)^\[\[crates\]\]\s*$", text)[1:]
found = False
for block in blocks:
    publish = re.search(r"(?m)^publish\s*=\s*(true|false)\s*$", block)
    if not publish or publish.group(1) != "true":
        continue
    package = re.search(r'(?m)^package\s*=\s*"([^"]+)"\s*$', block)
    cargo_toml = re.search(r'(?m)^cargo_toml\s*=\s*"([^"]+)"\s*$', block)
    if not package or not cargo_toml:
        sys.stderr.write("publish=true crate is missing package or cargo_toml\n")
        sys.exit(1)
    found = True
    sys.stdout.write(f"{package.group(1)}\t{cargo_toml.group(1)}\n")
if not found:
    sys.stderr.write("no publish=true crates in manifest\n")
    sys.exit(1)
PY
)

fail=0
for row in "${ROWS[@]}"; do
  package="${row%%$'\t'*}"
  rel_toml="${row#*$'\t'}"
  cargo_toml="${ROOT}/${rel_toml}"
  expected_docs="https://docs.rs/${package}"
  crate_dir="$(dirname "$cargo_toml")"

  if [[ ! -f "$cargo_toml" ]]; then
    echo "check-crates-publish-manifest: missing ${cargo_toml}" >&2
    fail=1
    continue
  fi

  if ! python3 - "$cargo_toml" "$package" "$expected_docs" <<'PY'
import re
import sys
from pathlib import Path

path, package, expected_docs = sys.argv[1:]
text = Path(path).read_text()
readme = re.search(r'(?m)^readme\s*=\s*"([^"]+)"\s*$', text)
include = re.search(r"(?ms)^include\s*=\s*\[(.*?)\]", text)
documentation = re.search(r'(?m)^documentation\s*=\s*"([^"]+)"\s*$', text)
errors = []
if not readme or readme.group(1) != "README.md":
    errors.append('readme must be "README.md"')
if not include or "README.md" not in re.findall(r'"([^"]+)"', include.group(1)):
    errors.append('include must list "README.md"')
if not documentation or documentation.group(1) != expected_docs:
    errors.append(f'documentation must be "{expected_docs}"')
if errors:
    sys.stderr.write(f"{package}: " + "; ".join(errors) + "\n")
    sys.exit(1)
PY
  then
    fail=1
    continue
  fi

  readme_file="${crate_dir}/README.md"
  if [[ ! -f "$readme_file" ]]; then
    echo "check-crates-publish-manifest: ${package} README.md is missing" >&2
    fail=1
    continue
  fi
  readme_nonspace="$(tr -d '[:space:]' <"$readme_file" | wc -c | tr -d ' ')"
  if [[ "$readme_nonspace" -eq 0 ]]; then
    echo "check-crates-publish-manifest: ${package} README.md is empty" >&2
    fail=1
    continue
  fi

  echo "check-crates-publish-manifest: packaging ${package}"
  # --list is the package file set. It succeeds before path deps exist on
  # crates.io. A full .crate tarball rewrites those deps to registry versions
  # and cannot be built until the dependency crates are published.
  list_log="$(mktemp)"
  if ! cargo package -p "$package" --allow-dirty --no-verify --list --manifest-path "${ROOT}/Cargo.toml" >"$list_log" 2>&1; then
    cat "$list_log" >&2
    echo "check-crates-publish-manifest: cargo package --list failed for ${package}" >&2
    rm -f "$list_log"
    fail=1
    continue
  fi
  if ! grep -qx "README.md" "$list_log"; then
    echo "check-crates-publish-manifest: ${package} package list has no README.md" >&2
    cat "$list_log" >&2
    rm -f "$list_log"
    fail=1
    continue
  fi
  rm -f "$list_log"

  pkg_log="$(mktemp)"
  if cargo package -p "$package" --allow-dirty --no-verify --manifest-path "${ROOT}/Cargo.toml" >"$pkg_log" 2>&1; then
    version="$(sed -n "s/^[[:space:]]*Packaging ${package} v\\([^[:space:]]*\\) (.*/\\1/p" "$pkg_log")"
    rm -f "$pkg_log"
    crate_file="${TARGET_DIR}/package/${package}-${version}.crate"
    if [[ -z "$version" || ! -f "$crate_file" ]]; then
      echo "check-crates-publish-manifest: missing tarball ${crate_file}" >&2
      fail=1
      continue
    fi
    if ! readme_bytes="$(python3 - "$crate_file" "$expected_docs" <<'PY'
import sys
import tarfile

crate_file, expected_docs = sys.argv[1:]
needle = f'documentation = "{expected_docs}"'
with tarfile.open(crate_file) as archive:
    names = archive.getnames()
    readmes = [name for name in names if name.count("/") == 1 and name.endswith("/README.md")]
    manifests = [name for name in names if name.count("/") == 1 and name.endswith("/Cargo.toml")]
    if len(readmes) != 1 or len(manifests) != 1:
        sys.stderr.write(f"{crate_file}: expected one README.md and one Cargo.toml at the crate root\n")
        sys.exit(1)
    readme = archive.extractfile(readmes[0]).read()
    if not readme.strip():
        sys.stderr.write(f"{crate_file}: packaged README.md is empty\n")
        sys.exit(1)
    manifest = archive.extractfile(manifests[0]).read().decode()
    if needle not in manifest:
        sys.stderr.write(f"{crate_file}: packaged Cargo.toml missing {needle}\n")
        sys.exit(1)
    sys.stdout.write(str(len(readme.strip())))
PY
    )"; then
      fail=1
      continue
    fi
    echo "check-crates-publish-manifest OK: ${package} ${version} tarball README (${readme_bytes} non-space bytes) documentation=${expected_docs}"
  elif grep -q "failed to select a version for the requirement" "$pkg_log"; then
    rm -f "$pkg_log"
    echo "check-crates-publish-manifest OK: ${package} package list includes README.md; tarball waits on unpublished path deps; documentation=${expected_docs}"
  else
    cat "$pkg_log" >&2
    rm -f "$pkg_log"
    echo "check-crates-publish-manifest: cargo package failed for ${package}" >&2
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo "check-crates-publish-manifest FAILED" >&2
  exit 1
fi

echo "check-crates-publish-manifest OK: ${#ROWS[@]} published crates"
