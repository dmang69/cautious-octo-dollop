# IntentKernel Implementation Plan

## Phase 0 - Specification Baseline (Current)

- Consolidate architecture docs, threat model, invariants, protocol drafts, and governance.
- Align terminology across IntentKernel, UCCS, IKRL, and IBPS.
- Define measurable acceptance criteria for reference demos.

## Phase 1 - Narrow MVP (Near Term)

Goal: demonstrate constrained file/network access with event-scoped capabilities.

- Minimal broker services (`intentd`, `capd`, `leasebroker`, `eventscope`) in development mode.
- Interceptor prototype for two operations:
  - one scoped file read/write path
  - one scoped outbound network request
- Demo scenarios:
  - authorized operation succeeds with valid one-shot token
  - unauthorized operation fails without token
  - token replay and expired token are rejected

## Phase 2 - Reference Implementation Hardening

- Replace educational skeletons with tested components.
- Add deterministic token serialization + signature verification path.
- Expand audit events and revocation propagation behavior.
- Add platform test harnesses for lifecycle transitions.

## Phase 3 - Compatibility Layers

- Implement staged integration adapters for Windows, Linux, Android, macOS, and selected embedded targets.
- Publish deployment profiles documenting enforcement coverage and assumptions.

## Phase 4 - Demos and External Validation

- Publish end-to-end demos for one-shot email send and constrained egress.
- Run independent review of threat model and protocol invariants.
- Track defects against security property claims before broadening guarantees.
