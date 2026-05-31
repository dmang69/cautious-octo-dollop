# Quaternary Encoding in IntentKernel

## Problem
IntentKernel capability tokens travel on binary buses.
Each wire carries 1 bit. For a 32-bit capability token,
32 physical wires are required.

## Solution
Quaternary (base-4) encoding: each wire carries 1 quat = 2 bits.
A 32-bit capability token needs only 16 physical wires.

## Physical layer
Differential Quaternary Signaling (DQS):
- State 0: ΔV = -600mV
- State 1: ΔV = -200mV
- State 2: ΔV = +200mV
- State 3: ΔV = +600mV

Three decision thresholds at -400mV, 0mV, +400mV.
400mV guard bands exceed 22nm FD-SOI thermal noise floor.

## Capability token format in quaternary
Binary 32-bit token → 16-quat token
Field mapping:
  Bits 31-24 (8 bits) → Quats 15-12 (4 quats): opcode + resource class
  Bits 23-16 (8 bits) → Quats 11-8  (4 quats): resource ID
  Bits 15-8  (8 bits) → Quats 7-4   (4 quats): permission + scope
  Bits 7-0   (8 bits) → Quats 3-0   (4 quats): event ID + nonce

## Error correction
GF(4) [5,3] single-symbol-correcting code protects Q-RAM storage.
Any level-shift error (1, 2, or 3 levels) is corrected in one pass.
This sits beneath qproofd — hardware integrity before proof verification.

## Implementation
See: intentkernel/subsystems/qalu/
