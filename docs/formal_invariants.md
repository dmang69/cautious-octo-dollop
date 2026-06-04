# IntentKernel Formal Invariants (Draft)

Let `P` be a process, `R` a protected resource, `A` an action, and `C` a capability token.

1. **Zero Ambient Authority**  
   `start(P) => auth(P) = ∅`.

2. **Intent Binding**  
   `valid(C) => exists intent I : bind(C, I) AND verify(I)`.

3. **Action Narrowness**  
   `valid_for(C, A, R) => (A, R) in scope(C)` and all requests outside `scope(C)` are denied.

4. **Temporal Expiry**  
   `now > exp(C) => invalid(C)`.

5. **Non-Replay**  
   For one-shot capabilities, once `consume(C)` succeeds, subsequent presentations of the same identity (`jti`/nonce tuple) are rejected.

6. **Controlled Delegation**  
   Delegation is denied unless `delegable(C)=true` and derived capability scope/time/uses are no broader than parent constraints.

7. **Revocation Semantics**  
   `revoke(C)` is monotonic: once revoked, `C` cannot transition back to a valid state.

8. **Auditability**  
   Every terminal authorization decision emits an audit event sufficient to reconstruct issuer, subject, audience, scope, time bounds, and outcome.
