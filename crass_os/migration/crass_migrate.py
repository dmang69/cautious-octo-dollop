#!/usr/bin/env python3
"""CRASS MIGRATE — migration engine for legacy systems."""

from __future__ import annotations

import argparse
import os
import shutil
import sys
import time
from pathlib import Path

HOME = Path.home()
TARGET_ROOT = Path.cwd() / "crass_migration_output"

SOURCES = [
    HOME / "Documents",
    HOME / "Pictures",
    HOME / "Desktop",
    HOME / "Downloads",
]


def copy_tree(source: Path, destination: Path) -> None:
    if not source.exists():
        return
    destination.mkdir(parents=True, exist_ok=True)
    for item in source.iterdir():
        target = destination / item.name
        if item.is_dir():
            copy_tree(item, target)
        else:
            shutil.copy2(item, target)


def run_migration(dry_run: bool = False) -> None:
    print("CRASS MIGRATE — scanning system")
    print(f"Target root: {TARGET_ROOT}")
    TARGET_ROOT.mkdir(parents=True, exist_ok=True)

    for source_dir in SOURCES:
        print(f"Migrating {source_dir}")
        if dry_run:
            continue
        copy_tree(source_dir, TARGET_ROOT / source_dir.name)
        time.sleep(0.5)

    print("Migration complete.")
    print("Migrated files are available in:")
    print(TARGET_ROOT)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="CRASS MIGRATE engine")
    parser.add_argument("--dry-run", action="store_true", help="Simulate migration without copying files")
    args = parser.parse_args()

    run_migration(dry_run=args.dry_run)
