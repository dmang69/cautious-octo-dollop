# IntentKernel Implementation Plan

## Current Status (v1.1)
- Spec suite complete; control surface + broker/eventscope/SDK MVP scaffolding in place.
- Acceptance checklist defines remaining verification for MVP hardening.

## Status Update (Stakeholder Summary)
- Scope: Delivering an IntentKernel MVP that demonstrates event-scoped capability enforcement through the control surface, broker/eventscope gate, and 9-API SDK.
- Progress: Architecture/RFCs are published; MVP scaffolding exists; acceptance checklist enumerates remaining verification work.
- Next: Complete MVP acceptance tests, strengthen demo coverage, and prepare IKRL integration milestones.

## Roadmap Summary
| Version | Timeline | Focus |
|---------|----------|-------|
| v1.1 | Current | Spec suite + MVP scaffolding |
| v1.2 | Months 1–3 | MVP hardening + demo proof |
| v1.3 | Months 4–9 | IKRL integrations + fleet console |
| v1.4 | Months 10–18 | Full SDK release + simulator + mobile |
| v2.0 | Year 2+ | Native microkernel + hardware enforcement |

## v1.2: MVP Hardening (Months 1-3)

### Objective
Demonstrate ransomware immunity on a standard Windows/Linux system with verifiable MVP controls.

### Checklist
- Broker: TTL/uses/revocation fully enforced with audit logs
- Eventscope: syscall/action gate + mismatch denial
- SDK: 9 primitives backed by broker/eventscope
- Control Surface UI + demo proves single-use/TTL
- Tests: acceptance checklist green

### Deliverables
- **intentd** reference implementation for Linux (userspace + SGX)
- **eventscope** interception library for C/Python
- **capd** token issuer using ML-DSA-87 (via liboqs)
- Live demonstration: ransomware binary runs inside IKRL, attempts file encryption, achieves 0 bytes encrypted

### Technical Milestones
| Week | Milestone |
|------|-----------|
| 1 | CBOR encoding/decoding library (TinyCBOR integration) |
| 2 | PQ crypto integration (liboqs ML-DSA-87 signing/verification) |
| 3 | capd prototype — issues tokens using RFC-INTENT-001 format |
| 4 | eventscope shim — intercepts syscalls, presents tokens to kernel |
| 5 | Ransomware immunity demo — WannaCry in IKRL, 0 bytes encrypted |
| 6 | Documentation and test suite |

## v1.3: IKRL Integration (Months 4-9)

### Objective
Deploy IKRL as a production security layer on enterprise infrastructure.

### Deliverables
- **Windows:** VBS-based broker service with Hyper-V micro-VM isolation
- **Linux:** LSM module + eBPF hooks for kernel-level token validation
- **Android:** Privileged system service via Device Owner enrollment
- IKRL management console for enterprise fleet administration
- Background lease dashboard for user visibility

### Deployment Model
- Month 4-5: Pilot on isolated network segment (finance/HR systems)
- Month 6-7: Critical infrastructure rollout (all sensitive data endpoints)
- Month 8-9: General workforce deployment, retire legacy AV/EDR

## v1.4: SDK and Ecosystem (Months 10-18)

### Objective
Enable third-party development of native IntentKernel applications.

### Deliverables
- Full SDK release (Rust, C, Python bindings)
- Developer documentation and tutorials
- App manifest specification
- IKRL simulator for testing capability flows
- Mobile SDK for Android integration
- Native kernel alpha release

## v2.0: Native Hardware (Year 2+)

### Objective
Transition from compatibility layer to bare-metal execution.

### Deliverables
- IntentKernel microkernel for ARM and RISC-V
- SoC reference design with hardware capability enforcement
- Embedded firmware SDK (ESP32, STM32, Raspberry Pi)
- Vehicle/industrial controller firmware
- Cloud hypervisor replacement

### Hardware Partnership Targets
- RISC-V vendors (SiFive, StarFive) for capability-aware silicon
- CHERI-enabled processors for hardware-enforced memory safety
- TPM/HSM vendors for hardware-backed broker key storage

## Success Metrics

| Metric | Target |
|--------|--------|
| Ransomware samples blocked | 100% (structural) |
| Malware detection rate (Sentinel AI) | >99.9% |
| Token validation latency | <1ms |
| Background lease overhead | <2% CPU |
| TCB size | <25,000 LOC |
| Cold boot time (native) | <2 seconds |
| Idle battery improvement | >3x over Android/iOS |
