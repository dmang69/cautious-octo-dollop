# IntentKernel

IntentKernel is a research repository for an event-scoped capability architecture.

It contains design documents, a Rust reference implementation, a legacy IKRL compatibility stack, and a small C reference core. The most direct implementation path in the repo today is the **in-process Rust runtime** built around three major components: **utilities**, **shell**, and **kernel**.

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Reference_Implementation-yellow.svg)](#current-implementation-status)
[![Version](https://img.shields.io/badge/Version-0.1.0-orange.svg)](#)

## Current implementation status

The repo does **not** currently ship a production-ready operating system or a proven security boundary. What it does provide is a set of reference implementations and experiments that exercise the IntentKernel model.

### Primary Rust reference runtime

Under [`rust/`](rust/), the main active path is the `intentos` binary and its three in-process crates:

| # | Component | Crate / Binary | Current role |
|---|-----------|----------------|--------------|
| 1 | Utilities | `intentos-utilities` | In-memory VFS, AI stub gateway, support utilities |
| 2 | Shell | `intentos-shell` | Interactive REPL and command dispatch |
| 3 | Kernel | `intentos-kernel` | Policy evaluation, token minting, capability table, lease tracking |
| — | Entry point | `intentos` | Boots the three components in one process |

Build and run:

```bash
cd rust
cargo run -p intentos --release
```

See [`rust/README.md`](rust/README.md) for details.

### Other code in this repository

The repository also includes:

- A **legacy IKRL daemon stack** in Rust (`capd`, `intentd`, `leasebroker`, `eventscope`, `ai-runtime`, etc.)
- A **C reference capability core** under [`src/reference/`](src/reference/)
- Architecture and protocol documents in [`docs/`](docs/)
- A Tauri desktop shell under [`shell/tauri-app/`](shell/tauri-app/)

Those parts are useful context, but they should not be confused with the current three-component `intentos` runtime.

## What the current Rust runtime demonstrates

The current `intentos-*` crates provide a reference flow for:

- evaluating an intent in [`rust/crates/intentos-kernel/src/policy.rs`](rust/crates/intentos-kernel/src/policy.rs)
- minting and verifying signed capability tokens in [`rust/crates/intentos-kernel/src/token.rs`](rust/crates/intentos-kernel/src/token.rs)
- registering handles and enforcing simple gated syscalls in [`rust/crates/intentos-kernel/src/lib.rs`](rust/crates/intentos-kernel/src/lib.rs)
- exposing gated utilities such as a virtual filesystem in [`rust/crates/intentos-utilities/src/vfs.rs`](rust/crates/intentos-utilities/src/vfs.rs)
- exposing a stubbed AI utility in [`rust/crates/intentos-utilities/src/ai.rs`](rust/crates/intentos-utilities/src/ai.rs)
- driving the flow from an interactive shell in [`rust/crates/intentos-shell/src/lib.rs`](rust/crates/intentos-shell/src/lib.rs)

The included ground-up test at [`rust/crates/intentos/tests/ground_up.rs`](rust/crates/intentos/tests/ground_up.rs) checks that the `intentos-*` path does not depend on the legacy IKRL daemon crates.

## What this repository does not currently prove

To keep the documentation honest:

- it does **not** prove malware, ransomware, spyware, or botnet immunity
- it does **not** provide a formally verified kernel
- it does **not** yet implement a production syscall-interception boundary for the `intentos` path
- it does **not** currently use production post-quantum cryptography in the `intentos-*` runtime
- it does **not** replace Windows, Linux, macOS, Android, or iOS today

This repo is best read as a **reference implementation plus architecture proposal**, not as a finished secure OS.

## Three-component architecture

The active Rust reference runtime is organized around these three layers:

```
user command / event
        |
        v
+--------------------+
| shell              |
| intentos-shell     |
| - parse commands   |
| - session state    |
+---------+----------+
          |
          v
+--------------------+
| kernel             |
| intentos-kernel    |
| - policy           |
| - tokens           |
| - capability table |
| - leases           |
+---------+----------+
          |
          v
+--------------------+
| utilities          |
| intentos-utilities |
| - vfs              |
| - ai gateway stub  |
| - helper tools     |
+--------------------+
```

This is an **in-process model**. It is separate from the older daemon-oriented IKRL path that remains in the workspace.

## Claims table: reference implementation status

| Topic | Status in this repo | Notes |
|-------|---------------------|-------|
| Event-scoped capability model | Implemented as a reference flow | `intentos-kernel` evaluates intents, mints tokens, registers handles, and gates operations |
| Interactive shell workflow | Implemented | `intentos-shell` provides `status`, `flow`, `ls`, `cat`, `write`, `ai infer`, and `lease` commands |
| File access mediation demo | Implemented in-memory | `intentos-utilities` gates reads/writes to an in-memory VFS, not the host filesystem |
| AI capability gating | Implemented as a stub | `AiGateway` returns a local stub response after kernel authorization |
| Lease tracking | Implemented | Lease grant, renew, tick, and list logic exists in `intentos-kernel` |
| Legacy multi-process stack | Present | `capd`, `intentd`, `leasebroker`, `eventscope`, and related crates remain at repo root |
| Bare-metal OS | Partial / experimental | C and low-level kernel sources exist under `src/` |
| Ransomware immunity | **Not proven** | Demos and architectural goals only |
| Spyware immunity | **Not proven** | No formal or system-wide proof |
| Quantum resistance | **Not yet in intentos-* runtime** | Current code uses Ed25519-based development signing |

## Cryptography note

The current `intentos-*` runtime uses [`rust/crates/intentos-kernel/src/crypto.rs`](rust/crates/intentos-kernel/src/crypto.rs), which is a development-oriented signing path built around `ed25519-dalek` and SHA3-384. It is useful for exercising token flow, but it should not be described as a finished post-quantum deployment.

Separate crypto experiments also exist in the legacy Rust workspace under [`crypto/intentkernel-crypto/`](crypto/intentkernel-crypto/).

## Repository structure

```
.
├── README.md                 # This file
├── rust/                     # Primary path: intentos reference runtime
│   └── crates/
│       ├── intentos/
│       ├── intentos-kernel/
│       ├── intentos-shell/
│       └── intentos-utilities/
├── docs/                     # Architecture and protocol documents
├── core/                     # Legacy IKRL daemons (capd, intentd, ai-runtime, ...)
├── kernel/                   # Legacy kernel experiments (eventscope, eBPF, ...)
├── crypto/                   # Legacy token/crypto experiments
├── shell/                    # Tauri desktop shell + iksh CLI
├── src/reference/            # C reference capability core
├── scripts/                  # ISO staging, build helpers
└── governance/               # Project principles (if present)
```

## Documents and references

- Architecture overview: [`docs/architecture_overview.md`](docs/architecture_overview.md)
- IntentKernel thesis: [`docs/intentkernel_thesis.md`](docs/intentkernel_thesis.md)
- UCCS specification: [`docs/uccs_spec.md`](docs/uccs_spec.md)
- IKRL specification: [`docs/ikrl_spec.md`](docs/ikrl_spec.md)
- Intent Broker Protocol: [`docs/ibp_spec.md`](docs/ibp_spec.md)
- Token RFC: [`docs/token_rfc.md`](docs/token_rfc.md)

## License

This repository is released under the [Apache License 2.0](LICENSE).