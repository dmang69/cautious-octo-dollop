#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
python3 src/qalu.py
python3 src/qram_gf4.py
python3 src/qcpu.py
