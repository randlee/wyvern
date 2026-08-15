#!/usr/bin/env bash
# Phase D wizard tour — curated path through the layout-picker DAG (visual).
# Blocks at each step until you finish the wizard or close the window.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec "$ROOT/scripts/demo-wizard.sh" layout-picker "$@"
