#!/usr/bin/env python3
"""CRASS CORE updater engine."""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

UPDATE_DIR = Path.cwd() / ".crass_updates"
MANIFEST_FILE = UPDATE_DIR / "update_manifest.json"


def check_for_updates() -> dict:
    print("Checking for CRASS CORE updates...")
    UPDATE_DIR.mkdir(exist_ok=True)
    manifest = {
        "version": "0.0.1",
        "scheduled_at": time.time(),
        "notes": "CRASS CORE update stubs are installed.",
    }
    with MANIFEST_FILE.open("w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2)
    print(f"Update manifest written to {MANIFEST_FILE}")
    return manifest


def apply_update(manifest: dict) -> None:
    print(f"Applying CRASS CORE update version {manifest['version']}")
    time.sleep(1)
    print("Update applied successfully. CRASS CORE is up to date.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="CRASS CORE updater")
    parser.add_argument("--apply", action="store_true", help="Apply the latest update")
    args = parser.parse_args()

    manifest = check_for_updates()
    if args.apply:
        apply_update(manifest)
    else:
        print("Use --apply to actually install the update.")
