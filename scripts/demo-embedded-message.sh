#!/usr/bin/env bash
# Demo embedded message dialog — run from Terminal.app (not Cursor agent shell).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build -q -p wyvern-cli -p wyvern-viewer 2>/dev/null || cargo build -p wyvern-cli -p wyvern-viewer
echo "Launching embedded viewer (window stays until you click OK or close it)..."
echo "Worktree: $ROOT"
exec ./target/debug/wyvern \
  '{"type":"message","title":"Wyvern Demo","message":"HTTP host + embedded wyvern-viewer","level":"info","buttons":"ok"}' \
  --ui-root "$ROOT/ui"
