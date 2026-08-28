#!/usr/bin/env python3
"""Testdata workflow.post — echo finish JSON to an optional marker file."""
import json
import os
import sys

payload = sys.stdin.read()
marker = os.environ.get("WYVERN_POST_MARKER")
if marker:
    with open(marker, "w", encoding="utf-8") as handle:
        handle.write(payload)
        if "--dry-run" in sys.argv:
            handle.write("\n--dry-run\n")
json.loads(payload)
