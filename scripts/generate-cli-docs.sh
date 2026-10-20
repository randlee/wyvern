#!/usr/bin/env bash
# Regenerate fixtures/cli/wyvern-cli-model.json and site/cli/ (ADR-0027, REQ-0150).
# Installs the sc-docli pin when the `docli` binary is missing or a different version.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PIN="${ROOT}/release/docli-pin.toml"
VERSION="$(
  python3 - "$PIN" <<'PY'
import re
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()
match = re.search(r'(?m)^version\s*=\s*"([^"]+)"\s*$', text)
if not match:
    sys.stderr.write(f"generate-cli-docs: no version in {sys.argv[1]}\n")
    sys.exit(1)
print(match.group(1))
PY
)"

installed="$(docli --version 2>/dev/null || true)"
if [[ "${installed}" != "docli ${VERSION}" ]]; then
  cargo install sc-docli --version "${VERSION}" --locked
fi

cargo run --bin gen-cli-model -- --output fixtures/cli/wyvern-cli-model.json
docli generate --input fixtures/cli/wyvern-cli-model.json --html site/cli
