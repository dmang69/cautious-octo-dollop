# IntentKernel Architecture Overview

## Executive Summary

IntentKernel defines a capability-secure execution model intended to reduce unauthorized post-compromise actions by removing ambient authority from process startup and requiring short-lived, intent-bound capabilities for protected operations.

The architecture is structured as four layers: **IntentKernel**, **UCCS**, **IKRL**, and **IBPS**.

## Layer Definitions

### IntentKernel
Core model and invariants for zero ambient authority, event-scoped capability issuance, validation, consumption, expiry, revocation, and audit.

### UCCS
Universal Capability Computing Substrate. Defines hardware-independent abstractions so capability semantics remain consistent across desktop, server, mobile, and embedded targets.

### IKRL
IntentKernel Relief Layer. Compatibility/deployment layer that integrates with existing host systems (Windows, Linux, Android, macOS, embedded runtimes) and enables staged adoption.

### IBPS
Intent Broker Protocol Stack. Defines broker interactions, token lifecycle states, validation decisions, and interoperability contracts between issuer, validator, and enforcement points.

## Main Runtime Components

- **`intentd`**: receives verified interaction events and normalizes intent claims.
- **`capd`**: issues and signs capability tokens under policy constraints.
- **`leasebroker`**: applies TTL/usage constraints, revocation propagation, and lease state transitions.
- **`eventscope`**: binds capability scope to the current execution context.
- **Host-side `interceptor`**: enforcement gateway that validates capability + context before allowing resource access.

## Verified Intent -> Capability -> Execution Flow

1. User/system event is captured and attested (`intentd`).
2. Policy checks determine allowable action/resource bounds.
3. `capd` mints token with action, resource, issuer, subject, audience, `jti`, `nbf`, `exp`, and constraints.
4. `leasebroker` tracks lifecycle and revocation state.
5. `eventscope` presents token to the execution context.
6. Host `interceptor` validates signature, audience, freshness, non-replay controls, and lifecycle state.
7. If valid, action executes within declared bounds; otherwise request is denied.
8. Token transitions to terminal state by consumption, expiry, or revocation.
