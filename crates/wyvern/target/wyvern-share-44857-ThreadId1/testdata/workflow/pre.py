#!/usr/bin/env python3
"""Testdata workflow.pre — emit a config_patch object."""
import json
import sys

print(json.dumps({"config_patch": {"patched": True, "dry_run": "--dry-run" in sys.argv}}))
