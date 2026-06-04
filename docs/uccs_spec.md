# UCCS Specification (Universal Capability Computing Substrate)

## Definition

**UCCS** is the hardware-independent abstraction layer for IntentKernel semantics across device classes.

## Goals

- Preserve capability semantics across heterogeneous hardware/OS environments.
- Standardize resource descriptors, action scopes, and lifecycle constraints.
- Enable portable policy and token interpretation.
- Keep enforcement contracts stable while underlying platform adapters vary.

## Non-Goals

- Replacing all host kernels in early stages.
- Defining hardware-specific driver internals.
- Guaranteeing identical performance on all targets.

## Core Abstractions

- **Resource Descriptor**: canonical identifier for file/device/network/actuator targets.
- **Action Verb**: constrained operation (`read`, `write`, `connect`, `invoke`, etc.).
- **Capability Envelope**: issuer/subject/audience/scope/time/use constraints.
- **Validation Context**: runtime metadata required for policy + non-replay checks.
- **Audit Event Schema**: normalized logs across deployment modes.

## Portability Considerations

- Platform adapters map host-native handles and APIs to UCCS descriptors.
- Time, randomness, and key-storage quality differ by platform and must be surfaced in trust posture.
- Embedded targets may need compact token transport/validation paths while preserving lifecycle semantics.
- Compatibility mode may enforce subsets first, then converge on stricter behavior as integrations mature.
