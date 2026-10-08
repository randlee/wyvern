#!/usr/bin/env bash
# Check out atm-core at release/atm-core-send-to-pin.toml, verify fixture
# hashes, and ensure the vendored picker page matches Wyvern's canonical copy.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIN="${ROOT}/release/atm-core-send-to-pin.toml"
CHECKOUT="${ATM_CORE_CHECKOUT:-${ROOT}/.cache/atm-core-send-to}"

if [[ ! -f "$PIN" ]]; then
  echo "verify_atm_core_send_to_contract: missing pin manifest: $PIN" >&2
  exit 1
fi

export ROOT PIN CHECKOUT
python3 <<'PY'
from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

root = Path(os.environ["ROOT"])
pin_path = Path(os.environ["PIN"])
checkout = Path(os.environ["CHECKOUT"])

with pin_path.open("rb") as handle:
    pin = tomllib.load(handle)["atm-core"]["send-to"]

repo = pin["repository"]
tag = pin["tag"]
revision = pin["revision"].lower()
wyvern_pin = pin["wyvern_pin"]
fixtures = pin["fixtures"]
assets = pin["assets"]

checkout.parent.mkdir(parents=True, exist_ok=True)

def run(*args: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=cwd, text=True, capture_output=True, check=False)

if not (checkout / ".git").is_dir():
    print(f"verify_atm_core_send_to_contract: cloning {repo} (tag {tag}) -> {checkout}")
    result = run("git", "clone", "--depth", "1", "--branch", tag, repo, str(checkout))
    if result.returncode != 0:
        print(result.stderr or result.stdout, file=sys.stderr)
        raise SystemExit(result.returncode)

head = run("git", "rev-parse", "HEAD", cwd=checkout).stdout.strip().lower()
if head != revision:
    print(
        f"verify_atm_core_send_to_contract: checkout {head} != pin {revision}; fetching pin",
        file=sys.stderr,
    )
    fetch = run("git", "fetch", "--depth", "1", "origin", revision, cwd=checkout)
    if fetch.returncode != 0:
        print(fetch.stderr or fetch.stdout, file=sys.stderr)
        raise SystemExit(fetch.returncode)
    checkout_rev = run("git", "checkout", "--detach", revision, cwd=checkout)
    if checkout_rev.returncode != 0:
        print(checkout_rev.stderr or checkout_rev.stdout, file=sys.stderr)
        raise SystemExit(checkout_rev.returncode)
    head = run("git", "rev-parse", "HEAD", cwd=checkout).stdout.strip().lower()
    if head != revision:
        print(f"verify_atm_core_send_to_contract: still at {head}, expected {revision}", file=sys.stderr)
        raise SystemExit(1)

for item in fixtures:
    rel = item["path"]
    expected = item["sha256"].lower()
    path = checkout / rel
    if not path.is_file():
        print(f"verify_atm_core_send_to_contract: missing fixture {rel}", file=sys.stderr)
        raise SystemExit(1)
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != expected:
        print(
            f"verify_atm_core_send_to_contract: SHA256 drift for {rel}\n"
            f"  expected {expected}\n  actual   {digest}",
            file=sys.stderr,
        )
        raise SystemExit(1)
    print(f"verify_atm_core_send_to_contract OK: {rel}")

def strip_leading_html_comment(text: str) -> str:
    stripped = text.lstrip()
    if not stripped.startswith("<!--"):
        return text
    end = stripped.find("-->")
    if end == -1:
        raise ValueError("unclosed HTML comment in vendored picker")
    return stripped[end + 3 :].lstrip("\n")

vendored = checkout / assets["vendored_picker_html"]
canonical = root / assets["wyvern_canonical_picker_html"]
if not vendored.is_file():
    print(f"verify_atm_core_send_to_contract: missing vendored page {vendored}", file=sys.stderr)
    raise SystemExit(1)
if not canonical.is_file():
    print(f"verify_atm_core_send_to_contract: missing canonical page {canonical}", file=sys.stderr)
    raise SystemExit(1)

v_body = strip_leading_html_comment(vendored.read_text(encoding="utf-8"))
c_body = canonical.read_text(encoding="utf-8")
if v_body != c_body:
    print(
        "verify_atm_core_send_to_contract: pick-member.html body differs from "
        f"atm-core {assets['vendored_picker_html']} (after stripping the vendored header comment)",
        file=sys.stderr,
    )
    raise SystemExit(1)
print("verify_atm_core_send_to_contract OK: vendored pick-member.html matches Wyvern canonical")

picker_input = json.loads((checkout / "docs/plans/phase-aq/fixtures/picker-input-v1.json").read_text())
wizard_path = root / "examples/wizards/atm-pick-member/wizard.json"
wizard = json.loads(wizard_path.read_text())
if wizard.get("config") != picker_input:
    print(
        "verify_atm_core_send_to_contract: examples/wizards/atm-pick-member/wizard.json "
        "config must match atm-core picker-input-v1.json byte-for-byte",
        file=sys.stderr,
    )
    raise SystemExit(1)
print("verify_atm_core_send_to_contract OK: wizard.json config matches picker-input-v1.json")

sh_pin = (checkout / "scripts/send-to/atm-send-to.sh").read_text(encoding="utf-8")
match = re.search(r'WYVERN_PIN="([^"]+)"', sh_pin)
if match is None or match.group(1) != wyvern_pin:
    found = match.group(1) if match else "<missing>"
    print(
        f"verify_atm_core_send_to_contract: atm-send-to.sh WYVERN_PIN={found}, pin expects {wyvern_pin}",
        file=sys.stderr,
    )
    raise SystemExit(1)
print(f"verify_atm_core_send_to_contract OK: atm-core WYVERN_PIN {wyvern_pin}")

# Emit checkout path for follow-on scripts in the same job.
print(checkout)
PY
