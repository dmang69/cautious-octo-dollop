## IntentKernel MVP Scope and Success Criteria

### Scope (ALL targets)
- **Platforms:** Windows, Linux, macOS, Android, iOS, Embedded/IoT (with Linux as the first MVP host).
- **Feature set:** Intent broker (capd/intentd), capability lifecycle, eventscope enforcement, and SDK surface.
- **AI runtime:** Pluggable orchestration runtime with local/remote execution hooks (no model bound by default).
- **Security guarantees (MVP):** single-use capabilities, hard TTL enforcement, and signature-verified tokens.

### MVP Success Criteria
- Capability tokens can be issued, validated, consumed, and revoked via the broker.
- Token TTLs and single-use enforcement are observable in the demo app.
- Event-scoped authorization rejects actions without valid tokens.
- Control surface exposes broker endpoints for request/consume/invoke.
- Cross-platform adapter registry identifies the active host and lists planned targets.
- Post-quantum crypto integration is wired through a pluggable adapter (simulated in MVP).
