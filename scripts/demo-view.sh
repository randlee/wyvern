#!/usr/bin/env bash
# Canned Wyvern demo views — list, run one, or run all (user-paced).
#
# Usage:
#   ./scripts/demo-view.sh list
#   ./scripts/demo-view.sh message-long-line
#   ./scripts/demo-view.sh all
#   ./scripts/demo-view.sh message-minimal --viewer safari
#
# Run from Terminal.app (embedded viewer needs a real display session).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VIEWS="$ROOT/scripts/demo-views"
WYVERN="$ROOT/target/debug/wyvern"
UI="$ROOT/ui"

# id|title|what it exercises
CATALOG=(
  "message-minimal|Message (minimal)|Auto-sized OK box"
  "message-long-line|Message (long line)|Single line word-wrap + auto-width"
  "message-paragraph|Message (paragraph)|Pre-wrap multiline + auto-height"
  "message-buttons|Message (3 buttons)|Wider auto-size for button row"
  "message-warning|Message (warning)|Level + ok/cancel layout"
  "message-fixed-size|Message (fixed 520×320)|Explicit JSON size — no auto-shrink"
  "input-text|Input (text)|Compact single-line field"
  "input-multiline|Input (multiline)|Taller auto-sized input"
  "input-file|Input (file)|Editable path + … browse"
  "input-folder|Input (folder)|Editable path + … browse"
  "markdown-short|Markdown (short)|Rendered doc + auto-size"
  "question-radio|Question (radio)|Two-option cards"
  "question-preview|Question (preview)|Option with markdown preview"
  "chrome-frame|Chrome frame|Full-page measure (no compact class)"
  "wizard-sized|Large shell (800×600)|Explicit max-size window"
)

usage() {
  echo "Wyvern canned demo views"
  echo ""
  echo "Usage:"
  echo "  $0 list"
  echo "  $0 <view-id> [--viewer embedded|safari|system|none ...]"
  echo "  $0 all [--viewer ...]     # Enter between views; Ctrl+C to stop"
  echo ""
  echo "Views:"
  list_views
}

list_views() {
  local row id title note
  for row in "${CATALOG[@]}"; do
    IFS='|' read -r id title note <<<"$row"
    printf "  %-22s %s — %s\n" "$id" "$title" "$note"
  done
}

resolve_view() {
  local id="$1"
  local path="$VIEWS/$id.json"
  if [[ ! -f "$path" ]]; then
    echo "Unknown view: $id" >&2
    echo "Run '$0 list' for ids." >&2
    exit 1
  fi
  printf '%s' "$path"
}

build_if_needed() {
  cd "$ROOT"
  cargo build -q -p wyvern -p wyvern-viewer 2>/dev/null || cargo build -p wyvern -p wyvern-viewer
}

wait_for_viewers() {
  local i=0
  while pgrep -x wyvern-viewer >/dev/null 2>&1 && [[ $i -lt 50 ]]; do
    sleep 0.1
    i=$((i + 1))
  done
}

run_view() {
  local id="$1"
  shift
  wait_for_viewers
  local path title note row
  path="$(resolve_view "$id")"
  title="$id"
  note=""
  for row in "${CATALOG[@]}"; do
    if [[ "$row" == "$id|"* ]]; then
      IFS='|' read -r _ title note <<<"$row"
      break
    fi
  done
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  $title"
  echo "  $note"
  echo "  Fixture: $path"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  "$WYVERN" "$(cat "$path")" --ui-root "$UI" "$@"
}

wait_for_user() {
  echo ""
  read -r -p "Press Enter for the next view (Ctrl+C to quit)… " _
}

main() {
  if [[ $# -lt 1 ]]; then
    usage
    exit 0
  fi

  local cmd="$1"
  shift

  case "$cmd" in
    -h|--help|help)
      usage
      ;;
    list)
      list_views
      ;;
    all)
      build_if_needed
      echo "Wyvern demo views — full catalog (user-paced)"
      echo "Worktree: $ROOT"
      local row id
      for row in "${CATALOG[@]}"; do
        IFS='|' read -r id _ _ <<<"$row"
        run_view "$id" "$@"
        wait_for_user
      done
      echo ""
      echo "Catalog complete."
      ;;
    *)
      build_if_needed
      run_view "$cmd" "$@"
      ;;
  esac
}

main "$@"
