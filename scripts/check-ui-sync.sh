#!/usr/bin/env bash
# Ensure packaged UI under crates/wyvern/ui/ matches the canonical ui/ tree (ARCH-D-001).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CANONICAL="${ROOT}/ui"
PACKAGED="${ROOT}/crates/wyvern/ui"

if [[ ! -d "$CANONICAL" ]]; then
  echo "check-ui-sync: missing canonical ui/: $CANONICAL" >&2
  exit 1
fi
if [[ ! -d "$PACKAGED" ]]; then
  echo "check-ui-sync: missing packaged ui/: $PACKAGED" >&2
  exit 1
fi

if diff -qr "$CANONICAL" "$PACKAGED" >/dev/null 2>&1; then
  echo "check-ui-sync OK: ui/ matches crates/wyvern/ui/"
  exit 0
fi

echo "check-ui-sync FAILED: ui/ differs from crates/wyvern/ui/" >&2
diff -qr "$CANONICAL" "$PACKAGED" >&2 || true
exit 1
