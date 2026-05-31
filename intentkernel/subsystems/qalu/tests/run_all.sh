#!/usr/bin/env bash
set -e
echo "[Q-ALU] Running Chapter 4 verification..."
python3 src/quaternary_simulator.py

echo "[Q-RAM] Running Chapter 5 GF(4) ECC verification..."
python3 src/qram_gf4.py

echo "[QCPU] Running Chapter 7 integration tests..."
python3 src/qcpu.py

echo "[ALL PASS] Quaternary subsystem verified."
