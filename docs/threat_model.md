# IntentKernel Threat Model (Early Architecture)

## Scope

This document defines intended security properties and assumptions for a specification-first architecture. It is not a proof of implementation correctness.

## Assets

- Capability private keys and trust anchors
- Token integrity (claims, signatures, lifecycle metadata)
- Intent-event integrity and binding context
- Policy configuration and revocation state
- Audit logs for issuance/validation/consumption decisions
- Protected host resources (files, devices, network paths, actuators)

## Trusted Components (Design Intent)

- Broker signing and validation logic (`capd`, validation path)
- Lease/lifecycle enforcement (`leasebroker`)
- Event binding logic (`intentd`, `eventscope`)
- Interceptor enforcement point
- Time and randomness primitives used by lifecycle/crypto logic

## Untrusted or Less-Trusted Components

- Application processes
- Legacy host OS services and APIs behind compatibility layers
- External networks and remote endpoints
- Third-party libraries unless explicitly included in trusted boundary

## Assumptions

1. Cryptographic primitives are implemented correctly and keys are protected.
2. Trusted components run with integrity protections appropriate for deployment stage.
3. Monotonic or bounded-skew time source exists for TTL enforcement.
4. Event provenance is not trivially forgeable within the selected trust path.
5. Operators configure policy with least-privilege defaults.

## Attacker Goals

- Execute protected actions without valid capability.
- Replay, forge, steal, or alter capability tokens.
- Expand authority beyond declared scope (resource/action/time/use).
- Bypass interception path via legacy interfaces.
- Persist access after expiry/revocation.
- Suppress or tamper with audit evidence.

## Excluded / Partially Addressed Classes

- Physical compromise beyond deployment hardening assumptions
- Side-channel attacks (timing, power, EM, cache) unless mitigations are explicitly implemented
- Denial-of-service resilience guarantees under full resource exhaustion
- Complete compromise of the trusted component set

## Intended Security Properties

- Protected actions require valid, in-scope, non-expired capability presentation.
- Capability authority is narrow (action/resource constrained) and time-bounded.
- Replay is reduced via `jti`, nonce, and freshness validation.
- Revocation and expiry terminate authority within bounded enforcement delay.
- Audit trails allow post-incident reconstruction of authorization decisions.

## Caveat

Claims in this repository describe architectural intent and targeted properties. Formal proof artifacts, independent evaluations, and production hardening are planned work.
