# RFC-INTENT-001 (Draft): Capability Token Wire Format

## Status

Draft specification for IntentKernel capability tokens. Not yet a production-complete standard.

## Intended Cryptographic Suite

- Signature: **ML-DSA-87**
- Key establishment: **ML-KEM-1024**
- Hashing: **SHA3-384** / **SHA3-512**
- Symmetric protection: **AES-256-GCM**

Current repository status: architecture/specification-first with educational reference code.

## JSON Canonical Form (Human-Readable)

```json
{
  "ver": "IK1",
  "jti": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "intentd://cluster-a/node-1",
  "sub": "process://mail-client/compose",
  "aud": "resource://network/smtp.api.example",
  "intent_id": "intent-3f44f7f8",
  "action": "send_email",
  "resource": {
    "to": ["alice@example.com"],
    "endpoint": "smtp.api.example:443"
  },
  "constraints": {
    "max_uses": 1,
    "ttl_ms": 5000,
    "bytes_out_max": 262144,
    "delegable": false
  },
  "nbf": 1760000000,
  "exp": 1760000005,
  "nonce": "base64:Q2FwTm9uY2UxMjM=",
  "alg": "ML-DSA-87",
  "sig": "base64:..."
}
```

## Required Fields

- Version and unique identity (`ver`, `jti`)
- Issuer/subject/audience (`iss`, `sub`, `aud`)
- Bound intent and action scope (`intent_id`, `action`, `resource`)
- Temporal constraints (`nbf`, `exp`)
- Anti-replay material (`nonce`, replay cache key)
- Cryptographic metadata (`alg`, `sig`)

## Validation Rules

A token is valid only if all checks pass:

1. Signature verifies under trusted issuer key.
2. `nbf <= now <= exp` and TTL policy bounds are respected.
3. `aud`, action, and resource constraints match requested operation.
4. `max_uses` not exhausted.
5. `jti`/nonce not previously consumed for one-shot context.
6. Token state is neither revoked nor expired.

## Security Considerations

- Use canonical serialization before signing.
- Enforce strict clock-skew bounds.
- Protect issuer private keys with hardened key management.
- Treat replay cache as security-critical state.
- Avoid scope over-broadening when deriving delegated tokens.
- Log validation failures for forensic visibility without leaking secret material.
