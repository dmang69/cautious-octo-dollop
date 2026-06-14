/*
 * IntentKernel — capd public API
 *
 * capd (Capability Daemon) is the Capability Verifier: it validates
 * presented tokens before any privileged action is performed.
 *
 * Validation checks
 * -----------------
 *  1. Signature integrity (HMAC-SHA256 stub / ML-DSA-87 in production)
 *  2. TTL not expired
 *  3. One-shot enforcement (token may be used exactly once)
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#ifndef IK_CAPD_H
#define IK_CAPD_H

#include <stdint.h>
#include "intentd/token.h"

/* ------------------------------------------------------------------ */
/* Lifecycle                                                           */
/* ------------------------------------------------------------------ */

int  capd_init(void);
void capd_shutdown(void);

/* ------------------------------------------------------------------ */
/* Validation                                                          */
/* ------------------------------------------------------------------ */

/*
 * capd_validate — validate a capability token.
 *
 * Returns the token's scope (>= 0) on success, or -1 on failure.
 * On success, the token is consumed (one-shot enforcement).
 *
 * Error reasons (printed to stderr):
 *   EXPIRED    — current time > expires_at_unix_ms
 *   BAD_SIG    — signature does not match header fields
 *   REPLAYED   — token_id was already consumed
 */
int capd_validate(const struct CapabilityToken *token);

#endif /* IK_CAPD_H */
