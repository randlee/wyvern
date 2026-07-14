#!/usr/bin/env bash
# c.9 deletion inventory gate — exit 0 when wyvern-window stack is fully removed.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "verify-c9-deletion: $*" >&2; exit 1; }

# --- wyvern-window crate ---
test ! -d crates/wyvern-window || fail "crates/wyvern-window/ still exists"
test ! -f crates/wyvern-schema/src/icons.rs || fail "icons.rs still exists"
test ! -d boundaries/wyvern-window || fail "boundaries/wyvern-window/ still exists"

if rg -n 'wyvern-window' Cargo.toml 2>/dev/null | rg -v '^\s*#' >/dev/null; then
  fail "Cargo.toml still references wyvern-window (non-comment)"
fi

if rg -l 'wyvern-window' boundaries/ 2>/dev/null | grep -q .; then
  fail "boundaries/ still references wyvern-window"
fi

# --- wyvern-schema icon catalog rework ---
if rg -n 'mod icons|NamedIconSpec' crates/wyvern-schema/src/lib.rs 2>/dev/null | rg -v '^\s*#' | grep -q .; then
  fail "wyvern-schema lib.rs still exports icons module or NamedIconSpec"
fi

if rg -n 'is_named_icon_spec|validate_named_icon' crates/wyvern-schema/src/validate/helpers.rs 2>/dev/null | grep -q .; then
  fail "validate/helpers.rs still has named-icon catalog validation"
fi

if rg -n 'icons::' crates/wyvern-schema/src/validate/ 2>/dev/null | grep -q .; then
  fail "validate/ still imports icons:: catalog"
fi

# --- wyvern CLI GUI serial tests removed ---
GUI_SERIAL_TESTS=(
  cli_valid_chrome_emits_dismissed
  cli_type_message_level_accepted
  cli_valid_message_emits_dismissed
  cli_valid_input_emits_dismissed
  cli_valid_input_file_mode_emits_dismissed
  cli_valid_markdown_file_emits_dismissed
  cli_markdown_md_shorthand_emits_dismissed
  cli_markdown_content_inline_emits_dismissed
  cli_question_auto_dismiss_emits_req_0068
)
for fn in "${GUI_SERIAL_TESTS[@]}"; do
  if rg -n "fn ${fn}\b" crates/wyvern/tests/cli_validation.rs 2>/dev/null | grep -q .; then
    fail "cli_validation.rs still contains GUI serial test: ${fn}"
  fi
done

# --- serial_test dep removed from wyvern ---
if rg -n 'serial_test' crates/wyvern/Cargo.toml 2>/dev/null | rg -v '^\s*#' | grep -q .; then
  fail "crates/wyvern/Cargo.toml still lists serial_test dev-dependency"
fi

echo "verify-c9-deletion: OK"
