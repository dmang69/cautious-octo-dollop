# IBPS / Intent Broker Protocol Specification (Draft)

## Scope

This document defines broker responsibilities and token lifecycle semantics for capability issuance and enforcement.

## Broker Responsibilities

- Accept verified intent claims and contextual metadata.
- Evaluate policy constraints for requested action/resource.
- Mint signed capability tokens with explicit bounds.
- Track lifecycle state and revocation signals.
- Provide validation inputs to enforcement points.
- Emit auditable lifecycle and decision events.

## Token Lifecycle State Machine

`CREATED -> ISSUED -> DELIVERED -> PRESENTED -> VALIDATED -> CONSUMED`

Terminal/side states:

- `EXPIRED`
- `REVOKED`
- `REJECTED`

### State Notes

- **CREATED**: logical token object initialized before signing.
- **ISSUED**: signed and accepted by broker for distribution.
- **DELIVERED**: provided to subject/runtime.
- **PRESENTED**: submitted to interceptor/enforcement path.
- **VALIDATED**: signature, scope, freshness, and policy checks passed.
- **CONSUMED**: one-shot or bounded-use token successfully spent.
- **EXPIRED**: TTL exceeded.
- **REVOKED**: invalidated by authority before normal expiry.
- **REJECTED**: validation failure (signature/scope/time/policy/replay).

## Validation Conditions (High Level)

Validation MUST fail when any of the following hold:

- invalid signature or unknown issuer
- `now < nbf` or `now > exp`
- audience/action/resource mismatch
- replay indicator reuse (`jti`/nonce)
- revoked or already consumed one-shot token

## Implementation Status

Protocol semantics are specification-stage; reference implementations are in progress.
