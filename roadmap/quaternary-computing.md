# Roadmap: Quaternary Computing Integration

## Status: Subsystem Complete — Integration In Progress

## Phase 1 — Subsystem (COMPLETE)
- [x] Q-ALU: 11 operations, 65,536 tests verified
- [x] Q-RAM: GF(4) ECC, 960 error tests verified
- [x] QCPU: fetch-decode-execute, 6 integration programs
- [x] Verilog RTL: synthesizable q_alu.v with testbench
- [x] Thesis published: DOI 10.5281/zenodo.19332771

## Phase 2 — capd Integration (NEXT)
- [ ] Wrap Q-ALU ops as revocable capability tokens via capd
- [ ] Map 8-quat Q-ISA instruction format to IntentKernel token fields
- [ ] Rust FFI bridge: qalu_cap.rs in daemons/capd/src/

## Phase 3 — Bus Encoding (FUTURE)
- [ ] Replace binary capability token buses with QSB (Quaternary System Bus)
- [ ] 16-quat data bus replaces 32-wire binary bus
- [ ] DQS differential signaling on inter-daemon communication

## Phase 4 — Q-RAM volatile store (FUTURE)
- [ ] Replace quantumd binary volatile store with Q-RAM
- [ ] Memristor 16x16 quat banks
- [ ] 2x storage density for capability token tables

## Phase 5 — Hardware (LONG-TERM)
- [ ] QEMU emulation of quaternary bus
- [ ] OpenROAD/Sky130 synthesis of q_alu.v
- [ ] Test chip validation of DQS noise margins
