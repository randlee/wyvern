#!/usr/bin/env bash
# Ensure packaged extension assets under crates/wyvern/ match repo share/ + scripts/ext/.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CANONICAL_SHARE="${ROOT}/share/wyvern"
PACKAGED_SHARE="${ROOT}/crates/wyvern/share/wyvern"
CANONICAL_SCRIPTS="${ROOT}/scripts/ext"
PACKAGED_SCRIPTS="${ROOT}/crates/wyvern/scripts/ext"

check_pair() {
  local label="$1"
  local canonical="$2"
  local packaged="$3"
  if [[ ! -e "$canonical" ]]; then
    echo "check-share-sync: missing canonical ${label}: $canonical" >&2
    exit 1
  fi
  if [[ ! -e "$packaged" ]]; then
    echo "check-share-sync: missing packaged ${label}: $packaged" >&2
    exit 1
  fi
  if diff -qr "$canonical" "$packaged" >/dev/null 2>&1; then
    echo "check-share-sync OK: ${label} matches crates/wyvern/"
    return 0
  fi
  echo "check-share-sync FAILED: ${label} differs from crates/wyvern/" >&2
  diff -qr "$canonical" "$packaged" >&2 || true
  exit 1
}

check_pair "share/wyvern" "$CANONICAL_SHARE" "$PACKAGED_SHARE"
check_pair "scripts/ext" "$CANONICAL_SCRIPTS" "$PACKAGED_SCRIPTS"
