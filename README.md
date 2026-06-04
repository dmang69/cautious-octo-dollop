# IntentKernel

IntentKernel is a capability-secure execution model designed around **zero ambient authority**, **event-scoped capabilities**, and **automatic expiry**. The project is currently specification-first, with a small educational reference skeleton and compatibility-oriented implementation plan.

## Problem

Most mainstream operating systems grant long-lived ambient authority to running processes. That model increases blast radius after compromise and makes least-privilege difficult to sustain over time.

IntentKernel proposes an alternative: processes begin with no default authority and can perform protected actions only when a valid, narrowly scoped capability is presented.

## Design Principles

1. **Zero Ambient Authority**: no process starts with implicit access to protected resources.
2. **Intent Binding**: authority is minted from a verified user/system event.
3. **Action Narrowness**: tokens bind to specific actions and resources.
4. **Automatic Expiry**: every capability has hard temporal bounds.
5. **Auditability**: issuance, validation, consumption, and revocation are recorded.

Example: tapping **Send** in an email app can grant a one-shot capability for exactly one outbound message to an approved destination. After consumption or expiry, the token is no longer valid.

## Architecture Stack

IntentKernel is organized into four layers:

| Layer | Purpose | Primary spec |
|---|---|---|
| **IntentKernel** | Core execution/security model | `docs/intentkernel_thesis.md` |
| **UCCS** | Hardware-independent capability abstractions | `docs/uccs_spec.md` |
| **IKRL** | Compatibility and staged deployment across existing OSes | `docs/ikrl_spec.md` |
| **IBPS** | Broker protocol and token lifecycle semantics | `docs/ibp_spec.md`, `docs/token_rfc.md` |

## Verified Intent Flow

1. A trusted interaction event is captured and normalized (`intentd`).
2. Policy and context checks determine whether a capability may be issued (`capd`).
3. Lease and expiry constraints are attached (`leasebroker`).
4. Event scope is bound to the execution context (`eventscope`).
5. Host-side interceptor validates token + constraints before resource access.
6. Token is consumed, expired, revoked, or rejected per lifecycle rules.

## Deployment Strategy

IntentKernel is intended for staged adoption rather than immediate host-OS replacement:

1. Compatibility-first deployment on existing platforms.
2. Incremental hardening through kernel/service integration.
3. Narrow production pilots (file/network constrained workflows).
4. Broader cross-platform support and SDK maturation.
5. Longer-term native execution targets.

## Post-Quantum Cryptography

Intended cryptographic suite:

- **ML-DSA-87** for capability signatures
- **ML-KEM-1024** for key establishment
- **SHA3-384/SHA3-512** for hashing
- **AES-256-GCM** for symmetric protection

This repository treats these as architectural targets; production-grade implementations and performance profiles are in progress.

## Developer Experience

IntentKernel centers on a compact primitive API surface (documented in `docs/sdk_spec.md`) for capability-aware applications and runtimes.

## Repository Layout

```text
cautious-octo-dollop/
├── README.md
├── LICENSE
├── AUTHORS.md
├── SECURITY.md
├── CONTRIBUTING.md
├── docs/
│   ├── architecture_overview.md
│   ├── threat_model.md
│   ├── formal_invariants.md
│   ├── intentkernel_thesis.md
│   ├── uccs_spec.md
│   ├── ikrl_spec.md
│   ├── ibp_spec.md
│   ├── token_rfc.md
│   ├── sdk_spec.md
│   └── compatibility_model.md
├── src/
│   └── reference/
│       └── capability_core.c
├── roadmap/
│   └── implementation_plan.md
└── governance/
    └── principles.md
```

## Roadmap

See `roadmap/implementation_plan.md` for phased delivery from specification to constrained MVP and compatibility-layer pilots.

## Status

- **Stage:** Specification-first architecture repository
- **Reference code:** Educational, non-production skeletons
- **Focus now:** rigorous specs, threat model, token semantics, and narrow MVP demos

## Contributing

Contributions are welcome. Start with `CONTRIBUTING.md`, then review `governance/principles.md` before proposing architectural or protocol changes.
