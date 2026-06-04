# IntentKernel SDK Specification (Draft)

This SDK defines nine primitive APIs. Each protected action requires an appropriate live capability.

## API Primitives

1. **`draw(frame)`**  
   Submit render output to display surface. Requires display/render capability when surface is protected.

2. **`wait_event(filter?)`**  
   Block until an event/capability arrives. No resource authority implied by waiting alone.

3. **`get_resource(request, cap)`**  
   Acquire a user-approved or broker-approved resource handle under scope constraints.

4. **`put_resource(handle, cap)`**  
   Return, release, or commit resource output according to capability policy.

5. **`network_request(req, cap)`**  
   Perform one outbound request constrained by endpoint/method/byte/time limits.

6. **`schedule_notification(req, cap)`**  
   Schedule one notification under bounded target/time semantics.

7. **`create_capability(spec, parent_cap?)`**  
   Request creation/derivation of a new capability, optionally bounded by parent capability.

8. **`invoke_capability(cap, payload?)`**  
   Present capability for guarded action execution; consumes use count per policy.

9. **`exit(code?)`**  
   Terminate execution context and release outstanding leases/resources.

## Semantics

- Default process authority is empty.
- Capability checks happen at invocation/enforcement boundaries.
- Expired/revoked/exhausted capabilities must fail closed.
- Delegation is denied unless explicitly enabled by policy and parent token.
