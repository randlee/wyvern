#!/usr/bin/env bash
# Release preflight: docli CLI HTML must match gen-cli-model output (ADR-0027).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

bash scripts/generate-cli-docs.sh
git diff --exit-code -- fixtures/cli/wyvern-cli-model.json site/cli/
if [[ -n "$(git status --porcelain -- fixtures/cli/wyvern-cli-model.json site/cli/)" ]]; then
  git status --porcelain -- fixtures/cli/wyvern-cli-model.json site/cli/
  exit 1
fi

echo "release-preflight-cli-docs: OK"
