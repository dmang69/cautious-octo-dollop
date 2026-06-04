# IntentKernel Governance Principles

Contributions that affect architecture/protocol behavior must preserve these principles.

1. **No Ambient Authority by Default**  
   New execution paths must not introduce standing protected authority.

2. **Capability Scope Must Stay Narrow**  
   Changes must avoid broad scopes when narrower action/resource constraints are viable.

3. **Temporal and Lifecycle Controls Are Mandatory**  
   Expiry, revocation, and consumption semantics cannot be optional.

4. **Fail Closed on Validation Errors**  
   Ambiguous or invalid capability state must deny protected actions.

5. **Explicit Assumptions**  
   Security-relevant assumptions and exclusions must be documented.

6. **Auditability and Traceability**  
   Security decisions should remain reconstructable from logs and protocol artifacts.

7. **Staged Claims Discipline**  
   Public claims must match implementation maturity; avoid unverifiable absolutes.
