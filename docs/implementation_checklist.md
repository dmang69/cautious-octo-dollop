## IntentKernel Implementation Checklist (Acceptance Tests)

### Intent Broker + Capability Lifecycle
- [ ] Issue a capability token with bounded TTL and use count.
- [ ] Validate signature and reject tampered tokens.
- [ ] Enforce TTL expiry (token invalid after TTL).
- [ ] Enforce single-use consumption (second use denied).
- [ ] Revoke tokens and confirm immediate denial.

### Eventscope Enforcement
- [ ] Map action → capability type and deny mismatches.
- [ ] Log authorization decisions for audit trail.
- [ ] Reject missing or malformed tokens.

### SDK (9 primitives)
- [ ] draw() requires display capability.
- [ ] wait_event() returns next capability event.
- [ ] get_resource()/put_resource() enforce resource-scoped authority.
- [ ] network_request() requires network capability.
- [ ] schedule_notification() requires notification capability.
- [ ] create_capability()/invoke_capability() operate via broker/eventscope.
- [ ] exit() terminates with expected code.

### Cross-Platform Adapter Layer
- [ ] Detect host platform and list supported actions.
- [ ] Provide stubs for Windows/Linux/macOS/Android/iOS/Embedded adapters.
- [ ] Expose adapter metadata via control surface or CLI.

### PQC Integration
- [ ] Token signing and verification use a pluggable crypto adapter.
- [ ] Algorithm identifiers are surfaced in tokens and logs.
- [ ] Switching algorithms requires no broker/eventscope code changes.

### Demo + Control Surface
- [ ] Demo app issues a token and proves single-use enforcement.
- [ ] Control surface endpoints can request, pop, and invoke capabilities.
