# Q-ALU Subsystem

This directory contains the IntentKernel experimental quaternary computing subsystem.
It is designed as a first-class extension point for IntentKernel's capability-secure architecture.

## Components

- `src/qalu.py` — Q-ALU simulation and verification harness
- `src/qram_gf4.py` — Q-RAM memory model with GF(4) ECC
- `src/qcpu.py` — QCPU fetch-decode-execute integration simulator
- `hdl/q_alu.v` — Verilog RTL for the quaternary ALU plus a testbench
- `tests/run_all.sh` — Convenience script to run the entire Q-ALU verification suite

## Architecture

The quaternary subsystem uses base-4 `qit` values to represent data in a post-binary
computing model. It is intended to connect with IntentKernel components such as:

- `capd` — capability daemon interfaces for secure invocation
- `qsimd` — SIMD backend for quaternary vector execution
- `quantumd` — volatile quaternary store and memory semantics
- `qproofd` — integrity and attestation layer for quaternary execution

## Usage

From the `subsystems/qalu/` directory:

```bash
./tests/run_all.sh
```

Or run individual verification programs:

```bash
python3 src/qalu.py
python3 src/qram_gf4.py
python3 src/qcpu.py
```
