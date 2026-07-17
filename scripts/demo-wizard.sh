#!/usr/bin/env bash
# Phase D wizard samples — interactive embedded viewer demos.
#
# Usage:
#   ./scripts/demo-wizard.sh list
#   ./scripts/demo-wizard.sh layout-picker
#   ./scripts/demo-wizard.sh all
#   ./scripts/demo-wizard.sh two-page --viewer safari
#
# Run from Terminal.app (embedded wyvern-viewer needs a real display session).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLES="$ROOT/examples/wizards"
WYVERN="$ROOT/target/debug/wyvern"

# id|title|what it exercises|ui-root subdir
CATALOG=(
  "layout-picker|Layout picker (DAG)|Solo/pair/trio cards, agent forms, back-nav branch|layout-picker"
  "workspace-hint|Workspace layout|page.layout=workspace + estimated_size chrome|workspace-hint"
  "turbo-flow|Turbo flow graph (dark)|Svelte Flow workspace, node detail/extras, review|turbo-flow"
  "turbo-flow-light|Turbo flow graph (light)|Same flow with config.theme=light|turbo-flow"
  "single-page|Single page (N=1)|Shared wizard chrome, terminal finish on one page|single-page"
  "two-page|Two-page chrome|Back/next across step-1 and step-2|two-page"
)

usage() {
  echo "Wyvern Phase D wizard samples (embedded viewer by default)"
  echo ""
  echo "Usage:"
  echo "  $0 list"
  echo "  $0 <sample-id> [--viewer embedded|safari|system|none ...]"
  echo "  $0 all [--viewer ...]     # Enter between samples; Ctrl+C to stop"
  echo ""
  echo "Samples:"
  list_samples
  echo ""
  echo "Phase C blocking dialogs: ./scripts/demo-view.sh list"
}

list_samples() {
  local row id title note _subdir
  for row in "${CATALOG[@]}"; do
    IFS='|' read -r id title note _subdir <<<"$row"
    printf "  %-16s %s — %s\n" "$id" "$title" "$note"
  done
}

resolve_sample() {
  local id="$1"
  local subdir=""
  local row
  for row in "${CATALOG[@]}"; do
    if [[ "$row" == "$id|"* ]]; then
      IFS='|' read -r _ _ _ subdir <<<"$row"
      break
    fi
  done
  if [[ -z "$subdir" ]]; then
    echo "Unknown sample: $id" >&2
    echo "Run '$0 list' for ids." >&2
    exit 1
  fi
  local ui_root="$EXAMPLES/$subdir"
  local wizard_json="$ui_root/wizard.json"
  if [[ "$id" == "turbo-flow-light" ]]; then
    wizard_json="$ui_root/wizard.light.json"
  fi
  if [[ ! -f "$wizard_json" ]]; then
    echo "Missing fixture: $wizard_json" >&2
    exit 1
  fi
  printf '%s|%s' "$wizard_json" "$ui_root"
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

run_sample() {
  local id="$1"
  shift
  wait_for_viewers
  local resolved wizard_json ui_root title note row
  resolved="$(resolve_sample "$id")"
  wizard_json="${resolved%%|*}"
  ui_root="${resolved#*|}"
  title="$id"
  note=""
  for row in "${CATALOG[@]}"; do
    if [[ "$row" == "$id|"* ]]; then
      IFS='|' read -r _ title note _ <<<"$row"
      break
    fi
  done
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  $title"
  echo "  $note"
  echo "  Fixture: $wizard_json"
  echo "  UI root: $ui_root"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  "$WYVERN" "$(cat "$wizard_json")" --ui-root "$ui_root" "$@"
}

wait_for_user() {
  echo ""
  read -r -p "Press Enter for the next wizard sample (Ctrl+C to quit)… " _
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
      list_samples
      ;;
    all)
      build_if_needed
      echo "Wyvern Phase D wizard samples — full tour (user-paced)"
      echo "Worktree: $ROOT"
      local row id
      for row in "${CATALOG[@]}"; do
        IFS='|' read -r id _ _ <<<"$row"
        run_sample "$id" "$@"
        wait_for_user
      done
      echo ""
      echo "Wizard sample tour complete."
      ;;
    *)
      build_if_needed
      run_sample "$cmd" "$@"
      ;;
  esac
}

main "$@"
