#!/usr/bin/env bash
# Interactive embedded viewer tour — one blocking dialog at a time.
# You press Enter when ready for the next type (no auto-advance).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
WYVERN="./target/debug/wyvern"
UI="$ROOT/ui"

cargo build -q -p wyvern -p wyvern-viewer 2>/dev/null || cargo build -p wyvern -p wyvern-viewer

next_demo() {
  local step="$1"
  local name="$2"
  local json="$3"
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  $step — $name"
  echo "  Blocks until you submit; JSON prints below."
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  "$WYVERN" "$json" --ui-root "$UI"
}

wait_for_user() {
  echo ""
  read -r -p "Press Enter for the next dialog (Ctrl+C to quit the tour)… " _
}

echo "Wyvern embedded viewer — interactive tour"
echo "Worktree: $ROOT"
echo "You control the pace: each step blocks until you click OK (or close)."

next_demo "1/5" "message" \
  '{"type":"message","title":"Message","message":"Simple modal with OK button.","level":"info","buttons":"ok"}'
wait_for_user

next_demo "2/5" "input (text)" \
  '{"type":"input","title":"Input","message":"Type your name and click OK.","placeholder":"Ada Lovelace","buttons":"ok_cancel"}'
wait_for_user

next_demo "3/5" "markdown" \
  '{"type":"markdown","title":"Markdown","content":"# Release notes\n\n- HTTP host + viewer\n- Blocking JSON to agent","buttons":"ok"}'
wait_for_user

next_demo "4/5" "question" \
  '{"type":"question","questions":[{"question":"Pick output format","header":"Format","options":[{"label":"JSON","description":"Structured result"},{"label":"Plain","description":"Text only"}],"multiSelect":false}]}'
wait_for_user

next_demo "5/5" "chrome" \
  '{"type":"chrome","title":"Chrome frame","status":"Foundation shell — OK to finish tour"}'

echo ""
echo "Tour complete."
