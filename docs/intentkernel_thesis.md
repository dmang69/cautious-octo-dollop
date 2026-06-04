# IntentKernel Thesis

## Core Thesis

Conventional ambient-authority systems grant broad standing permissions at process start, then attempt to contain misuse with layered controls. IntentKernel proposes an event-scoped capability model where protected actions are authorized only via explicit, short-lived capabilities bound to verified intent.

## Ambient Authority vs Event-Scoped Capability Execution

### Ambient-Authority Systems

- Authority is commonly attached to identity/session and persists longer than a single user action.
- Compromise can convert latent permissions into arbitrary action sequences.
- Containment controls are often additive and policy-heavy.

### Event-Scoped Capability Systems

- Process starts without protected authority.
- Authority is minted per event and constrained by action/resource/time/use.
- Unauthorized actions are prevented when no valid capability is present.
- Post-compromise actionability is reduced to currently valid scope.

## Example

When a user presses **Send** in an email compose view, a token can authorize exactly one send operation for declared destination constraints and a short TTL. The same process cannot silently perform additional sends after the capability is consumed or expired.

## Practical Position

IntentKernel is presented as an architectural direction, not a claim of complete implementation or universal proof today. Confidence depends on conformance to invariants, correctness of trusted components, and independent evaluation.
