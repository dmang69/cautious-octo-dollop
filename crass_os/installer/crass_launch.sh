#!/usr/bin/env bash
set -euo pipefail

MODE=${1:-install}
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "CRASS LAUNCH — platform installer"
echo "Mode: $MODE"

generate_usb() {
  echo "[CRASS LAUNCH] Preparing bootable USB media..."
  echo "This is a placeholder; implement filesystem and block device operations here."
}

run_install() {
  echo "[CRASS LAUNCH] Running CRASS OS installation on Linux/macOS..."
  echo "Detected distribution: $(uname -s)"
  echo "Checking existing partitions and bootloader..."
  echo "Install mode stub complete."
}

run_migrate() {
  echo "[CRASS LAUNCH] Launching CRASS MIGRATE for system migration..."
  python3 "$ROOT_DIR/migration/crass_migrate.py"
}

case "$MODE" in
  install)
    run_install
    ;;
  migrate)
    run_migrate
    ;;
  usb)
    generate_usb
    ;;
  scan)
    echo "Scanning current hardware and partitions..."
    ;;
  *)
    echo "Usage: $0 [install|migrate|usb|scan]"
    exit 1
    ;;
esac
